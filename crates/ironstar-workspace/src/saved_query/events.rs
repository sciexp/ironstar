//! Domain events for the SavedQuery aggregate.
//!
//! Events represent facts that have occurred. They are immutable, past-tense,
//! and self-describing per Hoffman's Law 1.
//!
//! The `QueryDeleted` event is terminal (`is_final` returns true), meaning
//! the aggregate transitions back to its initial NoQuery state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{QueryName, SavedQueryId};
use crate::workspace::WorkspaceId;
use ironstar_analytics::{DatasetRef, SqlQuery};
use ironstar_core::{DeciderType, EventType, Identifier, IsFinal};

/// Events emitted by the SavedQuery aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "events/")]
pub enum SavedQueryEvent {
    /// A query was saved.
    QuerySaved {
        query_id: SavedQueryId,
        workspace_id: WorkspaceId,
        name: QueryName,
        sql: SqlQuery,
        dataset_ref: DatasetRef,
        saved_at: DateTime<Utc>,
    },

    /// A query was deleted (terminal event).
    QueryDeleted {
        query_id: SavedQueryId,
        deleted_at: DateTime<Utc>,
    },

    /// A query was renamed.
    QueryRenamed {
        query_id: SavedQueryId,
        name: QueryName,
        renamed_at: DateTime<Utc>,
    },

    /// The SQL of a query was updated.
    QuerySqlUpdated {
        query_id: SavedQueryId,
        sql: SqlQuery,
        updated_at: DateTime<Utc>,
    },

    /// The dataset reference of a query was updated.
    DatasetRefUpdated {
        query_id: SavedQueryId,
        dataset_ref: DatasetRef,
        updated_at: DateTime<Utc>,
    },
}

impl SavedQueryEvent {
    /// Extract the query ID this event belongs to.
    #[must_use]
    pub fn query_id(&self) -> SavedQueryId {
        match self {
            Self::QuerySaved { query_id, .. }
            | Self::QueryDeleted { query_id, .. }
            | Self::QueryRenamed { query_id, .. }
            | Self::QuerySqlUpdated { query_id, .. }
            | Self::DatasetRefUpdated { query_id, .. } => *query_id,
        }
    }

    /// Get the event type name for storage and routing.
    #[must_use]
    pub fn event_type_str(&self) -> &'static str {
        match self {
            Self::QuerySaved { .. } => "QuerySaved",
            Self::QueryDeleted { .. } => "QueryDeleted",
            Self::QueryRenamed { .. } => "QueryRenamed",
            Self::QuerySqlUpdated { .. } => "QuerySqlUpdated",
            Self::DatasetRefUpdated { .. } => "DatasetRefUpdated",
        }
    }

    /// Get the event version for schema evolution.
    #[must_use]
    pub fn event_version(&self) -> &'static str {
        "1"
    }
}

impl Identifier for SavedQueryEvent {
    fn identifier(&self) -> String {
        format!("saved_query_{}", self.query_id())
    }
}

impl EventType for SavedQueryEvent {
    fn event_type(&self) -> String {
        self.event_type_str().to_string()
    }
}

impl DeciderType for SavedQueryEvent {
    fn decider_type(&self) -> String {
        "SavedQuery".to_string()
    }
}

impl IsFinal for SavedQueryEvent {
    fn is_final(&self) -> bool {
        matches!(self, Self::QueryDeleted { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_id() -> SavedQueryId {
        SavedQueryId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn saved_event_serializes_with_type_tag() {
        let event = SavedQueryEvent::QuerySaved {
            query_id: sample_id(),
            workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
            name: QueryName::new("Test").unwrap(),
            sql: SqlQuery::new("SELECT 1").unwrap(),
            dataset_ref: DatasetRef::new("hf://datasets/test/data").unwrap(),
            saved_at: sample_time(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "QuerySaved");
        assert_eq!(json["query_id"], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn event_roundtrips_through_json() {
        let original = SavedQueryEvent::QueryRenamed {
            query_id: sample_id(),
            name: QueryName::new("New Name").unwrap(),
            renamed_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: SavedQueryEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn identifier_follows_aggregate_id_pattern() {
        let event = SavedQueryEvent::QuerySaved {
            query_id: sample_id(),
            workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
            name: QueryName::new("Test").unwrap(),
            sql: SqlQuery::new("SELECT 1").unwrap(),
            dataset_ref: DatasetRef::new("hf://datasets/test/data").unwrap(),
            saved_at: sample_time(),
        };

        assert_eq!(
            event.identifier(),
            "saved_query_00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn event_type_matches_serde_tag() {
        let events: Vec<(SavedQueryEvent, &str)> = vec![
            (
                SavedQueryEvent::QuerySaved {
                    query_id: sample_id(),
                    workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
                    name: QueryName::new("Test").unwrap(),
                    sql: SqlQuery::new("SELECT 1").unwrap(),
                    dataset_ref: DatasetRef::new("hf://datasets/test/data").unwrap(),
                    saved_at: sample_time(),
                },
                "QuerySaved",
            ),
            (
                SavedQueryEvent::QueryDeleted {
                    query_id: sample_id(),
                    deleted_at: sample_time(),
                },
                "QueryDeleted",
            ),
            (
                SavedQueryEvent::QueryRenamed {
                    query_id: sample_id(),
                    name: QueryName::new("Renamed").unwrap(),
                    renamed_at: sample_time(),
                },
                "QueryRenamed",
            ),
            (
                SavedQueryEvent::QuerySqlUpdated {
                    query_id: sample_id(),
                    sql: SqlQuery::new("SELECT 2").unwrap(),
                    updated_at: sample_time(),
                },
                "QuerySqlUpdated",
            ),
            (
                SavedQueryEvent::DatasetRefUpdated {
                    query_id: sample_id(),
                    dataset_ref: DatasetRef::new("s3://bucket/data").unwrap(),
                    updated_at: sample_time(),
                },
                "DatasetRefUpdated",
            ),
        ];

        for (event, expected_type) in events {
            assert_eq!(event.event_type_str(), expected_type);

            let json = serde_json::to_value(&event).unwrap();
            assert_eq!(json["type"], expected_type);
        }
    }

    #[test]
    fn is_final_returns_true_only_for_deleted() {
        let deleted = SavedQueryEvent::QueryDeleted {
            query_id: sample_id(),
            deleted_at: sample_time(),
        };
        assert!(deleted.is_final());

        let saved = SavedQueryEvent::QuerySaved {
            query_id: sample_id(),
            workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
            name: QueryName::new("Test").unwrap(),
            sql: SqlQuery::new("SELECT 1").unwrap(),
            dataset_ref: DatasetRef::new("hf://datasets/test/data").unwrap(),
            saved_at: sample_time(),
        };
        assert!(!saved.is_final());

        let renamed = SavedQueryEvent::QueryRenamed {
            query_id: sample_id(),
            name: QueryName::new("New").unwrap(),
            renamed_at: sample_time(),
        };
        assert!(!renamed.is_final());
    }
}
