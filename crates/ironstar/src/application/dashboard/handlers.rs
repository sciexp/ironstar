//! Dashboard command handler wiring Decider to EventRepository.
//!
//! This module provides the `handle_dashboard_command` function that creates an
//! EventSourcedAggregate from the Dashboard Decider and SQLite event repository,
//! unifying domain and infrastructure errors via `CommandPipelineError`.

use crate::application::error::CommandPipelineError;
use crate::domain::dashboard::{
    DashboardCommand, DashboardError, DashboardEvent, dashboard_decider,
};
use crate::infrastructure::event_bus::{EventBus, ZenohEventBus, publish_events_fire_and_forget};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use std::sync::Arc;

/// Adapter wrapping SqliteEventRepository to map errors to CommandPipelineError.
pub struct DashboardEventRepositoryAdapter {
    inner: Arc<SqliteEventRepository<DashboardCommand, DashboardEvent>>,
}

impl DashboardEventRepositoryAdapter {
    pub fn new(inner: Arc<SqliteEventRepository<DashboardCommand, DashboardEvent>>) -> Self {
        Self { inner }
    }
}

impl EventRepository<DashboardCommand, DashboardEvent, String, CommandPipelineError>
    for DashboardEventRepositoryAdapter
{
    async fn fetch_events(
        &self,
        command: &DashboardCommand,
    ) -> Result<Vec<(DashboardEvent, String)>, CommandPipelineError> {
        self.inner.fetch_events(command).await.map_err(Into::into)
    }

    async fn save(
        &self,
        events: &[DashboardEvent],
    ) -> Result<Vec<(DashboardEvent, String)>, CommandPipelineError> {
        self.inner.save(events).await.map_err(Into::into)
    }

    async fn version_provider(
        &self,
        event: &DashboardEvent,
    ) -> Result<Option<String>, CommandPipelineError> {
        self.inner.version_provider(event).await.map_err(Into::into)
    }
}

/// Handle a Dashboard command through the EventSourcedAggregate pipeline.
pub async fn handle_dashboard_command<B: EventBus>(
    event_repository: Arc<SqliteEventRepository<DashboardCommand, DashboardEvent>>,
    event_bus: Option<&B>,
    command: DashboardCommand,
) -> Result<Vec<(DashboardEvent, String)>, CommandPipelineError> {
    let repo_adapter = DashboardEventRepositoryAdapter::new(event_repository);

    let mapped_decider = dashboard_decider().map_error(|e: &DashboardError| {
        CommandPipelineError::Dashboard(DashboardError::with_id(e.error_id(), e.kind().clone()))
    });

    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    let saved_events = aggregate.handle(&command).await?;

    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

/// Handle a Dashboard command with Zenoh event bus support.
pub async fn handle_dashboard_command_zenoh(
    event_repository: Arc<SqliteEventRepository<DashboardCommand, DashboardEvent>>,
    event_bus: Option<&ZenohEventBus>,
    command: DashboardCommand,
) -> Result<Vec<(DashboardEvent, String)>, CommandPipelineError> {
    let repo_adapter = DashboardEventRepositoryAdapter::new(event_repository);

    let mapped_decider = dashboard_decider().map_error(|e: &DashboardError| {
        CommandPipelineError::Dashboard(DashboardError::with_id(e.error_id(), e.kind().clone()))
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
    use crate::domain::common::DashboardTitle;
    use crate::domain::dashboard::{DashboardErrorKind, DashboardId};
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
    async fn create_dashboard_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = DashboardCommand::CreateDashboard {
            dashboard_id: DashboardId::from_uuid(Uuid::new_v4()),
            workspace_id: WorkspaceId::from_uuid(Uuid::new_v4()),
            name: DashboardTitle::new("Test Dashboard").expect("valid title"),
            created_at: Utc::now(),
        };

        let result = handle_dashboard_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_ok());
        let events = result.expect("command should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].0,
            DashboardEvent::DashboardCreated { .. }
        ));
    }

    #[tokio::test]
    async fn duplicate_create_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let dash_id = DashboardId::from_uuid(Uuid::new_v4());
        let ws_id = WorkspaceId::from_uuid(Uuid::new_v4());

        let command = DashboardCommand::CreateDashboard {
            dashboard_id: dash_id.clone(),
            workspace_id: ws_id.clone(),
            name: DashboardTitle::new("Dashboard").expect("valid title"),
            created_at: Utc::now(),
        };

        let _ = handle_dashboard_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("first create should succeed");

        let duplicate = DashboardCommand::CreateDashboard {
            dashboard_id: dash_id,
            workspace_id: ws_id,
            name: DashboardTitle::new("Duplicate").expect("valid title"),
            created_at: Utc::now(),
        };

        let result = handle_dashboard_command(repo, NO_EVENT_BUS, duplicate).await;
        assert!(result.is_err());
        match result.expect_err("duplicate should fail") {
            CommandPipelineError::Dashboard(ref e)
                if *e.kind() == DashboardErrorKind::AlreadyExists => {}
            other => panic!("Expected AlreadyExists, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn rename_without_create_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = DashboardCommand::RenameDashboard {
            dashboard_id: DashboardId::from_uuid(Uuid::new_v4()),
            name: DashboardTitle::new("New Name").expect("valid title"),
            renamed_at: Utc::now(),
        };

        let result = handle_dashboard_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_err());
        match result.expect_err("rename without create should fail") {
            CommandPipelineError::Dashboard(ref e)
                if *e.kind() == DashboardErrorKind::NotFound => {}
            other => panic!("Expected NotFound, got: {other:?}"),
        }
    }
}
