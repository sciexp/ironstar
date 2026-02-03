//! SavedQuery aggregate state types.
//!
//! State is derived from events via replay. Uses a sum type enum with
//! a terminal transition: DeleteQuery returns the aggregate to NoQuery.

use super::values::{QueryName, SavedQueryId};
use crate::domain::analytics::{DatasetRef, SqlQuery};
use crate::domain::workspace::WorkspaceId;

/// State of a saved query, derived from events.
///
/// ```text
///                 ┌───────────┐
///  SaveQuery ────►│QueryExists│◄──── RenameQuery, UpdateSql, UpdateDatasetRef
///                 └─────┬─────┘
///                       │
///                  DeleteQuery
///                       │
///                       ▼
///                 ┌───────────┐
///                 │  NoQuery  │ (terminal / can be re-created)
///                 └───────────┘
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SavedQueryState {
    /// Initial state before any query has been saved, or after deletion.
    #[default]
    NoQuery,

    /// A saved query exists.
    QueryExists {
        /// Unique identifier for this saved query.
        query_id: SavedQueryId,
        /// The workspace this query belongs to.
        workspace_id: WorkspaceId,
        /// Display name for the query.
        name: QueryName,
        /// The SQL query string.
        sql: SqlQuery,
        /// Reference to the dataset this query targets.
        dataset_ref: DatasetRef,
    },
}

impl SavedQueryState {
    /// Check if a saved query exists.
    #[must_use]
    pub fn exists(&self) -> bool {
        matches!(self, Self::QueryExists { .. })
    }

    /// Get the query ID, if the query exists.
    #[must_use]
    pub fn query_id(&self) -> Option<&SavedQueryId> {
        match self {
            Self::NoQuery => None,
            Self::QueryExists { query_id, .. } => Some(query_id),
        }
    }

    /// Get the workspace ID, if the query exists.
    #[must_use]
    pub fn workspace_id(&self) -> Option<&WorkspaceId> {
        match self {
            Self::NoQuery => None,
            Self::QueryExists { workspace_id, .. } => Some(workspace_id),
        }
    }

    /// Get the query name, if the query exists.
    #[must_use]
    pub fn name(&self) -> Option<&QueryName> {
        match self {
            Self::NoQuery => None,
            Self::QueryExists { name, .. } => Some(name),
        }
    }

    /// Get the SQL query, if the query exists.
    #[must_use]
    pub fn sql(&self) -> Option<&SqlQuery> {
        match self {
            Self::NoQuery => None,
            Self::QueryExists { sql, .. } => Some(sql),
        }
    }

    /// Get the dataset reference, if the query exists.
    #[must_use]
    pub fn dataset_ref(&self) -> Option<&DatasetRef> {
        match self {
            Self::NoQuery => None,
            Self::QueryExists { dataset_ref, .. } => Some(dataset_ref),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_no_query() {
        let state = SavedQueryState::default();
        assert!(!state.exists());
        assert!(state.query_id().is_none());
        assert!(state.workspace_id().is_none());
        assert!(state.name().is_none());
        assert!(state.sql().is_none());
        assert!(state.dataset_ref().is_none());
    }

    #[test]
    fn query_exists_state() {
        let qid = SavedQueryId::from_uuid(uuid::Uuid::nil());
        let ws_id = WorkspaceId::from_uuid(uuid::Uuid::nil());
        let name = QueryName::new("Test Query").unwrap();
        let sql = SqlQuery::new("SELECT 1").unwrap();
        let dataset = DatasetRef::new("hf://datasets/test/data").unwrap();

        let state = SavedQueryState::QueryExists {
            query_id: qid,
            workspace_id: ws_id,
            name: name.clone(),
            sql: sql.clone(),
            dataset_ref: dataset.clone(),
        };

        assert!(state.exists());
        assert_eq!(state.query_id(), Some(&qid));
        assert_eq!(state.workspace_id(), Some(&ws_id));
        assert_eq!(state.name(), Some(&name));
        assert_eq!(state.sql(), Some(&sql));
        assert_eq!(state.dataset_ref(), Some(&dataset));
    }
}
