//! Commands for the WorkspacePreferences aggregate.
//!
//! Commands represent requests to change workspace preferences state.
//! Timestamps are injected at the boundary layer; the pure decider
//! does not call `Utc::now()`.
//!
//! # Design: raw strings vs value objects
//!
//! Commands carry `CatalogUri` (not raw `String`) because catalog URI
//! validation is structural (non-empty, max length) and can fail early.
//! `LayoutDefaults` is accepted as-is since JSON validation is deferred
//! to the boundary.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{CatalogUri, LayoutDefaults};
use crate::domain::traits::{DeciderType, Identifier};
use crate::domain::workspace::WorkspaceId;

/// Commands that can be sent to the WorkspacePreferences aggregate.
///
/// The aggregate ID follows the pattern `workspace_{workspace_id}/preferences`,
/// making this a per-workspace singleton.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "commands/")]
pub enum WorkspacePreferencesCommand {
    /// Initialize preferences for a workspace.
    ///
    /// Can only succeed when preferences do not yet exist.
    InitializeWorkspacePreferences {
        workspace_id: WorkspaceId,
        initialized_at: DateTime<Utc>,
    },

    /// Set the default DuckDB catalog for this workspace.
    ///
    /// Requires preferences to be initialized. Idempotent when
    /// setting the same catalog URI.
    SetDefaultCatalog {
        workspace_id: WorkspaceId,
        catalog_uri: CatalogUri,
        set_at: DateTime<Utc>,
    },

    /// Clear the default catalog selection.
    ///
    /// Requires preferences to be initialized. Idempotent when
    /// no catalog is currently set.
    ClearDefaultCatalog {
        workspace_id: WorkspaceId,
        cleared_at: DateTime<Utc>,
    },

    /// Update layout defaults for this workspace.
    ///
    /// Requires preferences to be initialized. Idempotent when
    /// setting the same layout defaults.
    UpdateLayoutDefaults {
        workspace_id: WorkspaceId,
        layout_defaults: LayoutDefaults,
        updated_at: DateTime<Utc>,
    },
}

impl WorkspacePreferencesCommand {
    /// Extract the workspace ID from the command.
    #[must_use]
    pub fn workspace_id(&self) -> WorkspaceId {
        match self {
            Self::InitializeWorkspacePreferences { workspace_id, .. }
            | Self::SetDefaultCatalog { workspace_id, .. }
            | Self::ClearDefaultCatalog { workspace_id, .. }
            | Self::UpdateLayoutDefaults { workspace_id, .. } => *workspace_id,
        }
    }

    /// Get the command type name for logging and metrics.
    #[must_use]
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::InitializeWorkspacePreferences { .. } => "InitializeWorkspacePreferences",
            Self::SetDefaultCatalog { .. } => "SetDefaultCatalog",
            Self::ClearDefaultCatalog { .. } => "ClearDefaultCatalog",
            Self::UpdateLayoutDefaults { .. } => "UpdateLayoutDefaults",
        }
    }
}

impl Identifier for WorkspacePreferencesCommand {
    fn identifier(&self) -> String {
        format!("workspace_{}/preferences", self.workspace_id())
    }
}

impl DeciderType for WorkspacePreferencesCommand {
    fn decider_type(&self) -> String {
        "WorkspacePreferences".to_string()
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
    fn initialize_command_serializes_with_type_tag() {
        let ws_id = WorkspaceId::from_uuid(uuid::Uuid::nil());
        let cmd = WorkspacePreferencesCommand::InitializeWorkspacePreferences {
            workspace_id: ws_id,
            initialized_at: sample_time(),
        };

        let json = serde_json::to_value(&cmd).unwrap();
        assert_eq!(json["type"], "InitializeWorkspacePreferences");
        assert_eq!(json["workspace_id"], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn command_roundtrips_through_json() {
        let original = WorkspacePreferencesCommand::SetDefaultCatalog {
            workspace_id: WorkspaceId::new(),
            catalog_uri: CatalogUri::new("ducklake:test").unwrap(),
            set_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: WorkspacePreferencesCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn identifier_follows_aggregate_id_pattern() {
        let ws_id = WorkspaceId::from_uuid(uuid::Uuid::nil());
        let cmd = WorkspacePreferencesCommand::InitializeWorkspacePreferences {
            workspace_id: ws_id,
            initialized_at: sample_time(),
        };

        assert_eq!(
            cmd.identifier(),
            "workspace_00000000-0000-0000-0000-000000000000/preferences"
        );
    }

    #[test]
    fn workspace_id_extracts_correctly() {
        let ws_id = WorkspaceId::new();
        let ts = sample_time();

        let commands = vec![
            WorkspacePreferencesCommand::InitializeWorkspacePreferences {
                workspace_id: ws_id,
                initialized_at: ts,
            },
            WorkspacePreferencesCommand::SetDefaultCatalog {
                workspace_id: ws_id,
                catalog_uri: CatalogUri::new("ducklake:test").unwrap(),
                set_at: ts,
            },
            WorkspacePreferencesCommand::ClearDefaultCatalog {
                workspace_id: ws_id,
                cleared_at: ts,
            },
            WorkspacePreferencesCommand::UpdateLayoutDefaults {
                workspace_id: ws_id,
                layout_defaults: LayoutDefaults::default(),
                updated_at: ts,
            },
        ];

        for cmd in commands {
            assert_eq!(cmd.workspace_id(), ws_id);
        }
    }
}
