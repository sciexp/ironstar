//! Catalog View for read-side projections.
//!
//! The View materializes Catalog events into queryable state tracking the
//! currently active catalog and its metadata. Unlike the Decider, it has
//! no command handling — only event folding.

use fmodel_rust::view::View;

use crate::domain::{CatalogEvent, CatalogMetadata, CatalogRef};

/// State materialized by the Catalog View.
///
/// Tracks the currently selected catalog reference and its metadata.
/// When no catalog is selected, both fields are `None`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CatalogViewState {
    pub catalog_ref: Option<CatalogRef>,
    pub metadata: Option<CatalogMetadata>,
}

impl CatalogViewState {
    /// Whether a catalog is currently selected.
    #[must_use]
    pub fn has_catalog(&self) -> bool {
        self.catalog_ref.is_some()
    }

    /// Whether metadata has been loaded for the current catalog.
    #[must_use]
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }

    /// Number of datasets in the current catalog metadata, or 0 if none.
    #[must_use]
    pub fn dataset_count(&self) -> usize {
        self.metadata
            .as_ref()
            .map_or(0, |m| m.datasets.len())
    }
}

/// Type alias for the Catalog View.
pub type CatalogView<'a> = View<'a, CatalogViewState, CatalogEvent>;

/// Factory function creating a pure Catalog View.
pub fn catalog_view<'a>() -> CatalogView<'a> {
    View {
        evolve: Box::new(evolve),
        initial_state: Box::new(CatalogViewState::default),
    }
}

/// Pure evolve function: (State, Event) -> State
///
/// `CatalogSelected` sets the catalog ref and clears metadata (new catalog
/// starts with no metadata until a refresh). `CatalogMetadataRefreshed`
/// updates the metadata for the current catalog.
fn evolve(state: &CatalogViewState, event: &CatalogEvent) -> CatalogViewState {
    match event {
        CatalogEvent::CatalogSelected { catalog_ref, .. } => CatalogViewState {
            catalog_ref: Some(catalog_ref.clone()),
            metadata: None,
        },
        CatalogEvent::CatalogMetadataRefreshed { metadata, .. } => CatalogViewState {
            catalog_ref: state.catalog_ref.clone(),
            metadata: Some(metadata.clone()),
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use fmodel_rust::view::ViewStateComputation;

    use super::*;
    use crate::domain::DatasetInfo;

    fn as_refs(events: &[CatalogEvent]) -> Vec<&CatalogEvent> {
        events.iter().collect()
    }

    fn sample_catalog_ref() -> CatalogRef {
        CatalogRef::new("ducklake:my_catalog").unwrap()
    }

    fn sample_metadata() -> CatalogMetadata {
        CatalogMetadata {
            datasets: vec![
                DatasetInfo {
                    name: "genomics".to_string(),
                    table_count: 5,
                    schema_version: "1.0".to_string(),
                },
                DatasetInfo {
                    name: "proteomics".to_string(),
                    table_count: 3,
                    schema_version: "2.1".to_string(),
                },
            ],
            last_refreshed: Utc::now(),
        }
    }

    #[test]
    fn initial_state_has_no_catalog() {
        let view = catalog_view();
        let state = (view.initial_state)();
        assert!(!state.has_catalog());
        assert!(!state.has_metadata());
        assert_eq!(state.dataset_count(), 0);
    }

    #[test]
    fn catalog_selected_sets_ref_with_no_metadata() {
        let view = catalog_view();
        let catalog_ref = sample_catalog_ref();
        let events = vec![CatalogEvent::CatalogSelected {
            catalog_ref: catalog_ref.clone(),
            selected_at: Utc::now(),
        }];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(state.has_catalog());
        assert_eq!(state.catalog_ref.as_ref(), Some(&catalog_ref));
        assert!(!state.has_metadata());
    }

    #[test]
    fn metadata_refresh_populates_metadata() {
        let view = catalog_view();
        let catalog_ref = sample_catalog_ref();
        let metadata = sample_metadata();

        let events = vec![
            CatalogEvent::CatalogSelected {
                catalog_ref: catalog_ref.clone(),
                selected_at: Utc::now(),
            },
            CatalogEvent::CatalogMetadataRefreshed {
                metadata: metadata.clone(),
                refreshed_at: Utc::now(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(state.has_catalog());
        assert!(state.has_metadata());
        assert_eq!(state.dataset_count(), 2);
        assert_eq!(state.metadata.as_ref(), Some(&metadata));
    }

    #[test]
    fn selecting_new_catalog_clears_metadata() {
        let view = catalog_view();
        let first_ref = CatalogRef::new("ducklake:first").unwrap();
        let second_ref = CatalogRef::new("ducklake:second").unwrap();
        let metadata = sample_metadata();

        let events = vec![
            CatalogEvent::CatalogSelected {
                catalog_ref: first_ref,
                selected_at: Utc::now(),
            },
            CatalogEvent::CatalogMetadataRefreshed {
                metadata,
                refreshed_at: Utc::now(),
            },
            CatalogEvent::CatalogSelected {
                catalog_ref: second_ref.clone(),
                selected_at: Utc::now(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert_eq!(state.catalog_ref.as_ref(), Some(&second_ref));
        assert!(!state.has_metadata());
        assert_eq!(state.dataset_count(), 0);
    }

    #[test]
    fn metadata_refresh_without_selection_preserves_none_ref() {
        let view = catalog_view();
        let metadata = sample_metadata();

        let events = vec![CatalogEvent::CatalogMetadataRefreshed {
            metadata: metadata.clone(),
            refreshed_at: Utc::now(),
        }];

        let state = view.compute_new_state(None, &as_refs(&events));

        // View is infallible — it records the metadata even without a catalog ref.
        // The Decider prevents this sequence, but the View remains total.
        assert!(!state.has_catalog());
        assert!(state.has_metadata());
        assert_eq!(state.metadata.as_ref(), Some(&metadata));
    }

    #[test]
    fn compute_new_state_from_existing_state() {
        let view = catalog_view();
        let catalog_ref = sample_catalog_ref();
        let metadata = sample_metadata();

        let existing = CatalogViewState {
            catalog_ref: Some(catalog_ref.clone()),
            metadata: None,
        };

        let new_events = vec![CatalogEvent::CatalogMetadataRefreshed {
            metadata: metadata.clone(),
            refreshed_at: Utc::now(),
        }];

        let state = view.compute_new_state(Some(existing), &as_refs(&new_events));

        assert_eq!(state.catalog_ref.as_ref(), Some(&catalog_ref));
        assert_eq!(state.metadata.as_ref(), Some(&metadata));
    }
}
