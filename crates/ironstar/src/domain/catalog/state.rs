//! Catalog aggregate state types.
//!
//! State is derived from events and represents the current catalog selection.
//! The Catalog is a singleton aggregate with two states: no catalog selected
//! or a specific catalog active with its metadata.

use super::values::{CatalogMetadata, CatalogRef};

/// Catalog aggregate state machine.
///
/// ```text
///   NoCatalogSelected ──SelectCatalog──► CatalogActive
///                                           │
///                                    RefreshMetadata
///                                           │
///                                           ▼
///                                      CatalogActive (updated metadata)
/// ```
///
/// Invariant: only one catalog can be active at a time.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CatalogState {
    /// No catalog has been selected yet (initial state).
    #[default]
    NoCatalogSelected,
    /// A catalog is active with its metadata.
    CatalogActive {
        catalog_ref: CatalogRef,
        metadata: CatalogMetadata,
    },
}

impl CatalogState {
    /// Check if no catalog is currently selected.
    #[must_use]
    pub fn is_no_catalog_selected(&self) -> bool {
        matches!(self, Self::NoCatalogSelected)
    }

    /// Check if a catalog is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::CatalogActive { .. })
    }

    /// Get the active catalog reference, if any.
    #[must_use]
    pub fn catalog_ref(&self) -> Option<&CatalogRef> {
        match self {
            Self::CatalogActive { catalog_ref, .. } => Some(catalog_ref),
            Self::NoCatalogSelected => None,
        }
    }

    /// Get the active catalog metadata, if any.
    #[must_use]
    pub fn metadata(&self) -> Option<&CatalogMetadata> {
        match self {
            Self::CatalogActive { metadata, .. } => Some(metadata),
            Self::NoCatalogSelected => None,
        }
    }
}
