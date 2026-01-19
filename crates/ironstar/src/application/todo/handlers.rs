//! Todo command handler wiring Decider to EventRepository.
//!
//! This module provides the `handle_todo_command` function that creates an
//! EventSourcedAggregate from the Todo Decider and SQLite event repository,
//! unifying domain and infrastructure errors via `CommandPipelineError`.
//!
//! # Event publishing
//!
//! After successful persistence, events are published to the event bus using
//! fire-and-forget semantics. This enables SSE subscribers to receive real-time
//! updates while ensuring the event store remains the source of truth.

use crate::application::error::CommandPipelineError;
use crate::domain::todo::{TodoCommand, TodoEvent, todo_decider};
use crate::infrastructure::event_bus::{EventBus, ZenohEventBus, publish_events_fire_and_forget};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use std::sync::Arc;

/// Adapter wrapping SqliteEventRepository to map errors to CommandPipelineError.
///
/// fmodel-rust's EventSourcedAggregate requires the repository and decider to
/// share the same error type. This adapter transforms `InfrastructureError`
/// from the underlying repository into `CommandPipelineError::Infrastructure`.
pub struct TodoEventRepositoryAdapter {
    inner: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>,
}

impl TodoEventRepositoryAdapter {
    /// Create a new adapter wrapping the given repository.
    pub fn new(inner: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>) -> Self {
        Self { inner }
    }
}

impl EventRepository<TodoCommand, TodoEvent, String, CommandPipelineError>
    for TodoEventRepositoryAdapter
{
    async fn fetch_events(
        &self,
        command: &TodoCommand,
    ) -> Result<Vec<(TodoEvent, String)>, CommandPipelineError> {
        self.inner.fetch_events(command).await.map_err(Into::into)
    }

    async fn save(
        &self,
        events: &[TodoEvent],
    ) -> Result<Vec<(TodoEvent, String)>, CommandPipelineError> {
        self.inner.save(events).await.map_err(Into::into)
    }

    async fn version_provider(
        &self,
        event: &TodoEvent,
    ) -> Result<Option<String>, CommandPipelineError> {
        self.inner
            .version_provider(event)
            .await
            .map_err(Into::into)
    }
}

