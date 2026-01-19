//! Commands for the Workspace aggregate.
//!
//! Commands represent requests to change aggregate state. They are:
//!
//! - **Imperative**: Named for what should happen (Create, Rename, SetVisibility)
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
//! Commands use `String` for the name field, not `WorkspaceName`. This is intentional:
//!
//! 1. Commands carry *user intent*, not validated domain data
//! 2. Validation is the aggregate's responsibility
//! 3. Error messages should come from the aggregate, not deserialization
//! 4. This allows richer error context

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{Visibility, WorkspaceId};
use crate::domain::session::UserId;
use crate::domain::traits::{DeciderType, Identifier};

/// Commands that can be sent to the Workspace aggregate.
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
pub enum WorkspaceCommand {
    /// Create a new workspace.
    ///
    /// The `workspace_id` is generated client-side (UUID v4) to enable optimistic UI
    /// updates. The `name` is raw user input; validation happens in the
    /// decider. Timestamp injected at boundary layer.
    Create {
        /// Client-generated unique identifier.
        workspace_id: WorkspaceId,
        /// Raw name input (will be validated and trimmed).
        name: String,
        /// Owner of the workspace.
        owner_id: UserId,
        /// Visibility setting.
        visibility: Visibility,
        /// When the command was issued (injected at boundary).
        created_at: DateTime<Utc>,
    },

    /// Rename an existing workspace.
    Rename {
        /// Which workspace to rename.
        workspace_id: WorkspaceId,
        /// New name (raw, will be validated).
        new_name: String,
        /// When the rename was issued (injected at boundary).
        renamed_at: DateTime<Utc>,
    },

    /// Change workspace visibility.
    SetVisibility {
        /// Which workspace to modify.
        workspace_id: WorkspaceId,
        /// New visibility setting.
        visibility: Visibility,
        /// When the change was issued (injected at boundary).
        changed_at: DateTime<Utc>,
    },
}

impl WorkspaceCommand {
    /// Extract the target aggregate ID from the command.
    ///
    /// Used by command handlers to load the correct aggregate.
    #[must_use]
    pub fn aggregate_id(&self) -> WorkspaceId {
        match self {
            Self::Create { workspace_id, .. }
            | Self::Rename { workspace_id, .. }
            | Self::SetVisibility { workspace_id, .. } => *workspace_id,
        }
    }

    /// Get the command type name for logging and metrics.
    #[must_use]
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::Create { .. } => "Create",
            Self::Rename { .. } => "Rename",
            Self::SetVisibility { .. } => "SetVisibility",
        }
    }
}

impl Identifier for WorkspaceCommand {
    fn identifier(&self) -> String {
        self.aggregate_id().to_string()
    }
}

impl DeciderType for WorkspaceCommand {
    fn decider_type(&self) -> String {
        "Workspace".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn create_command_serializes_with_type_tag() {
        let cmd = WorkspaceCommand::Create {
            workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
            name: "My Workspace".to_string(),
            owner_id: UserId::from_uuid(uuid::Uuid::nil()),
            visibility: Visibility::Private,
            created_at: sample_time(),
        };

        let json = serde_json::to_value(&cmd).unwrap();

        assert_eq!(json["type"], "Create");
        assert_eq!(json["workspace_id"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(json["name"], "My Workspace");
        assert_eq!(json["visibility"], "private");
        assert!(json["created_at"].is_string());
    }

    #[test]
    fn command_roundtrips_through_json() {
        let original = WorkspaceCommand::Rename {
            workspace_id: WorkspaceId::new(),
            new_name: "New Name".to_string(),
            renamed_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: WorkspaceCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn aggregate_id_extracts_correctly() {
        let id = WorkspaceId::new();
        let ts = sample_time();
        let owner = UserId::new();

        let commands = vec![
            WorkspaceCommand::Create {
                workspace_id: id,
                name: "test".to_string(),
                owner_id: owner,
                visibility: Visibility::Private,
                created_at: ts,
            },
            WorkspaceCommand::Rename {
                workspace_id: id,
                new_name: "updated".to_string(),
                renamed_at: ts,
            },
            WorkspaceCommand::SetVisibility {
                workspace_id: id,
                visibility: Visibility::Public,
                changed_at: ts,
            },
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
            "workspace_id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "  Trim this name  ",
            "owner_id": "660e8400-e29b-41d4-a716-446655440000",
            "visibility": "public",
            "created_at": "2024-01-15T10:30:00Z"
        }"#;

        let cmd: WorkspaceCommand = serde_json::from_str(json).unwrap();

        assert!(matches!(cmd, WorkspaceCommand::Create { name, .. } if name == "  Trim this name  "));
        // Note: name is NOT trimmed here - that's the decider's job
    }
}
