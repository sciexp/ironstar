//! Pure Catalog Decider implementing fmodel-rust patterns.
//!
//! The Decider manages the lifecycle of DuckLake catalog selections.
//! It is a pure function with no side effects.

use ironstar_core::Decider;
use tracing::instrument;

use super::commands::CatalogCommand;
use super::errors::CatalogError;
use super::events::CatalogEvent;
use super::state::CatalogState;
use super::values::CatalogMetadata;

/// Type alias for the Catalog Decider.
///
/// Uses `CatalogState` directly (not `Option<CatalogState>`) since the
/// initial state is a concrete value (`NoCatalogSelected`).
pub type CatalogDecider<'a> = Decider<'a, CatalogCommand, CatalogState, CatalogEvent, CatalogError>;

/// Factory function creating a pure Catalog Decider.
pub fn catalog_decider<'a>() -> CatalogDecider<'a> {
    Decider {
        decide: Box::new(decide),
        evolve: Box::new(evolve),
        initial_state: Box::new(CatalogState::default),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
#[instrument(
    name = "decider.catalog.decide",
    skip_all,
    fields(
        command_type = command.command_type(),
        aggregate_type = "Catalog",
    )
)]
fn decide(
    command: &CatalogCommand,
    state: &CatalogState,
) -> Result<Vec<CatalogEvent>, CatalogError> {
    let result = match (command, state) {
        // SelectCatalog from NoCatalogSelected -> emit CatalogSelected
        (
            CatalogCommand::SelectCatalog {
                catalog_ref,
                selected_at,
            },
            CatalogState::NoCatalogSelected,
        ) => Ok(vec![CatalogEvent::CatalogSelected {
            catalog_ref: catalog_ref.clone(),
            selected_at: *selected_at,
        }]),

        // SelectCatalog from CatalogActive with same ref -> idempotent
        (
            CatalogCommand::SelectCatalog { catalog_ref, .. },
            CatalogState::CatalogActive {
                catalog_ref: active_ref,
                ..
            },
        ) if catalog_ref == active_ref => Ok(vec![]),

        // SelectCatalog from CatalogActive with different ref -> error
        (CatalogCommand::SelectCatalog { .. }, CatalogState::CatalogActive { .. }) => {
            Err(CatalogError::catalog_already_active())
        }

        // RefreshCatalogMetadata from NoCatalogSelected -> error
        (CatalogCommand::RefreshCatalogMetadata { .. }, CatalogState::NoCatalogSelected) => {
            Err(CatalogError::no_catalog_selected())
        }

        // RefreshCatalogMetadata from CatalogActive -> emit refresh
        (
            CatalogCommand::RefreshCatalogMetadata {
                metadata,
                refreshed_at,
            },
            CatalogState::CatalogActive { .. },
        ) => Ok(vec![CatalogEvent::CatalogMetadataRefreshed {
            metadata: metadata.clone(),
            refreshed_at: *refreshed_at,
        }]),
    };
    if let Ok(ref events) = result {
        tracing::debug!(event_count = events.len(), "decision complete");
    }
    result
}

