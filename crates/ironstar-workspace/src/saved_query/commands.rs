//! Commands for the SavedQuery aggregate.
//!
//! Commands represent requests to change saved query state.
//! Timestamps are injected at the boundary layer; the pure decider
//! does not call `Utc::now()`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{QueryName, SavedQueryId};
use crate::workspace::WorkspaceId;
use ironstar_analytics::{DatasetRef, SqlQuery};
use ironstar_core::{DeciderType, Identifier};

/// Commands that can be sent to the SavedQuery aggregate.
///
/// The aggregate ID follows the pattern `saved_query_{query_id}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "commands/")]
pub enum SavedQueryCommand {
    /// Save a new query.
    ///
    /// Can only succeed when no query exists with this ID.
    SaveQuery {
        query_id: SavedQueryId,
        workspace_id: WorkspaceId,
        name: QueryName,
        sql: SqlQuery,
        dataset_ref: DatasetRef,
        saved_at: DateTime<Utc>,
    },

    /// Delete an existing query.
    ///
    /// Transitions the aggregate to the terminal NoQuery state.
    DeleteQuery {
        query_id: SavedQueryId,
        deleted_at: DateTime<Utc>,
    },

    /// Rename an existing query.
    ///
    /// Idempotent when setting the same name.
    RenameQuery {
        query_id: SavedQueryId,
        name: QueryName,
        renamed_at: DateTime<Utc>,
    },

    /// Update the SQL of an existing query.
    ///
    /// Idempotent when setting the same SQL.
    UpdateQuerySql {
        query_id: SavedQueryId,
        sql: SqlQuery,
        updated_at: DateTime<Utc>,
    },

    /// Update the dataset reference of an existing query.
    ///
    /// Idempotent when setting the same dataset reference.
    UpdateDatasetRef {
        query_id: SavedQueryId,
        dataset_ref: DatasetRef,
        updated_at: DateTime<Utc>,
    },
}

impl SavedQueryCommand {
    /// Extract the query ID from the command.
    #[must_use]
    pub fn query_id(&self) -> SavedQueryId {
        match self {
            Self::SaveQuery { query_id, .. }
            | Self::DeleteQuery { query_id, .. }
            | Self::RenameQuery { query_id, .. }
            | Self::UpdateQuerySql { query_id, .. }
            | Self::UpdateDatasetRef { query_id, .. } => *query_id,
        }
    }

    /// Get the command type name for logging and metrics.
    #[must_use]
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::SaveQuery { .. } => "SaveQuery",
            Self::DeleteQuery { .. } => "DeleteQuery",
            Self::RenameQuery { .. } => "RenameQuery",
            Self::UpdateQuerySql { .. } => "UpdateQuerySql",
            Self::UpdateDatasetRef { .. } => "UpdateDatasetRef",
        }
    }
}

impl Identifier for SavedQueryCommand {
    fn identifier(&self) -> String {
        format!("saved_query_{}", self.query_id())
    }
}

impl DeciderType for SavedQueryCommand {
    fn decider_type(&self) -> String {
        "SavedQuery".to_string()
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
    fn save_command_serializes_with_type_tag() {
        let qid = SavedQueryId::from_uuid(uuid::Uuid::nil());
        let ws_id = WorkspaceId::from_uuid(uuid::Uuid::nil());
        let cmd = SavedQueryCommand::SaveQuery {
            query_id: qid,
            workspace_id: ws_id,
            name: QueryName::new("Test").unwrap(),
            sql: SqlQuery::new("SELECT 1").unwrap(),
            dataset_ref: DatasetRef::new("hf://datasets/test/data").unwrap(),
            saved_at: sample_time(),
        };

        let json = serde_json::to_value(&cmd).unwrap();
        assert_eq!(json["type"], "SaveQuery");
        assert_eq!(json["query_id"], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn command_roundtrips_through_json() {
        let original = SavedQueryCommand::RenameQuery {
            query_id: SavedQueryId::new(),
            name: QueryName::new("New Name").unwrap(),
            renamed_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: SavedQueryCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn identifier_follows_aggregate_id_pattern() {
        let qid = SavedQueryId::from_uuid(uuid::Uuid::nil());
        let cmd = SavedQueryCommand::DeleteQuery {
            query_id: qid,
            deleted_at: sample_time(),
        };

        assert_eq!(
            cmd.identifier(),
            "saved_query_00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn query_id_extracts_correctly() {
        let qid = SavedQueryId::new();
        let ws_id = WorkspaceId::new();
        let ts = sample_time();

        let commands = vec![
            SavedQueryCommand::SaveQuery {
                query_id: qid,
                workspace_id: ws_id,
                name: QueryName::new("Test").unwrap(),
                sql: SqlQuery::new("SELECT 1").unwrap(),
                dataset_ref: DatasetRef::new("hf://datasets/test/data").unwrap(),
                saved_at: ts,
            },
            SavedQueryCommand::DeleteQuery {
                query_id: qid,
                deleted_at: ts,
            },
            SavedQueryCommand::RenameQuery {
                query_id: qid,
                name: QueryName::new("New").unwrap(),
                renamed_at: ts,
            },
            SavedQueryCommand::UpdateQuerySql {
                query_id: qid,
                sql: SqlQuery::new("SELECT 2").unwrap(),
                updated_at: ts,
            },
            SavedQueryCommand::UpdateDatasetRef {
                query_id: qid,
                dataset_ref: DatasetRef::new("s3://bucket/data").unwrap(),
                updated_at: ts,
            },
        ];

        for cmd in commands {
            assert_eq!(cmd.query_id(), qid);
        }
    }
}
