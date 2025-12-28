//! Events emitted by the QuerySession aggregate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::super::analytics::{ChartConfig, DatasetRef, QueryId, SqlQuery};

/// Events emitted by the QuerySession aggregate.
///
/// Events represent facts that have occurred in the domain. They are
/// immutable records of state changes, persisted to the event store.
///
/// Note: PartialEq is derived for testing convenience but comparing events
/// with timestamps should use pattern matching, not assert_eq!, to avoid
/// timestamp comparison issues.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(tag = "type")]
pub enum QuerySessionEvent {
    /// A query has been started and is pending execution.
    QueryStarted {
        query_id: QueryId,
        sql: SqlQuery,
        #[serde(skip_serializing_if = "Option::is_none")]
        dataset_ref: Option<DatasetRef>,
        #[serde(skip_serializing_if = "Option::is_none")]
        chart_config: Option<ChartConfig>,
        started_at: DateTime<Utc>,
    },

    /// Query execution has begun (DuckDB task spawned).
    ExecutionBegan {
        query_id: QueryId,
        began_at: DateTime<Utc>,
    },

    /// Query completed successfully with results.
    QueryCompleted {
        query_id: QueryId,
        row_count: usize,
        duration_ms: u64,
        completed_at: DateTime<Utc>,
    },

    /// Query execution failed.
    QueryFailed {
        query_id: QueryId,
        error: String,
        failed_at: DateTime<Utc>,
    },

    /// Query was cancelled by user.
    QueryCancelled {
        query_id: QueryId,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
        cancelled_at: DateTime<Utc>,
    },

    /// Session was reset to idle state.
    SessionReset { reset_at: DateTime<Utc> },
}

impl QuerySessionEvent {
    /// Get the event type name for storage.
    #[must_use]
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::QueryStarted { .. } => "QueryStarted",
            Self::ExecutionBegan { .. } => "ExecutionBegan",
            Self::QueryCompleted { .. } => "QueryCompleted",
            Self::QueryFailed { .. } => "QueryFailed",
            Self::QueryCancelled { .. } => "QueryCancelled",
            Self::SessionReset { .. } => "SessionReset",
        }
    }

    /// Extract the query ID if this event has one.
    #[must_use]
    pub fn query_id(&self) -> Option<QueryId> {
        match self {
            Self::QueryStarted { query_id, .. }
            | Self::ExecutionBegan { query_id, .. }
            | Self::QueryCompleted { query_id, .. }
            | Self::QueryFailed { query_id, .. }
            | Self::QueryCancelled { query_id, .. } => Some(*query_id),
            Self::SessionReset { .. } => None,
        }
    }
}
