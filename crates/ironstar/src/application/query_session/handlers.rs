//! QuerySession command handler wiring Decider to EventRepository.
//!
//! This module provides the `handle_query_session_command` function that creates an
//! EventSourcedAggregate from the QuerySession Decider and SQLite event repository,
//! unifying domain and infrastructure errors via `CommandPipelineError`.
//!
//! # Event publishing
//!
//! After successful persistence, events are published to the event bus using
//! fire-and-forget semantics. This enables SSE subscribers to receive real-time
//! updates while ensuring the event store remains the source of truth.

use crate::application::error::CommandPipelineError;
use crate::domain::query_session::{
    QuerySessionCommand, QuerySessionError, QuerySessionEvent, query_session_decider,
};
use crate::infrastructure::event_bus::{EventBus, ZenohEventBus, publish_events_fire_and_forget};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use std::sync::Arc;

/// Adapter wrapping SqliteEventRepository to map errors to CommandPipelineError.
///
/// fmodel-rust's EventSourcedAggregate requires the repository and decider to
/// share the same error type. This adapter transforms `InfrastructureError`
/// from the underlying repository into `CommandPipelineError::Infrastructure`.
pub struct QuerySessionEventRepositoryAdapter {
    inner: Arc<SqliteEventRepository<QuerySessionCommand, QuerySessionEvent>>,
}

impl QuerySessionEventRepositoryAdapter {
    /// Create a new adapter wrapping the given repository.
    pub fn new(
        inner: Arc<SqliteEventRepository<QuerySessionCommand, QuerySessionEvent>>,
    ) -> Self {
        Self { inner }
    }
}

impl EventRepository<QuerySessionCommand, QuerySessionEvent, String, CommandPipelineError>
    for QuerySessionEventRepositoryAdapter
{
    async fn fetch_events(
        &self,
        command: &QuerySessionCommand,
    ) -> Result<Vec<(QuerySessionEvent, String)>, CommandPipelineError> {
        self.inner.fetch_events(command).await.map_err(Into::into)
    }

    async fn save(
        &self,
        events: &[QuerySessionEvent],
    ) -> Result<Vec<(QuerySessionEvent, String)>, CommandPipelineError> {
        self.inner.save(events).await.map_err(Into::into)
    }

    async fn version_provider(
        &self,
        event: &QuerySessionEvent,
    ) -> Result<Option<String>, CommandPipelineError> {
        self.inner.version_provider(event).await.map_err(Into::into)
    }
}

