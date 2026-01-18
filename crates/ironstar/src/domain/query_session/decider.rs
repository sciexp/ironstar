//! Pure QuerySession Decider implementing fmodel-rust patterns.
//!
//! The Decider is the core decision-making component that manages the
//! lifecycle of analytics query sessions. It is a pure function with
//! no side effects: all I/O (timestamps, persistence) happens at boundaries.
//!
//! # State Machine
//!
//! ```text
//!                    ┌──────────┐
//!     StartQuery ───►│ Pending  │
//!                    └────┬─────┘
//!                         │
//!                    BeginExecution
//!                         │
//!                         ▼
//!                    ┌──────────┐
//!                    │Executing │
//!                    └────┬─────┘
//!                         │
//!           ┌─────────────┼─────────────┐
//!           │             │             │
//!      CompleteQuery  FailQuery    CancelQuery (also from Pending)
//!           │             │             │
//!           ▼             ▼             ▼
//!      ┌──────────┐  ┌──────────┐  ┌──────────┐
//!      │Completed │  │  Failed  │  │Cancelled │
//!      └──────────┘  └──────────┘  └──────────┘
//!
//!      ────────────── terminal states ──────────────
//!                         │
//!                    ResetSession
//!                         │
//!                         ▼
//!                    ┌──────────┐
//!                    │   Idle   │ (initial)
//!                    └──────────┘
//! ```
//!
//! # Spawn-After-Persist Pattern
//!
//! 1. User submits query → `StartQuery` command
//! 2. Decider emits `QueryStarted` event
//! 3. Application layer persists event to SQLite
//! 4. Application layer spawns async DuckDB task
//! 5. Task completion triggers `CompleteQuery` or `FailQuery` command
//! 6. Decider emits corresponding completion event
//!
//! The decider remains pure; all async execution happens at boundaries.
//!
//! # Idempotency
//!
//! - ResetSession from Idle returns `Ok(vec![])` (already in target state)

use fmodel_rust::decider::Decider;

use super::commands::QuerySessionCommand;
use super::errors::QuerySessionError;
use super::events::QuerySessionEvent;
use super::state::{QuerySessionState, QuerySessionStatus};

/// Type alias for the QuerySession Decider.
///
/// Unlike the Todo aggregate which uses `Option<State>` to represent
/// non-existence, QuerySession is a singleton that always exists.
/// The initial state is `Idle` with `query_count: 0`.
pub type QuerySessionDecider<'a> =
    Decider<'a, QuerySessionCommand, QuerySessionState, QuerySessionEvent, QuerySessionError>;

