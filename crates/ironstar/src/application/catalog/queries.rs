//! Catalog query handlers for View-based state computation.
//!
//! Query handlers fetch events and fold them through the CatalogView to compute
//! current state on demand. The Catalog is a singleton aggregate ("default-catalog"),
//! so queries do not require an aggregate ID parameter.

use crate::domain::views::{CatalogViewState, catalog_view};
use crate::domain::{CatalogEvent, CatalogMetadata};
use crate::infrastructure::error::InfrastructureError;
use crate::infrastructure::event_store::SqliteEventRepository;

/// Query the current catalog state by replaying events through the View.
///
/// Returns the full `CatalogViewState` including catalog ref and metadata.
/// Returns the View's initial state (no catalog selected) if no events exist.
pub async fn query_catalog_state<C>(
    repo: &SqliteEventRepository<C, CatalogEvent>,
) -> Result<CatalogViewState, InfrastructureError> {
    let events = repo
        .fetch_events_by_aggregate("Catalog", "default-catalog")
        .await?;

    let view = catalog_view();
    let initial_state = (view.initial_state)();

    let state = events
        .iter()
        .fold(initial_state, |state, (event, _version)| {
            (view.evolve)(&state, event)
        });

    Ok(state)
}

/// Query catalog metadata for the currently active catalog.
///
/// Convenience wrapper over `query_catalog_state` that extracts just the
/// metadata. Returns `None` if no catalog is selected or metadata has not
/// been refreshed.
pub async fn query_catalog_metadata<C>(
    repo: &SqliteEventRepository<C, CatalogEvent>,
) -> Result<Option<CatalogMetadata>, InfrastructureError> {
    let state = query_catalog_state(repo).await?;
    Ok(state.metadata)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::application::catalog::handle_catalog_command;
    use crate::domain::CatalogCommand;
    use crate::domain::{CatalogRef, DatasetInfo};
    use crate::infrastructure::event_bus::ZenohEventBus;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;

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
    async fn query_empty_returns_no_catalog() {
        let pool = create_test_pool().await;
        let repo: SqliteEventRepository<CatalogCommand, CatalogEvent> =
            SqliteEventRepository::new(pool);

        let state = query_catalog_state(&repo)
            .await
            .expect("query should succeed");

        assert!(!state.has_catalog());
        assert!(!state.has_metadata());
    }

    #[tokio::test]
    async fn query_after_select_returns_catalog() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let catalog_ref = CatalogRef::new("ducklake:test").unwrap();
        let command = CatalogCommand::SelectCatalog {
            catalog_ref: catalog_ref.clone(),
            selected_at: Utc::now(),
        };
        handle_catalog_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("select should succeed");

        let state = query_catalog_state(&repo)
            .await
            .expect("query should succeed");

        assert!(state.has_catalog());
        assert_eq!(state.catalog_ref.as_ref(), Some(&catalog_ref));
        assert!(!state.has_metadata());
    }

    #[tokio::test]
    async fn query_metadata_returns_none_before_refresh() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = CatalogCommand::SelectCatalog {
            catalog_ref: CatalogRef::new("ducklake:test").unwrap(),
            selected_at: Utc::now(),
        };
        handle_catalog_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("select should succeed");

        let metadata = query_catalog_metadata(&repo)
            .await
            .expect("query should succeed");

        assert!(metadata.is_none());
    }

    #[tokio::test]
    async fn query_metadata_returns_data_after_refresh() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let select = CatalogCommand::SelectCatalog {
            catalog_ref: CatalogRef::new("ducklake:test").unwrap(),
            selected_at: Utc::now(),
        };
        handle_catalog_command(Arc::clone(&repo), NO_EVENT_BUS, select)
            .await
            .expect("select should succeed");

        let refresh = CatalogCommand::RefreshCatalogMetadata {
            metadata: CatalogMetadata {
                datasets: vec![DatasetInfo {
                    name: "genomics".to_string(),
                    table_count: 5,
                    schema_version: "1.0".to_string(),
                }],
                last_refreshed: Utc::now(),
            },
            refreshed_at: Utc::now(),
        };
        handle_catalog_command(Arc::clone(&repo), NO_EVENT_BUS, refresh)
            .await
            .expect("refresh should succeed");

        let metadata = query_catalog_metadata(&repo)
            .await
            .expect("query should succeed");

        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().datasets.len(), 1);
    }
}
