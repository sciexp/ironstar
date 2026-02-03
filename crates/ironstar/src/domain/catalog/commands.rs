//! Commands for the Catalog aggregate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{CatalogMetadata, CatalogRef};
use crate::domain::traits::{DeciderType, Identifier};

/// Commands for the Catalog aggregate.
///
/// Commands represent requests to change state. All commands include
/// timestamp fields for pure decision-making. Timestamps are injected
/// by the application layer; the decider never calls `Utc::now()`.
///
/// For `RefreshCatalogMetadata`, the application layer introspects the
/// DuckLake catalog via DuckDB and provides the metadata in the command.
/// The typed holes from the Idris spec (`?metadata`, `?timestamp_refresh`)
/// resolve at the effect boundary, not in the Decider.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(tag = "type")]
pub enum CatalogCommand {
    /// Select a DuckLake catalog. Transitions NoCatalogSelected → CatalogActive.
    SelectCatalog {
        /// Reference to the catalog to select.
        catalog_ref: CatalogRef,
        /// Timestamp when the catalog was selected (injected by application layer).
        selected_at: DateTime<Utc>,
    },

    /// Refresh the metadata for the active catalog.
    /// Only valid when a catalog is active.
    RefreshCatalogMetadata {
        /// Fresh metadata from DuckDB catalog introspection.
        metadata: CatalogMetadata,
        /// Timestamp when the refresh was performed (injected by application layer).
        refreshed_at: DateTime<Utc>,
    },
}

impl Identifier for CatalogCommand {
    fn identifier(&self) -> String {
        // Singleton aggregate pattern - all commands target the same catalog
        "default-catalog".to_string()
    }
}

impl DeciderType for CatalogCommand {
    fn decider_type(&self) -> String {
        "Catalog".to_string()
    }
}
