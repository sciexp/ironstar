//! State types for the QuerySession aggregate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::super::analytics::{ChartConfig, DatasetRef, QueryId, SqlQuery};

/// Lifecycle status of a query session.
///
/// This is a sum type representing the discrete states a session can be in.
/// Each variant carries the data relevant to that state, making illegal
/// states unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuerySessionStatus {
    /// No active query. Ready to accept StartQuery.
    #[default]
    Idle,

    /// Query submitted, waiting for execution to begin.
    Pending {
        query_id: QueryId,
        sql: SqlQuery,
        dataset_ref: Option<DatasetRef>,
        chart_config: Option<ChartConfig>,
        started_at: DateTime<Utc>,
    },

    /// Query is currently executing in DuckDB.
    Executing {
        query_id: QueryId,
        sql: SqlQuery,
        dataset_ref: Option<DatasetRef>,
        chart_config: Option<ChartConfig>,
        started_at: DateTime<Utc>,
        began_at: DateTime<Utc>,
    },

    /// Query completed successfully.
    Completed {
        query_id: QueryId,
        row_count: usize,
        duration_ms: u64,
        completed_at: DateTime<Utc>,
    },

    /// Query execution failed.
    Failed {
        query_id: QueryId,
        error: String,
        failed_at: DateTime<Utc>,
    },

    /// Query was cancelled.
    Cancelled {
        query_id: QueryId,
        reason: Option<String>,
        cancelled_at: DateTime<Utc>,
    },
}

impl QuerySessionStatus {
    /// Check if this is the idle state.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Check if a query is in progress (pending or executing).
    #[must_use]
    pub fn is_in_progress(&self) -> bool {
        matches!(self, Self::Pending { .. } | Self::Executing { .. })
    }

    /// Check if this is a terminal state (completed, failed, or cancelled).
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. } | Self::Failed { .. } | Self::Cancelled { .. }
        )
    }

    /// Get the current query ID if one exists.
    #[must_use]
    pub fn query_id(&self) -> Option<QueryId> {
        match self {
            Self::Idle => None,
            Self::Pending { query_id, .. }
            | Self::Executing { query_id, .. }
            | Self::Completed { query_id, .. }
            | Self::Failed { query_id, .. }
            | Self::Cancelled { query_id, .. } => Some(*query_id),
        }
    }

    /// Get the state name for error messages.
    #[must_use]
    pub fn state_name(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Pending { .. } => "pending",
            Self::Executing { .. } => "executing",
            Self::Completed { .. } => "completed",
            Self::Failed { .. } => "failed",
            Self::Cancelled { .. } => "cancelled",
        }
    }
}

/// State of a query session, derived from events.
///
/// The state contains the current status and any metadata tracked
/// across the session lifecycle.
#[derive(Debug, Clone, Default)]
pub struct QuerySessionState {
    /// Current lifecycle status.
    pub status: QuerySessionStatus,
    /// Count of queries executed in this session (for analytics).
    pub query_count: usize,
}

impl QuerySessionState {
    /// Check if the session is idle.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.status.is_idle()
    }

    /// Check if a query is in progress.
    #[must_use]
    pub fn is_in_progress(&self) -> bool {
        self.status.is_in_progress()
    }

    /// Check if the session is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    /// Get the current query ID if one exists.
    #[must_use]
    pub fn current_query_id(&self) -> Option<QueryId> {
        self.status.query_id()
    }
}
