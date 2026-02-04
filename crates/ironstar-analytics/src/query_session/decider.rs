//! Pure QuerySession Decider implementing fmodel-rust patterns.
//!
//! The Decider is the core decision-making component that manages the
//! lifecycle of analytics query sessions. It is a pure function with
//! no side effects: all I/O (timestamps, persistence) happens at boundaries.

use ironstar_core::Decider;

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
pub fn query_session_decider<'a>() -> QuerySessionDecider<'a> {
    Decider {
        decide: Box::new(decide),
        evolve: Box::new(evolve),
        initial_state: Box::new(QuerySessionState::default),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
fn decide(
    command: &QuerySessionCommand,
    state: &QuerySessionState,
) -> Result<Vec<QuerySessionEvent>, QuerySessionError> {
    match command {
        // StartQuery: Idle -> Pending
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
                Err(QuerySessionError::terminal_state(state.status.state_name()))
            }
        }

        // BeginExecution: Pending -> Executing
        QuerySessionCommand::BeginExecution { query_id, began_at } => match &state.status {
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
        },

        // CompleteQuery: Executing -> Completed
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

        // FailQuery: Executing -> Failed
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

        // CancelQuery: Pending/Executing -> Cancelled
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
            _ => Err(QuerySessionError::terminal_state(state.status.state_name())),
        },

        // ResetSession: Terminal -> Idle (idempotent from Idle)
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
fn evolve(state: &QuerySessionState, event: &QuerySessionEvent) -> QuerySessionState {
    match event {
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

        QuerySessionEvent::ExecutionBegan { began_at, .. } => {
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
                state.clone()
            }
        }

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
    use ironstar_core::DeciderTestSpecification;

    use crate::values::{QueryId, SqlQuery};

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
    fn reset_from_idle_is_idempotent() {
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(query_session_decider())
            .given(vec![])
            .when(QuerySessionCommand::ResetSession { reset_at: ts })
            .then(vec![]); // Idempotent: no events
    }

    // --- Full lifecycle ---

    #[test]
    fn full_success_lifecycle() {
        let query_id = sample_query_id();
        let ts = sample_time();

        let mut state = QuerySessionState::default();
        assert_eq!(state.query_count, 0);

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
        assert!(state.is_in_progress());

        let events = decide(
            &QuerySessionCommand::BeginExecution {
                query_id,
                began_at: ts,
            },
            &state,
        )
        .unwrap();
        state = evolve(&state, &events[0]);

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
        state = evolve(&state, &events[0]);
        assert!(state.is_terminal());
        assert_eq!(state.query_count, 1);

        let events = decide(&QuerySessionCommand::ResetSession { reset_at: ts }, &state).unwrap();
        state = evolve(&state, &events[0]);
        assert!(state.is_idle());
        assert_eq!(state.query_count, 1);
    }
}
