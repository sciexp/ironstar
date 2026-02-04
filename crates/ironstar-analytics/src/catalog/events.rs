//! Events emitted by the Catalog aggregate.

use chrono::{DateTime, Utc};
use ironstar_core::{DeciderType, EventType, Identifier, IsFinal};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{CatalogMetadata, CatalogRef};

/// Events emitted by the Catalog aggregate.
///
/// Events represent facts that have occurred. They are immutable records
/// of state changes, persisted to the event store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(tag = "type")]
pub enum CatalogEvent {
    /// A catalog was selected and is now active.
    CatalogSelected {
        catalog_ref: CatalogRef,
        selected_at: DateTime<Utc>,
    },

    /// Catalog metadata was refreshed with new dataset information.
    CatalogMetadataRefreshed {
        metadata: CatalogMetadata,
        refreshed_at: DateTime<Utc>,
    },
}

impl Identifier for CatalogEvent {
    fn identifier(&self) -> String {
        // Singleton aggregate pattern
        "default-catalog".to_string()
    }
}

impl EventType for CatalogEvent {
    fn event_type(&self) -> String {
        match self {
            Self::CatalogSelected { .. } => "CatalogSelected",
            Self::CatalogMetadataRefreshed { .. } => "CatalogMetadataRefreshed",
        }
        .to_string()
    }
}

impl DeciderType for CatalogEvent {
    fn decider_type(&self) -> String {
        "Catalog".to_string()
    }
}

impl IsFinal for CatalogEvent {
    fn is_final(&self) -> bool {
        false // Catalog can always be updated, never terminal
    }
}
