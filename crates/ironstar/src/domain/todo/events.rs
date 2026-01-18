//! Domain events for the Todo aggregate.
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
//! events through `apply_event`. This means:
//!
//! - Events must contain enough data to reconstruct state
//! - Event schemas evolve via upcasters, not mutations
//! - The event store is append-only
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

use super::values::{TodoId, TodoText};
use crate::domain::traits::{DeciderType, EventType, Identifier, IsFinal};

/// Events emitted by the Todo aggregate.
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
pub enum TodoEvent {
    /// A new todo was created.
    Created {
        /// Unique identifier for the todo.
        id: TodoId,
        /// The todo's text content (validated).
        text: TodoText,
        /// When the todo was created.
        created_at: DateTime<Utc>,
    },

    /// The todo's text was updated.
    TextUpdated {
        /// Which todo was updated.
        id: TodoId,
        /// The new text content.
        text: TodoText,
        /// When the update occurred.
        updated_at: DateTime<Utc>,
    },

    /// The todo was marked as completed.
    Completed {
        /// Which todo was completed.
        id: TodoId,
        /// When it was completed.
        completed_at: DateTime<Utc>,
    },

    /// The todo was marked as not completed (uncompleted).
    Uncompleted {
        /// Which todo was uncompleted.
        id: TodoId,
        /// When it was uncompleted.
        uncompleted_at: DateTime<Utc>,
    },

    /// The todo was deleted.
    Deleted {
        /// Which todo was deleted.
        id: TodoId,
        /// When it was deleted.
        deleted_at: DateTime<Utc>,
    },
}

impl TodoEvent {
    /// Extract the aggregate ID this event belongs to.
    ///
    /// Used by the event store to route events to the correct aggregate.
    #[must_use]
    pub fn aggregate_id(&self) -> TodoId {
        match self {
            Self::Created { id, .. }
            | Self::TextUpdated { id, .. }
            | Self::Completed { id, .. }
            | Self::Uncompleted { id, .. }
            | Self::Deleted { id, .. } => *id,
        }
    }

    /// Get the event type name for storage and routing.
    ///
    /// This matches the serde tag value and is used for:
    /// - Event store `event_type` column
    /// - Upcaster matching
    /// - Metrics and logging
    #[must_use]
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Created { .. } => "Created",
            Self::TextUpdated { .. } => "TextUpdated",
            Self::Completed { .. } => "Completed",
            Self::Uncompleted { .. } => "Uncompleted",
            Self::Deleted { .. } => "Deleted",
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

impl Identifier for TodoEvent {
    fn identifier(&self) -> String {
        self.aggregate_id().to_string()
    }
}

impl EventType for TodoEvent {
    fn event_type(&self) -> String {
        match self {
            Self::Created { .. } => "Created",
            Self::TextUpdated { .. } => "TextUpdated",
            Self::Completed { .. } => "Completed",
            Self::Uncompleted { .. } => "Uncompleted",
            Self::Deleted { .. } => "Deleted",
        }
        .to_string()
    }
}

impl DeciderType for TodoEvent {
    fn decider_type(&self) -> String {
        "Todo".to_string()
    }
}

impl IsFinal for TodoEvent {
    fn is_final(&self) -> bool {
        matches!(self, Self::Deleted { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_id() -> TodoId {
        TodoId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_text() -> TodoText {
        TodoText::new("Buy groceries").unwrap()
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn created_event_serializes_with_type_tag() {
        let event = TodoEvent::Created {
            id: sample_id(),
            text: sample_text(),
            created_at: sample_time(),
        };

        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["type"], "Created");
        assert_eq!(json["id"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(json["text"], "Buy groceries");
        assert!(json["created_at"].is_string());
    }

    #[test]
    fn event_roundtrips_through_json() {
        let original = TodoEvent::Completed {
            id: sample_id(),
            completed_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: TodoEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn aggregate_id_extracts_correctly() {
        let id = TodoId::new();
        let event = TodoEvent::TextUpdated {
            id,
            text: sample_text(),
            updated_at: sample_time(),
        };

        assert_eq!(event.aggregate_id(), id);
    }

    #[test]
    fn event_type_matches_serde_tag() {
        let events = vec![
            (
                TodoEvent::Created {
                    id: sample_id(),
                    text: sample_text(),
                    created_at: sample_time(),
                },
                "Created",
            ),
            (
                TodoEvent::Completed {
                    id: sample_id(),
                    completed_at: sample_time(),
                },
                "Completed",
            ),
            (
                TodoEvent::Deleted {
                    id: sample_id(),
                    deleted_at: sample_time(),
                },
                "Deleted",
            ),
        ];

        for (event, expected_type) in events {
            assert_eq!(event.event_type(), expected_type);

            // Verify serde tag matches
            let json = serde_json::to_value(&event).unwrap();
            assert_eq!(json["type"], expected_type);
        }
    }
}