/// Handle a QuerySession command through the EventSourcedAggregate pipeline.
///
/// This function wires the pure QuerySession Decider to the SQLite event repository,
/// creating a complete event-sourced aggregate. It:
///
/// 1. Wraps the repository in an adapter that maps infrastructure errors
/// 2. Maps decider errors from `QuerySessionError` to `CommandPipelineError::QuerySession`
/// 3. Creates an `EventSourcedAggregate` combining both
/// 4. Handles the command and returns saved events with their versions
/// 5. Publishes saved events to the event bus (fire-and-forget)
///
/// # Arguments
///
/// * `event_repository` - Shared SQLite event repository
/// * `event_bus` - Optional event bus for post-persist notification
/// * `command` - The QuerySession command to handle
///
/// # Returns
///
/// On success, returns the saved events paired with their event IDs (versions).
/// On failure, returns a `CommandPipelineError` from either domain or infrastructure.
pub async fn handle_query_session_command<B: EventBus>(
    event_repository: Arc<SqliteEventRepository<QuerySessionCommand, QuerySessionEvent>>,
    event_bus: Option<&B>,
    command: QuerySessionCommand,
) -> Result<Vec<(QuerySessionEvent, String)>, CommandPipelineError> {
    // Wrap repository to map infrastructure errors
    let repo_adapter = QuerySessionEventRepositoryAdapter::new(event_repository);

    // Map decider errors from QuerySessionError to CommandPipelineError, preserving UUID.
    let mapped_decider = query_session_decider().map_error(|e: &QuerySessionError| {
        CommandPipelineError::QuerySession(QuerySessionError::with_id(
            e.error_id(),
            e.kind().clone(),
        ))
    });

    // Create the EventSourcedAggregate
    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    // Handle the command
    let saved_events = aggregate.handle(&command).await?;

    // Publish events to event bus (fire-and-forget)
    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

/// Handle a QuerySession command with Zenoh event bus support.
///
/// This is a concrete (non-generic) version of `handle_query_session_command` that
/// specifically uses `ZenohEventBus`. This inlines the implementation to ensure
/// the future type is fully monomorphized, allowing axum to verify `Send` bounds.
///
/// Use this function in HTTP handlers. Use the generic `handle_query_session_command`
/// for testing with mock event buses or other implementations.
pub async fn handle_query_session_command_zenoh(
    event_repository: Arc<SqliteEventRepository<QuerySessionCommand, QuerySessionEvent>>,
    event_bus: Option<&ZenohEventBus>,
    command: QuerySessionCommand,
) -> Result<Vec<(QuerySessionEvent, String)>, CommandPipelineError> {
    // Wrap repository to map infrastructure errors
    let repo_adapter = QuerySessionEventRepositoryAdapter::new(event_repository);

    // Map decider errors from QuerySessionError to CommandPipelineError, preserving UUID.
    let mapped_decider = query_session_decider().map_error(|e: &QuerySessionError| {
        CommandPipelineError::QuerySession(QuerySessionError::with_id(
            e.error_id(),
            e.kind().clone(),
        ))
    });

    // Create the EventSourcedAggregate
    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    // Handle the command
    let saved_events = aggregate.handle(&command).await?;

    // Publish events to event bus (fire-and-forget)
    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::query_session::{QuerySessionErrorKind, QuerySessionEvent};
    use crate::domain::analytics::{QueryId, SqlQuery};
    use crate::infrastructure::event_bus::ZenohEventBus;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");

        sqlx::query(include_str!("../../../migrations/001_events.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migration");

        pool
    }

    // Type alias for None event bus to satisfy generic constraint
    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    fn sample_query_id() -> QueryId {
        QueryId::new()
    }

    fn sample_sql() -> SqlQuery {
        SqlQuery::try_from("SELECT 1".to_string()).expect("valid SQL")
    }

    #[tokio::test]
    async fn start_query_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = QuerySessionCommand::StartQuery {
            query_id: sample_query_id(),
            sql: sample_sql(),
            dataset_ref: None,
            chart_config: None,
            started_at: Utc::now(),
        };

        let result = handle_query_session_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_ok());
        let events = result.expect("command should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].0, QuerySessionEvent::QueryStarted { .. }));
    }

    #[tokio::test]
    async fn duplicate_start_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let now = Utc::now();

        // First start succeeds
        let command1 = QuerySessionCommand::StartQuery {
            query_id: sample_query_id(),
            sql: sample_sql(),
            dataset_ref: None,
            chart_config: None,
            started_at: now,
        };
        let _ = handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, command1)
            .await
            .expect("first start should succeed");

        // Second start fails with QueryAlreadyInProgress
        let command2 = QuerySessionCommand::StartQuery {
            query_id: sample_query_id(),
            sql: sample_sql(),
            dataset_ref: None,
            chart_config: None,
            started_at: now,
        };
        let result = handle_query_session_command(repo, NO_EVENT_BUS, command2).await;
        assert!(result.is_err());

        match result.expect_err("duplicate start should fail") {
            CommandPipelineError::QuerySession(ref e)
                if *e.kind() == QuerySessionErrorKind::QueryAlreadyInProgress => {}
            other => panic!("Expected QueryAlreadyInProgress, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn full_query_lifecycle() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let query_id = sample_query_id();
        let now = Utc::now();

        // Start query
        let start = QuerySessionCommand::StartQuery {
            query_id,
            sql: sample_sql(),
            dataset_ref: None,
            chart_config: None,
            started_at: now,
        };
        let _ = handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, start)
            .await
            .expect("start should succeed");

        // Begin execution
        let begin = QuerySessionCommand::BeginExecution {
            query_id,
            began_at: now,
        };
        let events = handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, begin)
            .await
            .expect("begin execution should succeed");
        assert_eq!(events.len(), 1);

        // Complete query
        let complete = QuerySessionCommand::CompleteQuery {
            query_id,
            row_count: 42,
            duration_ms: 150,
            completed_at: now,
        };
        let events = handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, complete)
            .await
            .expect("complete should succeed");
        assert_eq!(events.len(), 1);

        // Reset session
        let reset = QuerySessionCommand::ResetSession { reset_at: now };
        let events = handle_query_session_command(repo, NO_EVENT_BUS, reset)
            .await
            .expect("reset should succeed");
        assert_eq!(events.len(), 1);
    }
}
