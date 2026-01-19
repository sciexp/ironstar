//! Domain events for the Workspace aggregate.
//!
//! Events represent facts that have occurred in the domain. They are:
//!
//! - **Immutable**: Once emitted, events never change
//! - **Past tense**: Named for what happened, not what should happen
//! - **Self-describing**: Contain all data needed to reconstruct state
//!
//! # Event sourcing semantics
//!
//! Events are the source of truth. Aggregate state is derived by folding
//! events through `evolve`. This means:
//!
//! - Events must contain enough data to reconstruct state
//! - Event schemas evolve via upcasters, not mutations
//! - The event store is append-only
//!
//! # Audit trail
//!
//! Events include old values (`old_name`, `old_visibility`) for audit purposes,
//! enabling reconstruction of historical state without replaying the entire stream.
//!
//! # Serialization
//!
//! Events use internally tagged enums (`#[serde(tag = "type")]`) for:
//!
//! - Self-describing JSON in the event store
//! - Clean TypeScript discriminated unions via ts-rs
//! - Easy pattern matching in frontend code

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{Visibility, WorkspaceId, WorkspaceName};
use crate::domain::session::UserId;
use crate::domain::traits::{DeciderType, EventType, Identifier, IsFinal};

/// Events emitted by the Workspace aggregate.
///
/// Each variant represents a state change that occurred. The aggregate's
/// current state is the result of applying all its events in order.
///
/// # Versioning
///
/// Events have an implicit version (currently v1). Schema evolution is
/// handled by upcasters that transform old event formats during loading.
/// See `EventUpcaster` trait in the infrastructure layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "events/")]
pub enum WorkspaceEvent {
    /// A new workspace was created.
    Created {
        /// Unique identifier for the workspace.
        workspace_id: WorkspaceId,
        /// The workspace's name (validated).
        name: WorkspaceName,
        /// Owner of the workspace.
        owner_id: UserId,
        /// Visibility setting.
        visibility: Visibility,
        /// When the workspace was created.
        created_at: DateTime<Utc>,
    },

    /// The workspace was renamed.
    Renamed {
        /// Which workspace was renamed.
        workspace_id: WorkspaceId,
        /// The previous name (for audit trail).
        old_name: WorkspaceName,
        /// The new name.
        new_name: WorkspaceName,
        /// When the rename occurred.
        renamed_at: DateTime<Utc>,
    },

    /// The workspace visibility was changed.
    VisibilityChanged {
        /// Which workspace was modified.
        workspace_id: WorkspaceId,
        /// The previous visibility (for audit trail).
        old_visibility: Visibility,
        /// The new visibility.
        new_visibility: Visibility,
        /// When the change occurred.
        changed_at: DateTime<Utc>,
    },
}

impl WorkspaceEvent {
    /// Extract the aggregate ID this event belongs to.
    ///
    /// Used by the event store to route events to the correct aggregate.
    #[must_use]
    pub fn aggregate_id(&self) -> WorkspaceId {
        match self {
            Self::Created { workspace_id, .. }
            | Self::Renamed { workspace_id, .. }
            | Self::VisibilityChanged { workspace_id, .. } => *workspace_id,
        }
    }

    /// Get the event type name for storage and routing.
    ///
    /// This matches the serde tag value and is used for:
    /// - Event store `event_type` column
    /// - Upcaster matching
    /// - Metrics and logging
    #[must_use]
    pub fn event_type_str(&self) -> &'static str {
        match self {
            Self::Created { .. } => "Created",
            Self::Renamed { .. } => "Renamed",
            Self::VisibilityChanged { .. } => "VisibilityChanged",
        }
    }

    /// Get the event version for schema evolution.
    ///
    /// All current events are version 1. When schemas evolve, bump this
    /// version and add an upcaster to transform old events.
    #[must_use]
    pub fn event_version(&self) -> &'static str {
        "1"
    }
}

impl Identifier for WorkspaceEvent {
    fn identifier(&self) -> String {
        self.aggregate_id().to_string()
    }
}

impl EventType for WorkspaceEvent {
    fn event_type(&self) -> String {
        self.event_type_str().to_string()
    }
}

impl DeciderType for WorkspaceEvent {
    fn decider_type(&self) -> String {
        "Workspace".to_string()
    }
}

impl IsFinal for WorkspaceEvent {
    fn is_final(&self) -> bool {
        // Workspace has no terminal state currently (no Delete event)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_id() -> WorkspaceId {
        WorkspaceId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_owner() -> UserId {
        UserId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_name() -> WorkspaceName {
        WorkspaceName::new("My Workspace").unwrap()
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn created_event_serializes_with_type_tag() {
        let event = WorkspaceEvent::Created {
            workspace_id: sample_id(),
            name: sample_name(),
            owner_id: sample_owner(),
            visibility: Visibility::Private,
            created_at: sample_time(),
        };

        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["type"], "Created");
        assert_eq!(json["workspace_id"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(json["name"], "My Workspace");
        assert_eq!(json["visibility"], "private");
        assert!(json["created_at"].is_string());
    }

    #[test]
    fn event_roundtrips_through_json() {
        let original = WorkspaceEvent::Renamed {
            workspace_id: sample_id(),
            old_name: sample_name(),
            new_name: WorkspaceName::new("New Name").unwrap(),
            renamed_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: WorkspaceEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn aggregate_id_extracts_correctly() {
        let id = WorkspaceId::new();
        let event = WorkspaceEvent::VisibilityChanged {
            workspace_id: id,
            old_visibility: Visibility::Private,
            new_visibility: Visibility::Public,
            changed_at: sample_time(),
        };

        assert_eq!(event.aggregate_id(), id);
    }

    #[test]
    fn event_type_matches_serde_tag() {
        let events = vec![
            (
                WorkspaceEvent::Created {
                    workspace_id: sample_id(),
                    name: sample_name(),
                    owner_id: sample_owner(),
                    visibility: Visibility::Private,
                    created_at: sample_time(),
                },
                "Created",
            ),
            (
                WorkspaceEvent::Renamed {
                    workspace_id: sample_id(),
                    old_name: sample_name(),
                    new_name: WorkspaceName::new("New").unwrap(),
                    renamed_at: sample_time(),
                },
                "Renamed",
            ),
            (
                WorkspaceEvent::VisibilityChanged {
                    workspace_id: sample_id(),
                    old_visibility: Visibility::Private,
                    new_visibility: Visibility::Public,
                    changed_at: sample_time(),
                },
                "VisibilityChanged",
            ),
        ];

        for (event, expected_type) in events {
            assert_eq!(event.event_type_str(), expected_type);

            // Verify serde tag matches
            let json = serde_json::to_value(&event).unwrap();
            assert_eq!(json["type"], expected_type);
        }
    }

    #[test]
    fn is_final_returns_false_for_all_events() {
        let events = vec![
            WorkspaceEvent::Created {
                workspace_id: sample_id(),
                name: sample_name(),
                owner_id: sample_owner(),
                visibility: Visibility::Private,
                created_at: sample_time(),
            },
            WorkspaceEvent::Renamed {
                workspace_id: sample_id(),
                old_name: sample_name(),
                new_name: WorkspaceName::new("New").unwrap(),
                renamed_at: sample_time(),
            },
            WorkspaceEvent::VisibilityChanged {
                workspace_id: sample_id(),
                old_visibility: Visibility::Private,
                new_visibility: Visibility::Public,
                changed_at: sample_time(),
            },
        ];

        for event in events {
            assert!(!event.is_final());
        }
    }
}
