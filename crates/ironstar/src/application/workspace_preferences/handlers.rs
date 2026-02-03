//! WorkspacePreferences command handler wiring Decider to EventRepository.
//!
//! This module provides the `handle_workspace_preferences_command` function that
//! creates an EventSourcedAggregate from the WorkspacePreferences Decider and
//! SQLite event repository, unifying domain and infrastructure errors via
//! `CommandPipelineError`.

use crate::application::error::CommandPipelineError;
use crate::domain::workspace_preferences::{
    WorkspacePreferencesCommand, WorkspacePreferencesError, WorkspacePreferencesEvent,
    workspace_preferences_decider,
};
use crate::infrastructure::event_bus::{EventBus, ZenohEventBus, publish_events_fire_and_forget};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use std::sync::Arc;

/// Adapter wrapping SqliteEventRepository to map errors to CommandPipelineError.
pub struct WorkspacePreferencesEventRepositoryAdapter {
    inner: Arc<SqliteEventRepository<WorkspacePreferencesCommand, WorkspacePreferencesEvent>>,
}

impl WorkspacePreferencesEventRepositoryAdapter {
    pub fn new(
        inner: Arc<SqliteEventRepository<WorkspacePreferencesCommand, WorkspacePreferencesEvent>>,
    ) -> Self {
        Self { inner }
    }
}

impl
    EventRepository<
        WorkspacePreferencesCommand,
        WorkspacePreferencesEvent,
        String,
        CommandPipelineError,
    > for WorkspacePreferencesEventRepositoryAdapter
{
    async fn fetch_events(
        &self,
        command: &WorkspacePreferencesCommand,
    ) -> Result<Vec<(WorkspacePreferencesEvent, String)>, CommandPipelineError> {
        self.inner.fetch_events(command).await.map_err(Into::into)
    }

    async fn save(
        &self,
        events: &[WorkspacePreferencesEvent],
    ) -> Result<Vec<(WorkspacePreferencesEvent, String)>, CommandPipelineError> {
        self.inner.save(events).await.map_err(Into::into)
    }

    async fn version_provider(
        &self,
        event: &WorkspacePreferencesEvent,
    ) -> Result<Option<String>, CommandPipelineError> {
        self.inner.version_provider(event).await.map_err(Into::into)
    }
}

/// Handle a WorkspacePreferences command through the EventSourcedAggregate pipeline.
pub async fn handle_workspace_preferences_command<B: EventBus>(
    event_repository: Arc<
        SqliteEventRepository<WorkspacePreferencesCommand, WorkspacePreferencesEvent>,
    >,
    event_bus: Option<&B>,
    command: WorkspacePreferencesCommand,
) -> Result<Vec<(WorkspacePreferencesEvent, String)>, CommandPipelineError> {
    let repo_adapter = WorkspacePreferencesEventRepositoryAdapter::new(event_repository);

    let mapped_decider =
        workspace_preferences_decider().map_error(|e: &WorkspacePreferencesError| {
            CommandPipelineError::WorkspacePreferences(WorkspacePreferencesError::with_id(
                e.error_id(),
                e.kind().clone(),
            ))
        });

    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    let saved_events = aggregate.handle(&command).await?;

    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

/// Handle a WorkspacePreferences command with Zenoh event bus support.
pub async fn handle_workspace_preferences_command_zenoh(
    event_repository: Arc<
        SqliteEventRepository<WorkspacePreferencesCommand, WorkspacePreferencesEvent>,
    >,
    event_bus: Option<&ZenohEventBus>,
    command: WorkspacePreferencesCommand,
) -> Result<Vec<(WorkspacePreferencesEvent, String)>, CommandPipelineError> {
    let repo_adapter = WorkspacePreferencesEventRepositoryAdapter::new(event_repository);

    let mapped_decider =
        workspace_preferences_decider().map_error(|e: &WorkspacePreferencesError| {
            CommandPipelineError::WorkspacePreferences(WorkspacePreferencesError::with_id(
                e.error_id(),
                e.kind().clone(),
            ))
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
    use crate::domain::workspace::WorkspaceId;
    use crate::domain::workspace_preferences::{CatalogUri, WorkspacePreferencesErrorKind};
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
    async fn initialize_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = WorkspacePreferencesCommand::InitializeWorkspacePreferences {
            workspace_id: WorkspaceId::from_uuid(Uuid::new_v4()),
            initialized_at: Utc::now(),
        };

        let result = handle_workspace_preferences_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_ok());
        let events = result.expect("command should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].0,
            WorkspacePreferencesEvent::WorkspacePreferencesInitialized { .. }
        ));
    }

    #[tokio::test]
    async fn duplicate_initialize_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let ws_id = WorkspaceId::from_uuid(Uuid::new_v4());

        let command = WorkspacePreferencesCommand::InitializeWorkspacePreferences {
            workspace_id: ws_id,
            initialized_at: Utc::now(),
        };

        let _ = handle_workspace_preferences_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("first initialize should succeed");

        let duplicate = WorkspacePreferencesCommand::InitializeWorkspacePreferences {
            workspace_id: ws_id,
            initialized_at: Utc::now(),
        };

        let result = handle_workspace_preferences_command(repo, NO_EVENT_BUS, duplicate).await;
        assert!(result.is_err());
        match result.expect_err("duplicate should fail") {
            CommandPipelineError::WorkspacePreferences(ref e)
                if *e.kind() == WorkspacePreferencesErrorKind::AlreadyInitialized => {}
            other => panic!("Expected AlreadyInitialized, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_catalog_without_initialize_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = WorkspacePreferencesCommand::SetDefaultCatalog {
            workspace_id: WorkspaceId::from_uuid(Uuid::new_v4()),
            catalog_uri: CatalogUri::try_from("ducklake:test".to_string()).expect("valid uri"),
            set_at: Utc::now(),
        };

        let result = handle_workspace_preferences_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_err());
        match result.expect_err("set catalog without initialize should fail") {
            CommandPipelineError::WorkspacePreferences(ref e)
                if *e.kind() == WorkspacePreferencesErrorKind::NotInitialized => {}
            other => panic!("Expected NotInitialized, got: {other:?}"),
        }
    }
}