/// Factory function creating a pure QuerySession Decider.
///
/// The decider embodies the state machine for analytics query sessions:
/// - Idle → Pending → Executing → Completed/Failed/Cancelled (terminal)
/// - Terminal states can be reset to Idle via ResetSession
/// - ResetSession from Idle is idempotent (returns empty events)
///
/// # Example
///
/// ```rust,ignore
/// use fmodel_rust::decider::EventComputation;
/// use ironstar::domain::query_session::{query_session_decider, QuerySessionCommand, QueryId, SqlQuery};
/// use chrono::Utc;
///
/// let decider = query_session_decider();
/// let query_id = QueryId::new();
/// let sql = SqlQuery::new("SELECT 1").unwrap();
/// let now = Utc::now();
///
/// let events = decider.compute_new_events(
///     &[],
///     &QuerySessionCommand::StartQuery {
///         query_id,
///         sql,
///         dataset_ref: None,
///         chart_config: None,
///         started_at: now,
///     }
/// );
/// ```
pub fn query_session_decider<'a>() -> QuerySessionDecider<'a> {
    Decider {
        decide: Box::new(|command, state| decide(command, state)),
        evolve: Box::new(|state, event| evolve(state, event)),
        initial_state: Box::new(QuerySessionState::default),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
///
/// This function is the heart of the domain logic. It validates commands
/// against current state and returns events or errors. No side effects.
fn decide(
    command: &QuerySessionCommand,
    state: &QuerySessionState,
) -> Result<Vec<QuerySessionEvent>, QuerySessionError> {
    match command {
        // StartQuery: Idle → Pending
        QuerySessionCommand::StartQuery {
            query_id,
            sql,
            dataset_ref,
            chart_config,
            started_at,
        } => {
            if state.is_idle() {
                Ok(vec![QuerySessionEvent::QueryStarted {
                    query_id: *query_id,
                    sql: sql.clone(),
                    dataset_ref: dataset_ref.clone(),
                    chart_config: chart_config.clone(),
                    started_at: *started_at,
                }])
            } else if state.is_in_progress() {
                Err(QuerySessionError::query_already_in_progress())
            } else {
                // Terminal state - need reset first
                Err(QuerySessionError::terminal_state(
                    state.status.state_name(),
                ))
            }
        }

        // BeginExecution: Pending → Executing
        QuerySessionCommand::BeginExecution { query_id, began_at } => {
            match &state.status {
                QuerySessionStatus::Pending {
                    query_id: pending_id,
                    ..
                } => {
                    if *pending_id != *query_id {
                        return Err(QuerySessionError::query_id_mismatch(*pending_id, *query_id));
                    }
                    Ok(vec![QuerySessionEvent::ExecutionBegan {
                        query_id: *query_id,
                        began_at: *began_at,
                    }])
                }
                QuerySessionStatus::Idle => Err(QuerySessionError::no_query_in_progress()),
                _ => Err(QuerySessionError::invalid_transition(
                    "begin execution",
                    state.status.state_name(),
                )),
            }
        }

        // CompleteQuery: Executing → Completed
        QuerySessionCommand::CompleteQuery {
            query_id,
            row_count,
            duration_ms,
            completed_at,
        } => match &state.status {
            QuerySessionStatus::Executing {
                query_id: executing_id,
                ..
            } => {
                if *executing_id != *query_id {
                    return Err(QuerySessionError::query_id_mismatch(
                        *executing_id,
                        *query_id,
                    ));
                }
                Ok(vec![QuerySessionEvent::QueryCompleted {
                    query_id: *query_id,
                    row_count: *row_count,
                    duration_ms: *duration_ms,
                    completed_at: *completed_at,
                }])
            }
            QuerySessionStatus::Idle => Err(QuerySessionError::no_query_in_progress()),
            _ => Err(QuerySessionError::invalid_transition(
                "complete query",
                state.status.state_name(),
            )),
        },

        // FailQuery: Executing → Failed
        QuerySessionCommand::FailQuery {
            query_id,
            error,
            failed_at,
        } => match &state.status {
            QuerySessionStatus::Executing {
                query_id: executing_id,
                ..
            } => {
                if *executing_id != *query_id {
                    return Err(QuerySessionError::query_id_mismatch(
                        *executing_id,
                        *query_id,
                    ));
                }
                Ok(vec![QuerySessionEvent::QueryFailed {
                    query_id: *query_id,
                    error: error.clone(),
                    failed_at: *failed_at,
                }])
            }
            QuerySessionStatus::Idle => Err(QuerySessionError::no_query_in_progress()),
            _ => Err(QuerySessionError::invalid_transition(
                "fail query",
                state.status.state_name(),
            )),
        },

        // CancelQuery: Pending/Executing → Cancelled
        QuerySessionCommand::CancelQuery {
            query_id,
            reason,
            cancelled_at,
        } => match &state.status {
            QuerySessionStatus::Pending {
                query_id: pending_id,
                ..
            }
            | QuerySessionStatus::Executing {
                query_id: pending_id,
                ..
            } => {
                if *pending_id != *query_id {
                    return Err(QuerySessionError::query_id_mismatch(*pending_id, *query_id));
                }
                Ok(vec![QuerySessionEvent::QueryCancelled {
                    query_id: *query_id,
                    reason: reason.clone(),
                    cancelled_at: *cancelled_at,
                }])
            }
            QuerySessionStatus::Idle => Err(QuerySessionError::no_query_in_progress()),
            _ => Err(QuerySessionError::terminal_state(
                state.status.state_name(),
            )),
        },

        // ResetSession: Terminal → Idle (idempotent from Idle)
        QuerySessionCommand::ResetSession { reset_at } => {
            if state.is_idle() {
                // Idempotent: already idle
                Ok(vec![])
            } else if state.is_terminal() {
                Ok(vec![QuerySessionEvent::SessionReset {
                    reset_at: *reset_at,
                }])
            } else {
                // Cannot reset while query in progress
                Err(QuerySessionError::invalid_transition(
                    "reset session",
                    state.status.state_name(),
                ))
            }
        }
    }
}

/// Pure evolve function: (State, Event) -> State
///
/// This function applies an event to produce new state. It must be
/// deterministic and total (handle all event variants).
fn evolve(state: &QuerySessionState, event: &QuerySessionEvent) -> QuerySessionState {
    match event {
        // QueryStarted: Idle → Pending
        QuerySessionEvent::QueryStarted {
            query_id,
            sql,
            dataset_ref,
            chart_config,
            started_at,
        } => QuerySessionState {
            status: QuerySessionStatus::Pending {
                query_id: *query_id,
                sql: sql.clone(),
                dataset_ref: dataset_ref.clone(),
                chart_config: chart_config.clone(),
                started_at: *started_at,
            },
            query_count: state.query_count,
        },

        // ExecutionBegan: Pending → Executing
        QuerySessionEvent::ExecutionBegan { began_at, .. } => {
            // Transition from Pending to Executing, preserving query data
            if let QuerySessionStatus::Pending {
                query_id,
                sql,
                dataset_ref,
                chart_config,
                started_at,
            } = &state.status
            {
                QuerySessionState {
                    status: QuerySessionStatus::Executing {
                        query_id: *query_id,
                        sql: sql.clone(),
                        dataset_ref: dataset_ref.clone(),
                        chart_config: chart_config.clone(),
                        started_at: *started_at,
                        began_at: *began_at,
                    },
                    query_count: state.query_count,
                }
            } else {
                // Should never happen if decide is correct, but preserve state
                state.clone()
            }
        }

        // QueryCompleted: Executing → Completed (increments query_count)
        QuerySessionEvent::QueryCompleted {
            query_id,
            row_count,
            duration_ms,
            completed_at,
        } => QuerySessionState {
            status: QuerySessionStatus::Completed {
                query_id: *query_id,
                row_count: *row_count,
                duration_ms: *duration_ms,
                completed_at: *completed_at,
            },
            query_count: state.query_count + 1,
        },

        // QueryFailed: Executing → Failed (increments query_count)
        QuerySessionEvent::QueryFailed {
            query_id,
            error,
            failed_at,
        } => QuerySessionState {
            status: QuerySessionStatus::Failed {
                query_id: *query_id,
                error: error.clone(),
                failed_at: *failed_at,
            },
            query_count: state.query_count + 1,
        },

        // QueryCancelled: Pending/Executing → Cancelled (does NOT increment query_count)
        QuerySessionEvent::QueryCancelled {
            query_id,
            reason,
            cancelled_at,
        } => QuerySessionState {
            status: QuerySessionStatus::Cancelled {
                query_id: *query_id,
                reason: reason.clone(),
                cancelled_at: *cancelled_at,
            },
            query_count: state.query_count,
        },

        // SessionReset: Terminal → Idle (preserves query_count)
        QuerySessionEvent::SessionReset { .. } => QuerySessionState {
            status: QuerySessionStatus::Idle,
            query_count: state.query_count,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use fmodel_rust::specification::DeciderTestSpecification;

    use crate::domain::analytics::{QueryId, SqlQuery};

    fn sample_query_id() -> QueryId {
        QueryId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_sql() -> SqlQuery {
        SqlQuery::new("SELECT * FROM test").unwrap()
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    // --- StartQuery transitions ---

    #[test]
    fn start_query_from_idle_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![])
            .when(QuerySessionCommand::StartQuery {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            })
            .then(vec![QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            }]);
    }

    #[test]
    fn start_query_when_pending_fails() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            }])
            .when(QuerySessionCommand::StartQuery {
                query_id: QueryId::new(), // Different ID
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            })
            .then_error(QuerySessionError::query_already_in_progress());
    }

    #[test]
    fn start_query_from_completed_fails() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
                QuerySessionEvent::QueryCompleted {
                    query_id,
                    row_count: 10,
                    duration_ms: 100,
                    completed_at: ts,
                },
            ])
            .when(QuerySessionCommand::StartQuery {
                query_id: QueryId::new(),
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            })
            .then_error(QuerySessionError::terminal_state("completed"));
    }

    // --- BeginExecution transitions ---

    #[test]
    fn begin_execution_from_pending_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            }])
            .when(QuerySessionCommand::BeginExecution {
                query_id,
                began_at: ts,
            })
            .then(vec![QuerySessionEvent::ExecutionBegan {
                query_id,
                began_at: ts,
            }]);
    }

    #[test]
    fn begin_execution_with_wrong_id_fails() {
        let query_id = sample_query_id();
        let wrong_id = QueryId::new();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            }])
            .when(QuerySessionCommand::BeginExecution {
                query_id: wrong_id,
                began_at: ts,
            })
            .then_error(QuerySessionError::query_id_mismatch(query_id, wrong_id));
    }

    #[test]
    fn begin_execution_from_idle_fails() {
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![])
            .when(QuerySessionCommand::BeginExecution {
                query_id: sample_query_id(),
                began_at: ts,
            })
            .then_error(QuerySessionError::no_query_in_progress());
    }

    // --- CompleteQuery transitions ---

    #[test]
    fn complete_query_from_executing_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
            ])
            .when(QuerySessionCommand::CompleteQuery {
                query_id,
                row_count: 100,
                duration_ms: 1500,
                completed_at: ts,
            })
            .then(vec![QuerySessionEvent::QueryCompleted {
                query_id,
                row_count: 100,
                duration_ms: 1500,
                completed_at: ts,
            }]);
    }

    #[test]
    fn complete_query_with_wrong_id_fails() {
        let query_id = sample_query_id();
        let wrong_id = QueryId::new();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
            ])
            .when(QuerySessionCommand::CompleteQuery {
                query_id: wrong_id,
                row_count: 100,
                duration_ms: 1500,
                completed_at: ts,
            })
            .then_error(QuerySessionError::query_id_mismatch(query_id, wrong_id));
    }

    #[test]
    fn complete_query_from_pending_fails() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            }])
            .when(QuerySessionCommand::CompleteQuery {
                query_id,
                row_count: 100,
                duration_ms: 1500,
                completed_at: ts,
            })
            .then_error(QuerySessionError::invalid_transition(
                "complete query",
                "pending",
            ));
    }

    // --- FailQuery transitions ---

    #[test]
    fn fail_query_from_executing_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
            ])
            .when(QuerySessionCommand::FailQuery {
                query_id,
                error: "Syntax error".to_string(),
                failed_at: ts,
            })
            .then(vec![QuerySessionEvent::QueryFailed {
                query_id,
                error: "Syntax error".to_string(),
                failed_at: ts,
            }]);
    }

    #[test]
    fn fail_query_with_wrong_id_fails() {
        let query_id = sample_query_id();
        let wrong_id = QueryId::new();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
            ])
            .when(QuerySessionCommand::FailQuery {
                query_id: wrong_id,
                error: "Syntax error".to_string(),
                failed_at: ts,
            })
            .then_error(QuerySessionError::query_id_mismatch(query_id, wrong_id));
    }

    // --- CancelQuery transitions ---

    #[test]
    fn cancel_pending_query_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            }])
            .when(QuerySessionCommand::CancelQuery {
                query_id,
                reason: Some("User cancelled".to_string()),
                cancelled_at: ts,
            })
            .then(vec![QuerySessionEvent::QueryCancelled {
                query_id,
                reason: Some("User cancelled".to_string()),
                cancelled_at: ts,
            }]);
    }

    #[test]
    fn cancel_executing_query_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
            ])
            .when(QuerySessionCommand::CancelQuery {
                query_id,
                reason: None,
                cancelled_at: ts,
            })
            .then(vec![QuerySessionEvent::QueryCancelled {
                query_id,
                reason: None,
                cancelled_at: ts,
            }]);
    }

    #[test]
    fn cancel_with_wrong_id_fails() {
        let query_id = sample_query_id();
        let wrong_id = QueryId::new();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            }])
            .when(QuerySessionCommand::CancelQuery {
                query_id: wrong_id,
                reason: None,
                cancelled_at: ts,
            })
            .then_error(QuerySessionError::query_id_mismatch(query_id, wrong_id));
    }

    #[test]
    fn cancel_from_idle_fails() {
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![])
            .when(QuerySessionCommand::CancelQuery {
                query_id: sample_query_id(),
                reason: None,
                cancelled_at: ts,
            })
            .then_error(QuerySessionError::no_query_in_progress());
    }

    #[test]
    fn cancel_from_completed_fails() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
                QuerySessionEvent::QueryCompleted {
                    query_id,
                    row_count: 10,
                    duration_ms: 100,
                    completed_at: ts,
                },
            ])
            .when(QuerySessionCommand::CancelQuery {
                query_id,
                reason: None,
                cancelled_at: ts,
            })
            .then_error(QuerySessionError::terminal_state("completed"));
    }

    // --- ResetSession transitions ---

    #[test]
    fn reset_from_completed_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
                QuerySessionEvent::QueryCompleted {
                    query_id,
                    row_count: 10,
                    duration_ms: 100,
                    completed_at: ts,
                },
            ])
            .when(QuerySessionCommand::ResetSession { reset_at: ts })
            .then(vec![QuerySessionEvent::SessionReset { reset_at: ts }]);
    }

    #[test]
    fn reset_from_failed_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
                QuerySessionEvent::QueryFailed {
                    query_id,
                    error: "Some error".to_string(),
                    failed_at: ts,
                },
            ])
            .when(QuerySessionCommand::ResetSession { reset_at: ts })
            .then(vec![QuerySessionEvent::SessionReset { reset_at: ts }]);
    }

    #[test]
    fn reset_from_cancelled_succeeds() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::QueryCancelled {
                    query_id,
                    reason: None,
                    cancelled_at: ts,
                },
            ])
            .when(QuerySessionCommand::ResetSession { reset_at: ts })
            .then(vec![QuerySessionEvent::SessionReset { reset_at: ts }]);
    }

    #[test]
    fn reset_from_idle_is_idempotent() {
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![])
            .when(QuerySessionCommand::ResetSession { reset_at: ts })
            .then(vec![]); // Idempotent: no events
    }

    #[test]
    fn reset_from_pending_fails() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            }])
            .when(QuerySessionCommand::ResetSession { reset_at: ts })
            .then_error(QuerySessionError::invalid_transition(
                "reset session",
                "pending",
            ));
    }

    #[test]
    fn reset_from_executing_fails() {
        let query_id = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![
                QuerySessionEvent::QueryStarted {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: ts,
                },
                QuerySessionEvent::ExecutionBegan {
                    query_id,
                    began_at: ts,
                },
            ])
            .when(QuerySessionCommand::ResetSession { reset_at: ts })
            .then_error(QuerySessionError::invalid_transition(
                "reset session",
                "executing",
            ));
    }

    // --- State evolution: query_count semantics ---

    #[test]
    fn query_count_increments_on_completion() {
        let query_id = sample_query_id();
        let ts = sample_time();

        // Start with empty events
        let state = QuerySessionState::default();
        assert_eq!(state.query_count, 0);

        // Apply QueryStarted
        let state = evolve(
            &state,
            &QuerySessionEvent::QueryStarted {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            },
        );
        assert_eq!(state.query_count, 0);

        // Apply ExecutionBegan
        let state = evolve(
            &state,
            &QuerySessionEvent::ExecutionBegan {
                query_id,
                began_at: ts,
            },
        );
        assert_eq!(state.query_count, 0);

        // Apply QueryCompleted - should increment
        let state = evolve(
            &state,
            &QuerySessionEvent::QueryCompleted {
                query_id,
                row_count: 10,
                duration_ms: 100,
                completed_at: ts,
            },
        );
        assert_eq!(state.query_count, 1);
    }

    #[test]
    fn query_count_increments_on_failure() {
        let query_id = sample_query_id();
        let ts = sample_time();

        // Start from executing state
        let state = QuerySessionState {
            status: QuerySessionStatus::Executing {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
                began_at: ts,
            },
            query_count: 5,
        };

        // Apply QueryFailed - should increment
        let state = evolve(
            &state,
            &QuerySessionEvent::QueryFailed {
                query_id,
                error: "Error".to_string(),
                failed_at: ts,
            },
        );
        assert_eq!(state.query_count, 6);
    }

    #[test]
    fn query_count_preserved_on_cancel() {
        let query_id = sample_query_id();
        let ts = sample_time();

        // Start from pending state
        let state = QuerySessionState {
            status: QuerySessionStatus::Pending {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            },
            query_count: 3,
        };

        // Apply QueryCancelled - should NOT increment
        let state = evolve(
            &state,
            &QuerySessionEvent::QueryCancelled {
                query_id,
                reason: None,
                cancelled_at: ts,
            },
        );
        assert_eq!(state.query_count, 3);
    }

    #[test]
    fn query_count_preserved_on_reset() {
        let query_id = sample_query_id();
        let ts = sample_time();

        // Start from completed state with count
        let state = QuerySessionState {
            status: QuerySessionStatus::Completed {
                query_id,
                row_count: 10,
                duration_ms: 100,
                completed_at: ts,
            },
            query_count: 7,
        };

        // Apply SessionReset - count preserved
        let state = evolve(&state, &QuerySessionEvent::SessionReset { reset_at: ts });
        assert_eq!(state.query_count, 7);
        assert!(state.is_idle());
    }

    // --- Full lifecycle ---

    #[test]
    fn full_success_lifecycle() {
        let query_id = sample_query_id();
        let ts = sample_time();

        let mut state = QuerySessionState::default();

        // Start query
        let events = decide(
            &QuerySessionCommand::StartQuery {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);
        state = evolve(&state, &events[0]);
        assert!(state.is_in_progress());

        // Begin execution
        let events = decide(
            &QuerySessionCommand::BeginExecution {
                query_id,
                began_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);
        state = evolve(&state, &events[0]);
        assert!(matches!(
            state.status,
            QuerySessionStatus::Executing { .. }
        ));

        // Complete
        let events = decide(
            &QuerySessionCommand::CompleteQuery {
                query_id,
                row_count: 100,
                duration_ms: 500,
                completed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);
        state = evolve(&state, &events[0]);
        assert!(state.is_terminal());
        assert_eq!(state.query_count, 1);

        // Reset
        let events = decide(
            &QuerySessionCommand::ResetSession { reset_at: ts },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);
        state = evolve(&state, &events[0]);
        assert!(state.is_idle());
        assert_eq!(state.query_count, 1); // Preserved
    }

    #[test]
    fn full_failure_lifecycle() {
        let query_id = sample_query_id();
        let ts = sample_time();

        let mut state = QuerySessionState::default();

        // Start and begin execution
        let events = decide(
            &QuerySessionCommand::StartQuery {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            },
            &state,
        )
        .unwrap();
        state = evolve(&state, &events[0]);

        let events = decide(
            &QuerySessionCommand::BeginExecution {
                query_id,
                began_at: ts,
            },
            &state,
        )
        .unwrap();
        state = evolve(&state, &events[0]);

        // Fail
        let events = decide(
            &QuerySessionCommand::FailQuery {
                query_id,
                error: "Table not found".to_string(),
                failed_at: ts,
            },
            &state,
        )
        .unwrap();
        state = evolve(&state, &events[0]);

        assert!(matches!(state.status, QuerySessionStatus::Failed { .. }));
        assert_eq!(state.query_count, 1);
    }

    #[test]
    fn cancel_lifecycle() {
        let query_id = sample_query_id();
        let ts = sample_time();

        let mut state = QuerySessionState::default();

        // Start query
        let events = decide(
            &QuerySessionCommand::StartQuery {
                query_id,
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: ts,
            },
            &state,
        )
        .unwrap();
        state = evolve(&state, &events[0]);

        // Cancel from pending
        let events = decide(
            &QuerySessionCommand::CancelQuery {
                query_id,
                reason: Some("Changed my mind".to_string()),
                cancelled_at: ts,
            },
            &state,
        )
        .unwrap();
        state = evolve(&state, &events[0]);

        assert!(matches!(
            state.status,
            QuerySessionStatus::Cancelled { .. }
        ));
        assert_eq!(state.query_count, 0); // Not incremented for cancel
    }
}
