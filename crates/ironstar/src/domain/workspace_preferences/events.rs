//! Domain events for the WorkspacePreferences aggregate.
//!
//! Events represent facts that have occurred. They are immutable, past-tense,
//! and self-describing per Hoffman's Law 1.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{CatalogUri, LayoutDefaults};
use crate::domain::traits::{DeciderType, EventType, Identifier, IsFinal};
use crate::domain::workspace::WorkspaceId;

/// Events emitted by the WorkspacePreferences aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "events/")]
pub enum WorkspacePreferencesEvent {
    /// Preferences were initialized for a workspace.
    WorkspacePreferencesInitialized {
        workspace_id: WorkspaceId,
        initialized_at: DateTime<Utc>,
    },

    /// Default catalog was set.
    DefaultCatalogSet {
        workspace_id: WorkspaceId,
        catalog_uri: CatalogUri,
        set_at: DateTime<Utc>,
    },

    /// Default catalog was cleared.
    DefaultCatalogCleared {
        workspace_id: WorkspaceId,
        cleared_at: DateTime<Utc>,
    },

    /// Layout defaults were updated.
    LayoutDefaultsUpdated {
        workspace_id: WorkspaceId,
        layout_defaults: LayoutDefaults,
        updated_at: DateTime<Utc>,
    },
}

impl WorkspacePreferencesEvent {
    /// Extract the workspace ID this event belongs to.
    #[must_use]
    pub fn workspace_id(&self) -> WorkspaceId {
        match self {
            Self::WorkspacePreferencesInitialized { workspace_id, .. }
            | Self::DefaultCatalogSet { workspace_id, .. }
            | Self::DefaultCatalogCleared { workspace_id, .. }
            | Self::LayoutDefaultsUpdated { workspace_id, .. } => *workspace_id,
        }
    }

    /// Get the event type name for storage and routing.
    #[must_use]
    pub fn event_type_str(&self) -> &'static str {
        match self {
            Self::WorkspacePreferencesInitialized { .. } => "WorkspacePreferencesInitialized",
            Self::DefaultCatalogSet { .. } => "DefaultCatalogSet",
            Self::DefaultCatalogCleared { .. } => "DefaultCatalogCleared",
            Self::LayoutDefaultsUpdated { .. } => "LayoutDefaultsUpdated",
        }
    }

    /// Get the event version for schema evolution.
    #[must_use]
    pub fn event_version(&self) -> &'static str {
        "1"
    }
}

impl Identifier for WorkspacePreferencesEvent {
    fn identifier(&self) -> String {
        format!("workspace_{}/preferences", self.workspace_id())
    }
}

impl EventType for WorkspacePreferencesEvent {
    fn event_type(&self) -> String {
        self.event_type_str().to_string()
    }
}

impl DeciderType for WorkspacePreferencesEvent {
    fn decider_type(&self) -> String {
        "WorkspacePreferences".to_string()
    }
}

impl IsFinal for WorkspacePreferencesEvent {
    fn is_final(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_id() -> WorkspaceId {
        WorkspaceId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn initialized_event_serializes_with_type_tag() {
        let event = WorkspacePreferencesEvent::WorkspacePreferencesInitialized {
            workspace_id: sample_id(),
            initialized_at: sample_time(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "WorkspacePreferencesInitialized");
        assert_eq!(json["workspace_id"], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn event_roundtrips_through_json() {
        let original = WorkspacePreferencesEvent::DefaultCatalogSet {
            workspace_id: sample_id(),
            catalog_uri: CatalogUri::new("ducklake:test").unwrap(),
            set_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: WorkspacePreferencesEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn identifier_follows_aggregate_id_pattern() {
        let event = WorkspacePreferencesEvent::WorkspacePreferencesInitialized {
            workspace_id: sample_id(),
            initialized_at: sample_time(),
        };

        assert_eq!(
            event.identifier(),
            "workspace_00000000-0000-0000-0000-000000000000/preferences"
        );
    }

    #[test]
    fn event_type_matches_serde_tag() {
        let events: Vec<(WorkspacePreferencesEvent, &str)> = vec![
            (
                WorkspacePreferencesEvent::WorkspacePreferencesInitialized {
                    workspace_id: sample_id(),
                    initialized_at: sample_time(),
                },
                "WorkspacePreferencesInitialized",
            ),
            (
                WorkspacePreferencesEvent::DefaultCatalogSet {
                    workspace_id: sample_id(),
                    catalog_uri: CatalogUri::new("test").unwrap(),
                    set_at: sample_time(),
                },
                "DefaultCatalogSet",
            ),
            (
                WorkspacePreferencesEvent::DefaultCatalogCleared {
                    workspace_id: sample_id(),
                    cleared_at: sample_time(),
                },
                "DefaultCatalogCleared",
            ),
            (
                WorkspacePreferencesEvent::LayoutDefaultsUpdated {
                    workspace_id: sample_id(),
                    layout_defaults: LayoutDefaults::default(),
                    updated_at: sample_time(),
                },
                "LayoutDefaultsUpdated",
            ),
        ];

        for (event, expected_type) in events {
            assert_eq!(event.event_type_str(), expected_type);

            let json = serde_json::to_value(&event).unwrap();
            assert_eq!(json["type"], expected_type);
        }
    }

    #[test]
    fn is_final_returns_false_for_all_events() {
        let events = vec![
            WorkspacePreferencesEvent::WorkspacePreferencesInitialized {
                workspace_id: sample_id(),
                initialized_at: sample_time(),
            },
            WorkspacePreferencesEvent::DefaultCatalogCleared {
                workspace_id: sample_id(),
                cleared_at: sample_time(),
            },
        ];

        for event in events {
            assert!(!event.is_final());
        }
    }
}
