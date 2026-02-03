//! SavedQuery command handler wiring Decider to EventRepository.
//!
//! This module provides the `handle_saved_query_command` function that creates an
//! EventSourcedAggregate from the SavedQuery Decider and SQLite event repository,
//! unifying domain and infrastructure errors via `CommandPipelineError`.

use crate::application::error::CommandPipelineError;
use crate::domain::saved_query::{
    SavedQueryCommand, SavedQueryError, SavedQueryEvent, saved_query_decider,
};
use crate::infrastructure::event_bus::{EventBus, ZenohEventBus, publish_events_fire_and_forget};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use std::sync::Arc;

/// Adapter wrapping SqliteEventRepository to map errors to CommandPipelineError.
pub struct SavedQueryEventRepositoryAdapter {
    inner: Arc<SqliteEventRepository<SavedQueryCommand, SavedQueryEvent>>,
}

impl SavedQueryEventRepositoryAdapter {
    pub fn new(inner: Arc<SqliteEventRepository<SavedQueryCommand, SavedQueryEvent>>) -> Self {
        Self { inner }
    }
}

impl EventRepository<SavedQueryCommand, SavedQueryEvent, String, CommandPipelineError>
    for SavedQueryEventRepositoryAdapter
{
    async fn fetch_events(
        &self,
        command: &SavedQueryCommand,
    ) -> Result<Vec<(SavedQueryEvent, String)>, CommandPipelineError> {
        self.inner.fetch_events(command).await.map_err(Into::into)
    }

    async fn save(
        &self,
        events: &[SavedQueryEvent],
    ) -> Result<Vec<(SavedQueryEvent, String)>, CommandPipelineError> {
        self.inner.save(events).await.map_err(Into::into)
    }

    async fn version_provider(
        &self,
        event: &SavedQueryEvent,
    ) -> Result<Option<String>, CommandPipelineError> {
        self.inner.version_provider(event).await.map_err(Into::into)
    }
}

/// Handle a SavedQuery command through the EventSourcedAggregate pipeline.
pub async fn handle_saved_query_command<B: EventBus>(
    event_repository: Arc<SqliteEventRepository<SavedQueryCommand, SavedQueryEvent>>,
    event_bus: Option<&B>,
    command: SavedQueryCommand,
) -> Result<Vec<(SavedQueryEvent, String)>, CommandPipelineError> {
    let repo_adapter = SavedQueryEventRepositoryAdapter::new(event_repository);

    let mapped_decider = saved_query_decider().map_error(|e: &SavedQueryError| {
        CommandPipelineError::SavedQuery(SavedQueryError::with_id(e.error_id(), e.kind().clone()))
    });

    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    let saved_events = aggregate.handle(&command).await?;

    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

/// Handle a SavedQuery command with Zenoh event bus support.
pub async fn handle_saved_query_command_zenoh(
    event_repository: Arc<SqliteEventRepository<SavedQueryCommand, SavedQueryEvent>>,
    event_bus: Option<&ZenohEventBus>,
    command: SavedQueryCommand,
) -> Result<Vec<(SavedQueryEvent, String)>, CommandPipelineError> {
    let repo_adapter = SavedQueryEventRepositoryAdapter::new(event_repository);

    let mapped_decider = saved_query_decider().map_error(|e: &SavedQueryError| {
        CommandPipelineError::SavedQuery(SavedQueryError::with_id(e.error_id(), e.kind().clone()))
    });

    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    let saved_events = aggregate.handle(&command).await?;

    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::analytics::{DatasetRef, SqlQuery};
    use crate::domain::saved_query::{QueryName, SavedQueryErrorKind, SavedQueryId};
    use crate::domain::workspace::WorkspaceId;
    use crate::infrastructure::event_bus::ZenohEventBus;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;
    use uuid::Uuid;

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

    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    #[tokio::test]
    async fn save_query_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = SavedQueryCommand::SaveQuery {
            query_id: SavedQueryId::new(),
            workspace_id: WorkspaceId::from_uuid(Uuid::new_v4()),
            name: QueryName::try_from("Test Query".to_string()).expect("valid name"),
            sql: SqlQuery::try_from("SELECT 1".to_string()).expect("valid sql"),
            dataset_ref: DatasetRef::try_from("hf://test/dataset".to_string()).expect("valid ref"),
            saved_at: Utc::now(),
        };

        let result = handle_saved_query_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_ok());
        let events = result.expect("command should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].0, SavedQueryEvent::QuerySaved { .. }));
    }

    #[tokio::test]
    async fn duplicate_save_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let id = SavedQueryId::new();

        let command = SavedQueryCommand::SaveQuery {
            query_id: id,
            workspace_id: WorkspaceId::from_uuid(Uuid::new_v4()),
            name: QueryName::try_from("Query".to_string()).expect("valid name"),
            sql: SqlQuery::try_from("SELECT 1".to_string()).expect("valid sql"),
            dataset_ref: DatasetRef::try_from("hf://other/dataset".to_string()).expect("valid ref"),
            saved_at: Utc::now(),
        };

        let _ = handle_saved_query_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("first save should succeed");

        let duplicate = SavedQueryCommand::SaveQuery {
            query_id: id,
            workspace_id: WorkspaceId::from_uuid(Uuid::new_v4()),
            name: QueryName::try_from("Duplicate".to_string()).expect("valid name"),
            sql: SqlQuery::try_from("SELECT 2".to_string()).expect("valid sql"),
            dataset_ref: DatasetRef::try_from("hf://other/dataset".to_string()).expect("valid ref"),
            saved_at: Utc::now(),
        };

        let result = handle_saved_query_command(repo, NO_EVENT_BUS, duplicate).await;
        assert!(result.is_err());
        match result.expect_err("duplicate should fail") {
            CommandPipelineError::SavedQuery(ref e)
                if *e.kind() == SavedQueryErrorKind::AlreadyExists => {}
            other => panic!("Expected AlreadyExists, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn rename_without_save_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = SavedQueryCommand::RenameQuery {
            query_id: SavedQueryId::new(),
            name: QueryName::try_from("New Name".to_string()).expect("valid name"),
            renamed_at: Utc::now(),
        };

        let result = handle_saved_query_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_err());
        match result.expect_err("rename without save should fail") {
            CommandPipelineError::SavedQuery(ref e)
                if *e.kind() == SavedQueryErrorKind::NotFound => {}
            other => panic!("Expected NotFound, got: {other:?}"),
        }
    }
}
