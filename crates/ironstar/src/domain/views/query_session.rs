//! QuerySession View for read-side projections.
//!
//! The View materializes QuerySession events into queryable state tracking
//! the current session status and complete query history. Unlike the Decider's
//! state (which only tracks `query_count`), the View retains the full history
//! of completed, failed, and cancelled queries for UI display.

use chrono::{DateTime, Utc};
use fmodel_rust::view::View;

use crate::domain::{
    ChartConfig, DatasetRef, QueryId, QuerySessionEvent, QuerySessionStatus, SqlQuery,
};

/// Outcome of a completed query lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryOutcome {
    Completed {
        row_count: usize,
        duration_ms: u64,
        completed_at: DateTime<Utc>,
    },
    Failed {
        error: String,
        failed_at: DateTime<Utc>,
    },
    Cancelled {
        reason: Option<String>,
        cancelled_at: DateTime<Utc>,
    },
}

/// A single entry in the query history.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryHistoryEntry {
    pub query_id: QueryId,
    pub sql: SqlQuery,
    pub dataset_ref: Option<DatasetRef>,
    pub chart_config: Option<ChartConfig>,
    pub started_at: DateTime<Utc>,
    pub outcome: QueryOutcome,
}

/// State materialized by the QuerySession View.
///
/// Tracks the current session status and accumulated query history.
/// The history only contains queries that have reached a terminal state
/// (completed, failed, or cancelled).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QuerySessionViewState {
    pub status: QuerySessionStatus,
    pub query_history: Vec<QueryHistoryEntry>,
    pub completed_count: usize,
    pub failed_count: usize,
    pub cancelled_count: usize,
}

impl QuerySessionViewState {
    /// Total number of queries that reached a terminal state.
    #[must_use]
    pub fn total_finished(&self) -> usize {
        self.completed_count + self.failed_count + self.cancelled_count
    }

    /// Whether the session is currently idle.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.status.is_idle()
    }

    /// Whether a query is currently in progress.
    #[must_use]
    pub fn is_in_progress(&self) -> bool {
        self.status.is_in_progress()
    }
}

/// Type alias for the QuerySession View.
pub type QuerySessionView<'a> = View<'a, QuerySessionViewState, QuerySessionEvent>;

/// Factory function creating a pure QuerySession View.
pub fn query_session_view<'a>() -> QuerySessionView<'a> {
    View {
        evolve: Box::new(evolve),
        initial_state: Box::new(QuerySessionViewState::default),
    }
}