/// Pure evolve function: (State, Event) -> State
#[instrument(
    name = "decider.catalog.evolve",
    level = "trace",
    skip_all,
    fields(aggregate_type = "Catalog")
)]
fn evolve(state: &CatalogState, event: &CatalogEvent) -> CatalogState {
    match event {
        CatalogEvent::CatalogSelected {
            catalog_ref,
            selected_at,
        } => CatalogState::CatalogActive {
            catalog_ref: catalog_ref.clone(),
            metadata: CatalogMetadata {
                datasets: vec![],
                last_refreshed: *selected_at,
            },
        },

        CatalogEvent::CatalogMetadataRefreshed { metadata, .. } => match state {
            CatalogState::NoCatalogSelected => state.clone(), // Defensive: should not happen
            CatalogState::CatalogActive { catalog_ref, .. } => CatalogState::CatalogActive {
                catalog_ref: catalog_ref.clone(),
                metadata: metadata.clone(),
            },
        },
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::super::values::{CatalogRef, DatasetInfo};
    use super::*;
    use chrono::Utc;
    use ironstar_core::DeciderTestSpecification;

    fn sample_ref() -> CatalogRef {
        CatalogRef::new("ducklake://hf/sciexp/fixtures").expect("valid ref")
    }

    fn other_ref() -> CatalogRef {
        CatalogRef::new("ducklake://hf/sciexp/other").expect("valid ref")
    }

    fn sample_time() -> chrono::DateTime<Utc> {
        Utc::now()
    }

    fn sample_metadata(ts: chrono::DateTime<Utc>) -> CatalogMetadata {
        CatalogMetadata {
            datasets: vec![DatasetInfo {
                name: "genomics".to_string(),
                table_count: 5,
                schema_version: "1.0.0".to_string(),
            }],
            last_refreshed: ts,
        }
    }

    // ===== SelectCatalog tests =====

    #[test]
    fn select_catalog_from_no_catalog_succeeds() {
        let r = sample_ref();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(catalog_decider())
            .given(vec![])
            .when(CatalogCommand::SelectCatalog {
                catalog_ref: r.clone(),
                selected_at: ts,
            })
            .then(vec![CatalogEvent::CatalogSelected {
                catalog_ref: r,
                selected_at: ts,
            }]);
    }

    #[test]
    fn select_same_catalog_is_idempotent() {
        let r = sample_ref();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(catalog_decider())
            .given(vec![CatalogEvent::CatalogSelected {
                catalog_ref: r.clone(),
                selected_at: ts,
            }])
            .when(CatalogCommand::SelectCatalog {
                catalog_ref: r,
                selected_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn select_different_catalog_fails() {
        let r = sample_ref();
        let other = other_ref();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(catalog_decider())
            .given(vec![CatalogEvent::CatalogSelected {
                catalog_ref: r,
                selected_at: ts,
            }])
            .when(CatalogCommand::SelectCatalog {
                catalog_ref: other,
                selected_at: ts,
            })
            .then_error(CatalogError::catalog_already_active());
    }

    // ===== RefreshCatalogMetadata tests =====

    #[test]
    fn refresh_with_no_catalog_fails() {
        let ts = sample_time();
        let meta = sample_metadata(ts);

        DeciderTestSpecification::default()
            .for_decider(catalog_decider())
            .given(vec![])
            .when(CatalogCommand::RefreshCatalogMetadata {
                metadata: meta,
                refreshed_at: ts,
            })
            .then_error(CatalogError::no_catalog_selected());
    }

    #[test]
    fn refresh_with_active_catalog_succeeds() {
        let r = sample_ref();
        let ts = sample_time();
        let meta = sample_metadata(ts);

        DeciderTestSpecification::default()
            .for_decider(catalog_decider())
            .given(vec![CatalogEvent::CatalogSelected {
                catalog_ref: r,
                selected_at: ts,
            }])
            .when(CatalogCommand::RefreshCatalogMetadata {
                metadata: meta.clone(),
                refreshed_at: ts,
            })
            .then(vec![CatalogEvent::CatalogMetadataRefreshed {
                metadata: meta,
                refreshed_at: ts,
            }]);
    }

    // ===== Evolve tests =====

    #[test]
    fn evolve_catalog_selected_creates_active_state() {
        let r = sample_ref();
        let ts = sample_time();

        let state = evolve(
            &CatalogState::NoCatalogSelected,
            &CatalogEvent::CatalogSelected {
                catalog_ref: r.clone(),
                selected_at: ts,
            },
        );

        assert!(state.is_active());
        assert_eq!(state.catalog_ref(), Some(&r));
        assert!(state.metadata().expect("has metadata").datasets.is_empty());
    }

    #[test]
    fn evolve_metadata_refreshed_updates_metadata() {
        let r = sample_ref();
        let ts = sample_time();
        let meta = sample_metadata(ts);

        let initial = CatalogState::CatalogActive {
            catalog_ref: r.clone(),
            metadata: CatalogMetadata {
                datasets: vec![],
                last_refreshed: ts,
            },
        };

        let state = evolve(
            &initial,
            &CatalogEvent::CatalogMetadataRefreshed {
                metadata: meta.clone(),
                refreshed_at: ts,
            },
        );

        assert!(state.is_active());
        assert_eq!(state.catalog_ref(), Some(&r));
        let actual_meta = state.metadata().expect("has metadata");
        assert_eq!(actual_meta.datasets.len(), 1);
        assert_eq!(actual_meta.datasets[0].name, "genomics");
    }

    #[test]
    fn evolve_metadata_refreshed_from_no_catalog_is_defensive() {
        let ts = sample_time();
        let meta = sample_metadata(ts);

        let state = evolve(
            &CatalogState::NoCatalogSelected,
            &CatalogEvent::CatalogMetadataRefreshed {
                metadata: meta,
                refreshed_at: ts,
            },
        );

        assert!(state.is_no_catalog_selected());
    }

    // ===== Full lifecycle test =====

    #[test]
    fn full_lifecycle_select_then_refresh() {
        let r = sample_ref();
        let ts = sample_time();
        let meta = sample_metadata(ts);

        DeciderTestSpecification::default()
            .for_decider(catalog_decider())
            .given(vec![CatalogEvent::CatalogSelected {
                catalog_ref: r.clone(),
                selected_at: ts,
            }])
            .when(CatalogCommand::RefreshCatalogMetadata {
                metadata: meta.clone(),
                refreshed_at: ts,
            })
            .then(vec![CatalogEvent::CatalogMetadataRefreshed {
                metadata: meta,
                refreshed_at: ts,
            }]);
    }

    #[test]
    fn idempotent_select_after_refresh_preserves_metadata() {
        let r = sample_ref();
        let ts = sample_time();
        let meta = sample_metadata(ts);

        DeciderTestSpecification::default()
            .for_decider(catalog_decider())
            .given(vec![
                CatalogEvent::CatalogSelected {
                    catalog_ref: r.clone(),
                    selected_at: ts,
                },
                CatalogEvent::CatalogMetadataRefreshed {
                    metadata: meta,
                    refreshed_at: ts,
                },
            ])
            .when(CatalogCommand::SelectCatalog {
                catalog_ref: r,
                selected_at: ts,
            })
            .then(vec![]);
    }
}
