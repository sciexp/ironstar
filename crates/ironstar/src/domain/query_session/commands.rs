//! Commands for the QuerySession aggregate.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::super::analytics::{ChartConfig, DatasetRef, QueryId, SqlQuery};
use crate::domain::traits::{DeciderType, Identifier};

/// Commands for the QuerySession aggregate.
///
/// Commands represent requests to change state. They carry the data needed
/// for validation and event emission. The aggregate validates commands
/// against current state and either rejects them or emits events.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(tag = "type")]
pub enum QuerySessionCommand {
    /// Start a new query. Transitions Idle → Pending.
    StartQuery {
        /// Unique identifier for this query.
        query_id: QueryId,
        /// The SQL query to execute.
        sql: SqlQuery,
        /// Optional dataset reference (for remote data sources).
        #[serde(skip_serializing_if = "Option::is_none")]
        dataset_ref: Option<DatasetRef>,
        /// Optional chart configuration for visualization.
        #[serde(skip_serializing_if = "Option::is_none")]
        chart_config: Option<ChartConfig>,
    },

    /// Mark query execution as started (called by application layer).
    /// Transitions Pending → Executing.
    BeginExecution {
        /// Must match the pending query ID.
        query_id: QueryId,
    },

    /// Complete a query successfully. Transitions Executing → Completed.
    CompleteQuery {
        /// Must match the executing query ID.
        query_id: QueryId,
        /// Number of rows returned.
        row_count: usize,
        /// Execution duration in milliseconds.
        duration_ms: u64,
    },

    /// Mark a query as failed. Transitions Executing → Failed.
    FailQuery {
        /// Must match the executing query ID.
        query_id: QueryId,
        /// Error message describing the failure.
        error: String,
    },

    /// Cancel a pending or executing query. Transitions to Cancelled.
    CancelQuery {
        /// Must match the current query ID.
        query_id: QueryId,
        /// Optional reason for cancellation.
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// Reset the session to idle state (only from terminal states).
    ResetSession,
}

impl QuerySessionCommand {
    /// Extract the query ID if this command has one.
    #[must_use]
    pub fn query_id(&self) -> Option<QueryId> {
        match self {
            Self::StartQuery { query_id, .. }
            | Self::BeginExecution { query_id }
            | Self::CompleteQuery { query_id, .. }
            | Self::FailQuery { query_id, .. }
            | Self::CancelQuery { query_id, .. } => Some(*query_id),
            Self::ResetSession => None,
        }
    }
}

impl Identifier for QuerySessionCommand {
    fn identifier(&self) -> String {
        // Singleton aggregate pattern - all commands target the same session
        "default-session".to_string()
    }
}

impl DeciderType for QuerySessionCommand {
    fn decider_type(&self) -> String {
        "QuerySession".to_string()
    }
}