/// Pure evolve function: (State, Event) -> State
///
/// Tracks both the current session status (mirroring the Decider) and
/// accumulates query history entries when queries reach terminal states.
fn evolve(state: &QuerySessionViewState, event: &QuerySessionEvent) -> QuerySessionViewState {
    match event {
        QuerySessionEvent::QueryStarted {
            query_id,
            sql,
            dataset_ref,
            chart_config,
            started_at,
        } => QuerySessionViewState {
            status: QuerySessionStatus::Pending {
                query_id: *query_id,
                sql: sql.clone(),
                dataset_ref: dataset_ref.clone(),
                chart_config: chart_config.clone(),
                started_at: *started_at,
            },
            query_history: state.query_history.clone(),
            ..*state
        },

        QuerySessionEvent::ExecutionBegan {
            query_id,
            began_at,
        } => {
            // Extract pending fields to promote to Executing
            if let QuerySessionStatus::Pending {
                sql,
                dataset_ref,
                chart_config,
                started_at,
                ..
            } = &state.status
            {
                QuerySessionViewState {
                    status: QuerySessionStatus::Executing {
                        query_id: *query_id,
                        sql: sql.clone(),
                        dataset_ref: dataset_ref.clone(),
                        chart_config: chart_config.clone(),
                        started_at: *started_at,
                        began_at: *began_at,
                    },
                    query_history: state.query_history.clone(),
                    ..*state
                }
            } else {
                // View is infallible — if event order is wrong, preserve state
                state.clone()
            }
        }

        QuerySessionEvent::QueryCompleted {
            query_id,
            row_count,
            duration_ms,
            completed_at,
        } => {
            let mut history = state.query_history.clone();
            if let Some(entry) = build_history_entry(state, *query_id, |_| {
                QueryOutcome::Completed {
                    row_count: *row_count,
                    duration_ms: *duration_ms,
                    completed_at: *completed_at,
                }
            }) {
                history.push(entry);
            }
            QuerySessionViewState {
                status: QuerySessionStatus::Completed {
                    query_id: *query_id,
                    row_count: *row_count,
                    duration_ms: *duration_ms,
                    completed_at: *completed_at,
                },
                query_history: history,
                completed_count: state.completed_count + 1,
                ..*state
            }
        }

        QuerySessionEvent::QueryFailed {
            query_id,
            error,
            failed_at,
        } => {
            let mut history = state.query_history.clone();
            if let Some(entry) = build_history_entry(state, *query_id, |_| QueryOutcome::Failed {
                error: error.clone(),
                failed_at: *failed_at,
            }) {
                history.push(entry);
            }
            QuerySessionViewState {
                status: QuerySessionStatus::Failed {
                    query_id: *query_id,
                    error: error.clone(),
                    failed_at: *failed_at,
                },
                query_history: history,
                failed_count: state.failed_count + 1,
                ..*state
            }
        }

        QuerySessionEvent::QueryCancelled {
            query_id,
            reason,
            cancelled_at,
        } => {
            let mut history = state.query_history.clone();
            if let Some(entry) =
                build_history_entry(state, *query_id, |_| QueryOutcome::Cancelled {
                    reason: reason.clone(),
                    cancelled_at: *cancelled_at,
                })
            {
                history.push(entry);
            }
            QuerySessionViewState {
                status: QuerySessionStatus::Cancelled {
                    query_id: *query_id,
                    reason: reason.clone(),
                    cancelled_at: *cancelled_at,
                },
                query_history: history,
                cancelled_count: state.cancelled_count + 1,
                ..*state
            }
        }

        QuerySessionEvent::SessionReset { .. } => QuerySessionViewState {
            status: QuerySessionStatus::Idle,
            query_history: state.query_history.clone(),
            ..*state
        },
    }
}

