//! Commands for the Todo aggregate.
//!
//! Commands represent requests to change aggregate state. They are:
//!
//! - **Imperative**: Named for what should happen (Create, Complete, Delete)
//! - **Validatable**: The aggregate may accept or reject them
//! - **Carry raw input**: Validation happens in the aggregate, not here
//!
//! # Command vs Event
//!
//! | Aspect | Command | Event |
//! |--------|---------|-------|
//! | Tense | Imperative (Create) | Past (Created) |
//! | Outcome | May fail | Already happened |
//! | Data | Raw user input | Validated domain types |
//! | Source | External (user, API) | Internal (aggregate) |
//!
//! # Design choice: raw strings in commands
//!
//! Commands use `String` for text fields, not `TodoText`. This is intentional:
//!
//! 1. Commands carry *user intent*, not validated domain data
//! 2. Validation is the aggregate's responsibility
//! 3. Error messages should come from the aggregate, not deserialization
//! 4. This allows richer error context (e.g., "text too long by 15 chars")
//!
//! The aggregate's `handle_command` validates and converts to `TodoText`,
//! then emits events containing the validated types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::TodoId;

/// Commands that can be sent to the Todo aggregate.
///
/// Each command represents a user intention. The aggregate validates
/// the command against its current state and either:
///
/// - Emits events (success)
/// - Returns an error (validation failure)
///
/// Commands are deserialized from HTTP requests (via datastar signals)
/// and routed to the appropriate aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "commands/")]
pub enum TodoCommand {
    /// Create a new todo item.
    ///
    /// The `id` is generated client-side (UUID v4) to enable optimistic UI
    /// updates. The `text` is raw user input; validation happens in the
    /// aggregate.
    Create {
        /// Client-generated unique identifier.
        id: TodoId,
        /// Raw text input (will be validated and trimmed).
        text: String,
    },

    /// Update the text of an existing todo.
    UpdateText {
        /// Which todo to update.
        id: TodoId,
        /// New text content (raw, will be validated).
        text: String,
    },

    /// Mark a todo as completed.
    Complete {
        /// Which todo to complete.
        id: TodoId,
    },

    /// Mark a todo as not completed (undo completion).
    Uncomplete {
        /// Which todo to uncomplete.
        id: TodoId,
    },

    /// Delete a todo item.
    Delete {
        /// Which todo to delete.
        id: TodoId,
    },
}

impl TodoCommand {
    /// Extract the target aggregate ID from the command.
    ///
    /// Used by command handlers to load the correct aggregate.
    #[must_use]
    pub fn aggregate_id(&self) -> TodoId {
        match self {
            Self::Create { id, .. }
            | Self::UpdateText { id, .. }
            | Self::Complete { id }
            | Self::Uncomplete { id }
            | Self::Delete { id } => *id,
        }
    }

    /// Get the command type name for logging and metrics.
    #[must_use]
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::Create { .. } => "Create",
            Self::UpdateText { .. } => "UpdateText",
            Self::Complete { .. } => "Complete",
            Self::Uncomplete { .. } => "Uncomplete",
            Self::Delete { .. } => "Delete",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_command_serializes_with_type_tag() {
        let cmd = TodoCommand::Create {
            id: TodoId::from_uuid(uuid::Uuid::nil()),
            text: "Buy groceries".to_string(),
        };

        let json = serde_json::to_value(&cmd).unwrap();

        assert_eq!(json["type"], "Create");
        assert_eq!(json["id"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(json["text"], "Buy groceries");
    }

    #[test]
    fn command_roundtrips_through_json() {
        let original = TodoCommand::Complete { id: TodoId::new() };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: TodoCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn aggregate_id_extracts_correctly() {
        let id = TodoId::new();

        let commands = vec![
            TodoCommand::Create {
                id,
                text: "test".to_string(),
            },
            TodoCommand::UpdateText {
                id,
                text: "updated".to_string(),
            },
            TodoCommand::Complete { id },
            TodoCommand::Uncomplete { id },
            TodoCommand::Delete { id },
        ];

        for cmd in commands {
            assert_eq!(cmd.aggregate_id(), id);
        }
    }

    #[test]
    fn deserializes_from_datastar_signal_format() {
        // This is how datastar sends commands via ReadSignals
        let json = r#"{
            "type": "Create",
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "text": "  Trim this text  "
        }"#;

        let cmd: TodoCommand = serde_json::from_str(json).unwrap();

        assert!(matches!(cmd, TodoCommand::Create { text, .. } if text == "  Trim this text  "));
        // Note: text is NOT trimmed here - that's the aggregate's job
    }
}
