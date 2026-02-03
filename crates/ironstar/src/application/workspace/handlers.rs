//! Workspace command handler wiring Decider to EventRepository.
//!
//! This module provides the `handle_workspace_command` function that creates an
//! EventSourcedAggregate from the Workspace Decider and SQLite event repository,
//! unifying domain and infrastructure errors via `CommandPipelineError`.

use crate::application::error::CommandPipelineError;
use crate::domain::workspace::{WorkspaceCommand, WorkspaceError, WorkspaceEvent, workspace_decider};
use crate::infrastructure::event_bus::{EventBus, ZenohEventBus, publish_events_fire_and_forget};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use std::sync::Arc;

/// Adapter wrapping SqliteEventRepository to map errors to CommandPipelineError.
pub struct WorkspaceEventRepositoryAdapter {
    inner: Arc<SqliteEventRepository<WorkspaceCommand, WorkspaceEvent>>,
}

impl WorkspaceEventRepositoryAdapter {
    pub fn new(inner: Arc<SqliteEventRepository<WorkspaceCommand, WorkspaceEvent>>) -> Self {
        Self { inner }
    }
}

impl EventRepository<WorkspaceCommand, WorkspaceEvent, String, CommandPipelineError>
    for WorkspaceEventRepositoryAdapter
{
    async fn fetch_events(
        &self,
        command: &WorkspaceCommand,
    ) -> Result<Vec<(WorkspaceEvent, String)>, CommandPipelineError> {
        self.inner.fetch_events(command).await.map_err(Into::into)
    }

    async fn save(
        &self,
        events: &[WorkspaceEvent],
    ) -> Result<Vec<(WorkspaceEvent, String)>, CommandPipelineError> {
        self.inner.save(events).await.map_err(Into::into)
    }

    async fn version_provider(
        &self,
        event: &WorkspaceEvent,
    ) -> Result<Option<String>, CommandPipelineError> {
        self.inner.version_provider(event).await.map_err(Into::into)
    }
}

/// Handle a Workspace command through the EventSourcedAggregate pipeline.
pub async fn handle_workspace_command<B: EventBus>(
    event_repository: Arc<SqliteEventRepository<WorkspaceCommand, WorkspaceEvent>>,
    event_bus: Option<&B>,
    command: WorkspaceCommand,
) -> Result<Vec<(WorkspaceEvent, String)>, CommandPipelineError> {
    let repo_adapter = WorkspaceEventRepositoryAdapter::new(event_repository);

    let mapped_decider = workspace_decider().map_error(|e: &WorkspaceError| {
        CommandPipelineError::Workspace(WorkspaceError::with_id(e.error_id(), e.kind().clone()))
    });

    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    let saved_events = aggregate.handle(&command).await?;

    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

/// Handle a Workspace command with Zenoh event bus support.
///
/// Concrete (non-generic) version for axum `Send` bounds.
pub async fn handle_workspace_command_zenoh(
    event_repository: Arc<SqliteEventRepository<WorkspaceCommand, WorkspaceEvent>>,
    event_bus: Option<&ZenohEventBus>,
    command: WorkspaceCommand,
) -> Result<Vec<(WorkspaceEvent, String)>, CommandPipelineError> {
    let repo_adapter = WorkspaceEventRepositoryAdapter::new(event_repository);

    let mapped_decider = workspace_decider().map_error(|e: &WorkspaceError| {
        CommandPipelineError::Workspace(WorkspaceError::with_id(e.error_id(), e.kind().clone()))
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
    use crate::domain::workspace::{Visibility, WorkspaceErrorKind, WorkspaceId};
    use crate::domain::UserId;
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

    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    #[tokio::test]
    async fn create_workspace_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = WorkspaceCommand::Create {
            workspace_id: WorkspaceId::new(),
            name: "Test Workspace".to_string(),
            owner_id: UserId::new(),
            visibility: Visibility::Private,
            created_at: Utc::now(),
        };

        let result = handle_workspace_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_ok());
        let events = result.expect("command should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].0, WorkspaceEvent::Created { .. }));
    }

    #[tokio::test]
    async fn duplicate_create_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let id = WorkspaceId::new();

        let command = WorkspaceCommand::Create {
            workspace_id: id.clone(),
            name: "Workspace".to_string(),
            owner_id: UserId::new(),
            visibility: Visibility::Private,
            created_at: Utc::now(),
        };

        let _ = handle_workspace_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("first create should succeed");

        let duplicate = WorkspaceCommand::Create {
            workspace_id: id,
            name: "Duplicate".to_string(),
            owner_id: UserId::new(),
            visibility: Visibility::Private,
            created_at: Utc::now(),
        };

        let result = handle_workspace_command(repo, NO_EVENT_BUS, duplicate).await;
        assert!(result.is_err());
        match result.expect_err("duplicate should fail") {
            CommandPipelineError::Workspace(ref e)
                if *e.kind() == WorkspaceErrorKind::AlreadyExists => {}
            other => panic!("Expected AlreadyExists, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn rename_without_create_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = WorkspaceCommand::Rename {
            workspace_id: WorkspaceId::new(),
            new_name: "New Name".to_string(),
            renamed_at: Utc::now(),
        };

        let result = handle_workspace_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_err());
        match result.expect_err("rename without create should fail") {
            CommandPipelineError::Workspace(ref e)
                if *e.kind() == WorkspaceErrorKind::NotFound => {}
            other => panic!("Expected NotFound, got: {other:?}"),
        }
    }
}