/// Handle a Todo command through the EventSourcedAggregate pipeline.
///
/// This function wires the pure Todo Decider to the SQLite event repository,
/// creating a complete event-sourced aggregate. It:
///
/// 1. Wraps the repository in an adapter that maps infrastructure errors
/// 2. Maps decider errors from `TodoError` to `CommandPipelineError::Todo`
/// 3. Creates an `EventSourcedAggregate` combining both
/// 4. Handles the command and returns saved events with their versions
/// 5. Publishes saved events to the event bus (fire-and-forget)
///
/// # Arguments
///
/// * `event_repository` - Shared SQLite event repository
/// * `event_bus` - Optional event bus for post-persist notification
/// * `command` - The Todo command to handle
///
/// # Returns
///
/// On success, returns the saved events paired with their event IDs (versions).
/// On failure, returns a `CommandPipelineError` from either domain or infrastructure.
///
/// # Event publishing
///
/// When an event bus is provided, saved events are published after successful
/// persistence using fire-and-forget semantics. Publish errors are logged but
/// do not fail the command, as the event store is the source of truth.
///
/// # Example
///
/// ```rust,ignore
/// use ironstar::application::todo::handle_todo_command;
/// use ironstar::domain::todo::{TodoCommand, TodoId};
/// use chrono::Utc;
///
/// let repo = Arc::new(SqliteEventRepository::new(pool));
/// let event_bus = ZenohEventBus::new(session);
/// let command = TodoCommand::Create {
///     id: TodoId::new(),
///     text: "Buy groceries".to_string(),
///     created_at: Utc::now(),
/// };
///
/// let events = handle_todo_command(repo, Some(&event_bus), command).await?;
/// ```
pub async fn handle_todo_command<B: EventBus>(
    event_repository: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>,
    event_bus: Option<&B>,
    command: TodoCommand,
) -> Result<Vec<(TodoEvent, String)>, CommandPipelineError> {
    // Wrap repository to map infrastructure errors
    let repo_adapter = TodoEventRepositoryAdapter::new(event_repository);

    // Map decider errors from TodoError to CommandPipelineError
    let mapped_decider = todo_decider().map_error(|e| CommandPipelineError::Todo(e.kind().clone()));

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

/// Handle a Todo command with Zenoh event bus support.
///
/// This is a concrete (non-generic) version of `handle_todo_command` that
/// specifically uses `ZenohEventBus`. This inlines the implementation to ensure
/// the future type is fully monomorphized, allowing axum to verify `Send` bounds.
///
/// Use this function in HTTP handlers. Use the generic `handle_todo_command`
/// for testing with mock event buses or other implementations.
pub async fn handle_todo_command_zenoh(
    event_repository: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>,
    event_bus: Option<&ZenohEventBus>,
    command: TodoCommand,
) -> Result<Vec<(TodoEvent, String)>, CommandPipelineError> {
    // Wrap repository to map infrastructure errors
    let repo_adapter = TodoEventRepositoryAdapter::new(event_repository);

    // Map decider errors from TodoError to CommandPipelineError
    let mapped_decider = todo_decider().map_error(|e| CommandPipelineError::Todo(e.kind().clone()));

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
mod tests {
    use super::*;
    use crate::domain::todo::{TodoErrorKind, TodoId};
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

    #[tokio::test]
    async fn create_todo_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let id = TodoId::new();
        let now = Utc::now();
        let command = TodoCommand::Create {
            id,
            text: "Test todo".to_string(),
            created_at: now,
        };

        let result = handle_todo_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_ok());
        let events = result.expect("command should succeed");
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn create_duplicate_fails_with_domain_error() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let id = TodoId::new();
        let now = Utc::now();

        // First create succeeds
        let command1 = TodoCommand::Create {
            id,
            text: "Test todo".to_string(),
            created_at: now,
        };
        let _ = handle_todo_command(Arc::clone(&repo), NO_EVENT_BUS, command1)
            .await
            .expect("first create should succeed");

        // Second create fails with AlreadyExists
        let command2 = TodoCommand::Create {
            id,
            text: "Duplicate".to_string(),
            created_at: now,
        };
        let result = handle_todo_command(repo, NO_EVENT_BUS, command2).await;
        assert!(result.is_err());

        match result.expect_err("duplicate create should fail") {
            CommandPipelineError::Todo(TodoErrorKind::AlreadyExists) => {}
            other => panic!("Expected AlreadyExists, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn complete_nonexistent_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = TodoCommand::Complete {
            id: TodoId::new(),
            completed_at: Utc::now(),
        };

        let result = handle_todo_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_err());

        match result.expect_err("complete nonexistent should fail") {
            CommandPipelineError::Todo(TodoErrorKind::CannotComplete) => {}
            other => panic!("Expected CannotComplete, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn full_lifecycle() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let id = TodoId::new();
        let now = Utc::now();

        // Create
        let create = TodoCommand::Create {
            id,
            text: "Lifecycle test".to_string(),
            created_at: now,
        };
        let _ = handle_todo_command(Arc::clone(&repo), NO_EVENT_BUS, create)
            .await
            .expect("create should succeed");

        // Complete
        let complete = TodoCommand::Complete {
            id,
            completed_at: now,
        };
        let events = handle_todo_command(Arc::clone(&repo), NO_EVENT_BUS, complete)
            .await
            .expect("complete should succeed");
        assert_eq!(events.len(), 1);

        // Uncomplete
        let uncomplete = TodoCommand::Uncomplete {
            id,
            uncompleted_at: now,
        };
        let events = handle_todo_command(Arc::clone(&repo), NO_EVENT_BUS, uncomplete)
            .await
            .expect("uncomplete should succeed");
        assert_eq!(events.len(), 1);

        // Delete
        let delete = TodoCommand::Delete {
            id,
            deleted_at: now,
        };
        let events = handle_todo_command(Arc::clone(&repo), NO_EVENT_BUS, delete)
            .await
            .expect("delete should succeed");
        assert_eq!(events.len(), 1);

        // Delete again (idempotent)
        let delete_again = TodoCommand::Delete {
            id,
            deleted_at: now,
        };
        let events = handle_todo_command(repo, NO_EVENT_BUS, delete_again)
            .await
            .expect("idempotent delete should succeed");
        assert!(events.is_empty()); // Idempotent: no new events
    }
}