/// Extract query metadata from the current session status to build a history entry.
///
/// Returns `None` if the current status doesn't contain the query metadata
/// (e.g., if events arrive out of order — the View remains infallible).
fn build_history_entry(
    state: &QuerySessionViewState,
    query_id: QueryId,
    make_outcome: impl FnOnce(QueryId) -> QueryOutcome,
) -> Option<QueryHistoryEntry> {
    match &state.status {
        QuerySessionStatus::Pending {
            sql,
            dataset_ref,
            chart_config,
            started_at,
            ..
        }
        | QuerySessionStatus::Executing {
            sql,
            dataset_ref,
            chart_config,
            started_at,
            ..
        } => Some(QueryHistoryEntry {
            query_id,
            sql: sql.clone(),
            dataset_ref: dataset_ref.clone(),
            chart_config: chart_config.clone(),
            started_at: *started_at,
            outcome: make_outcome(query_id),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use fmodel_rust::view::ViewStateComputation;

    use super::*;

    fn as_refs(events: &[QuerySessionEvent]) -> Vec<&QuerySessionEvent> {
        events.iter().collect()
    }

    fn sample_query_id() -> QueryId {
        QueryId::new()
    }

    fn sample_sql() -> SqlQuery {
        SqlQuery::new("SELECT 1").unwrap()
    }

    #[test]
    fn initial_state_is_idle_with_empty_history() {
        let view = query_session_view();
        let state = (view.initial_state)();
        assert!(state.is_idle());
        assert!(state.query_history.is_empty());
        assert_eq!(state.total_finished(), 0);
    }

    #[test]
    fn query_started_transitions_to_pending() {
        let view = query_session_view();
        let qid = sample_query_id();
        let events = vec![QuerySessionEvent::QueryStarted {
            query_id: qid,
            sql: sample_sql(),
            dataset_ref: None,
            chart_config: None,
            started_at: Utc::now(),
        }];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(state.status.is_in_progress());
        assert!(state.query_history.is_empty());
    }

    #[test]
    fn execution_began_transitions_to_executing() {
        let view = query_session_view();
        let qid = sample_query_id();
        let events = vec![
            QuerySessionEvent::QueryStarted {
                query_id: qid,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: Utc::now(),
            },
            QuerySessionEvent::ExecutionBegan {
                query_id: qid,
                began_at: Utc::now(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(matches!(state.status, QuerySessionStatus::Executing { .. }));
        assert!(state.query_history.is_empty());
    }

    #[test]
    fn query_completed_adds_to_history() {
        let view = query_session_view();
        let qid = sample_query_id();
        let events = vec![
            QuerySessionEvent::QueryStarted {
                query_id: qid,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: Utc::now(),
            },
            QuerySessionEvent::ExecutionBegan {
                query_id: qid,
                began_at: Utc::now(),
            },
            QuerySessionEvent::QueryCompleted {
                query_id: qid,
                row_count: 42,
                duration_ms: 150,
                completed_at: Utc::now(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(matches!(state.status, QuerySessionStatus::Completed { .. }));
        assert_eq!(state.query_history.len(), 1);
        assert_eq!(state.completed_count, 1);
        assert_eq!(state.failed_count, 0);
        assert_eq!(state.total_finished(), 1);

        let entry = &state.query_history[0];
        assert_eq!(entry.query_id, qid);
        assert!(matches!(
            entry.outcome,
            QueryOutcome::Completed {
                row_count: 42,
                duration_ms: 150,
                ..
            }
        ));
    }

    #[test]
    fn query_failed_adds_to_history() {
        let view = query_session_view();
        let qid = sample_query_id();
        let events = vec![
            QuerySessionEvent::QueryStarted {
                query_id: qid,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: Utc::now(),
            },
            QuerySessionEvent::QueryFailed {
                query_id: qid,
                error: "table not found".to_string(),
                failed_at: Utc::now(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(matches!(state.status, QuerySessionStatus::Failed { .. }));
        assert_eq!(state.query_history.len(), 1);
        assert_eq!(state.failed_count, 1);
        assert_eq!(state.completed_count, 0);

        let entry = &state.query_history[0];
        assert!(matches!(
            &entry.outcome,
            QueryOutcome::Failed { error, .. } if error == "table not found"
        ));
    }

    #[test]
    fn query_cancelled_adds_to_history() {
        let view = query_session_view();
        let qid = sample_query_id();
        let events = vec![
            QuerySessionEvent::QueryStarted {
                query_id: qid,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: Utc::now(),
            },
            QuerySessionEvent::QueryCancelled {
                query_id: qid,
                reason: Some("user requested".to_string()),
                cancelled_at: Utc::now(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(matches!(
            state.status,
            QuerySessionStatus::Cancelled { .. }
        ));
        assert_eq!(state.query_history.len(), 1);
        assert_eq!(state.cancelled_count, 1);
    }

    #[test]
    fn session_reset_preserves_history() {
        let view = query_session_view();
        let qid = sample_query_id();
        let events = vec![
            QuerySessionEvent::QueryStarted {
                query_id: qid,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: Utc::now(),
            },
            QuerySessionEvent::QueryCompleted {
                query_id: qid,
                row_count: 10,
                duration_ms: 50,
                completed_at: Utc::now(),
            },
            QuerySessionEvent::SessionReset {
                reset_at: Utc::now(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(state.is_idle());
        assert_eq!(state.query_history.len(), 1);
        assert_eq!(state.completed_count, 1);
    }

    #[test]
    fn multiple_query_lifecycle_accumulates_history() {
        let view = query_session_view();
        let qid1 = sample_query_id();
        let qid2 = sample_query_id();
        let events = vec![
            // First query: completed
            QuerySessionEvent::QueryStarted {
                query_id: qid1,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: Utc::now(),
            },
            QuerySessionEvent::QueryCompleted {
                query_id: qid1,
                row_count: 10,
                duration_ms: 50,
                completed_at: Utc::now(),
            },
            QuerySessionEvent::SessionReset {
                reset_at: Utc::now(),
            },
            // Second query: failed
            QuerySessionEvent::QueryStarted {
                query_id: qid2,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: Utc::now(),
            },
            QuerySessionEvent::QueryFailed {
                query_id: qid2,
                error: "timeout".to_string(),
                failed_at: Utc::now(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert_eq!(state.query_history.len(), 2);
        assert_eq!(state.completed_count, 1);
        assert_eq!(state.failed_count, 1);
        assert_eq!(state.total_finished(), 2);
        assert_eq!(state.query_history[0].query_id, qid1);
        assert_eq!(state.query_history[1].query_id, qid2);
    }
}
