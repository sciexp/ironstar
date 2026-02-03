//! Catalog command handler wiring Decider to EventRepository.
//!
//! This module provides the `handle_catalog_command` function that creates an
//! EventSourcedAggregate from the Catalog Decider and SQLite event repository,
//! unifying domain and infrastructure errors via `CommandPipelineError`.
//!
//! The Catalog aggregate is simpler than QuerySession: it has no spawn-after-persist
//! pattern since catalog operations (select, refresh metadata) are synchronous
//! from the Decider's perspective.

use crate::application::error::CommandPipelineError;
use crate::domain::catalog::{CatalogCommand, CatalogError, CatalogEvent, catalog_decider};
use crate::infrastructure::event_bus::{EventBus, ZenohEventBus, publish_events_fire_and_forget};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use std::sync::Arc;

/// Adapter wrapping SqliteEventRepository to map errors to CommandPipelineError.
///
/// fmodel-rust's EventSourcedAggregate requires the repository and decider to
/// share the same error type. This adapter transforms `InfrastructureError`
/// from the underlying repository into `CommandPipelineError::Infrastructure`.
pub struct CatalogEventRepositoryAdapter {
    inner: Arc<SqliteEventRepository<CatalogCommand, CatalogEvent>>,
}

impl CatalogEventRepositoryAdapter {
    /// Create a new adapter wrapping the given repository.
    pub fn new(inner: Arc<SqliteEventRepository<CatalogCommand, CatalogEvent>>) -> Self {
        Self { inner }
    }
}

impl EventRepository<CatalogCommand, CatalogEvent, String, CommandPipelineError>
    for CatalogEventRepositoryAdapter
{
    async fn fetch_events(
        &self,
        command: &CatalogCommand,
    ) -> Result<Vec<(CatalogEvent, String)>, CommandPipelineError> {
        self.inner.fetch_events(command).await.map_err(Into::into)
    }

    async fn save(
        &self,
        events: &[CatalogEvent],
    ) -> Result<Vec<(CatalogEvent, String)>, CommandPipelineError> {
        self.inner.save(events).await.map_err(Into::into)
    }

    async fn version_provider(
        &self,
        event: &CatalogEvent,
    ) -> Result<Option<String>, CommandPipelineError> {
        self.inner.version_provider(event).await.map_err(Into::into)
    }
}

/// Handle a Catalog command through the EventSourcedAggregate pipeline.
///
/// This function wires the pure Catalog Decider to the SQLite event repository,
/// creating a complete event-sourced aggregate. It:
///
/// 1. Wraps the repository in an adapter that maps infrastructure errors
/// 2. Maps decider errors from `CatalogError` to `CommandPipelineError::Catalog`
/// 3. Creates an `EventSourcedAggregate` combining both
/// 4. Handles the command and returns saved events with their versions
/// 5. Publishes saved events to the event bus (fire-and-forget)
pub async fn handle_catalog_command<B: EventBus>(
    event_repository: Arc<SqliteEventRepository<CatalogCommand, CatalogEvent>>,
    event_bus: Option<&B>,
    command: CatalogCommand,
) -> Result<Vec<(CatalogEvent, String)>, CommandPipelineError> {
    let repo_adapter = CatalogEventRepositoryAdapter::new(event_repository);

    let mapped_decider = catalog_decider().map_error(|e: &CatalogError| {
        CommandPipelineError::Catalog(CatalogError::with_id(e.error_id(), e.kind().clone()))
    });

    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    let saved_events = aggregate.handle(&command).await?;

    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

/// Handle a Catalog command with Zenoh event bus support.
///
/// Concrete (non-generic) version of `handle_catalog_command` that uses
/// `ZenohEventBus`. This inlines the implementation to ensure the future
/// type is fully monomorphized, allowing axum to verify `Send` bounds.
///
/// Use this function in HTTP handlers. Use the generic `handle_catalog_command`
/// for testing with mock event buses or other implementations.
pub async fn handle_catalog_command_zenoh(
    event_repository: Arc<SqliteEventRepository<CatalogCommand, CatalogEvent>>,
    event_bus: Option<&ZenohEventBus>,
    command: CatalogCommand,
) -> Result<Vec<(CatalogEvent, String)>, CommandPipelineError> {
    let repo_adapter = CatalogEventRepositoryAdapter::new(event_repository);

    let mapped_decider = catalog_decider().map_error(|e: &CatalogError| {
        CommandPipelineError::Catalog(CatalogError::with_id(e.error_id(), e.kind().clone()))
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
    use crate::domain::catalog::{CatalogErrorKind, CatalogMetadata, CatalogRef, DatasetInfo};
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

    fn sample_catalog_ref() -> CatalogRef {
        CatalogRef::try_from("ducklake:my_catalog".to_string()).expect("valid catalog ref")
    }

    fn sample_metadata() -> CatalogMetadata {
        CatalogMetadata {
            datasets: vec![DatasetInfo {
                name: "test_dataset".to_string(),
                table_count: 5,
                schema_version: "1.0".to_string(),
            }],
            last_refreshed: Utc::now(),
        }
    }

    #[tokio::test]
    async fn select_catalog_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = CatalogCommand::SelectCatalog {
            catalog_ref: sample_catalog_ref(),
            selected_at: Utc::now(),
        };

        let result = handle_catalog_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_ok());
        let events = result.expect("command should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].0, CatalogEvent::CatalogSelected { .. }));
    }

    #[tokio::test]
    async fn refresh_metadata_without_selection_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = CatalogCommand::RefreshCatalogMetadata {
            metadata: sample_metadata(),
            refreshed_at: Utc::now(),
        };

        let result = handle_catalog_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_err());

        match result.expect_err("refresh without selection should fail") {
            CommandPipelineError::Catalog(ref e)
                if *e.kind() == CatalogErrorKind::NoCatalogSelected => {}
            other => panic!("Expected NoCatalogSelected, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn select_then_refresh_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let now = Utc::now();

        // Select catalog
        let select = CatalogCommand::SelectCatalog {
            catalog_ref: sample_catalog_ref(),
            selected_at: now,
        };
        let _ = handle_catalog_command(Arc::clone(&repo), NO_EVENT_BUS, select)
            .await
            .expect("select should succeed");

        // Refresh metadata
        let refresh = CatalogCommand::RefreshCatalogMetadata {
            metadata: sample_metadata(),
            refreshed_at: now,
        };
        let events = handle_catalog_command(repo, NO_EVENT_BUS, refresh)
            .await
            .expect("refresh should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].0,
            CatalogEvent::CatalogMetadataRefreshed { .. }
        ));
    }
}
