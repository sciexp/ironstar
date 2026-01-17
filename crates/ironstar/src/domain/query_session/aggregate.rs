//! QuerySession aggregate implementation.
//!
//! The QuerySession aggregate manages the lifecycle of an analytics query session.
//! Unlike the Todo aggregate (which is synchronous end-to-end), QuerySession
//! demonstrates the **spawn-after-persist** pattern where long-running async
//! work (DuckDB query execution) happens AFTER event persistence.
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
//!      CompleteQuery  FailQuery    CancelQuery
//!           │             │             │
//!           ▼             ▼             ▼
//!      ┌──────────┐  ┌──────────┐  ┌──────────┐
//!      │Completed │  │  Failed  │  │Cancelled │
//!      └──────────┘  └──────────┘  └──────────┘
//!
//!      ────────────── terminal states ──────────────
//! ```
//!
//! # Spawn-After-Persist Pattern
//!
//! 1. User submits query → `StartQuery` command
//! 2. Aggregate emits `QueryStarted` event
//! 3. Application layer persists event to SQLite
//! 4. Application layer spawns async DuckDB task
//! 5. Task completion triggers `CompleteQuery` or `FailQuery` command
//! 6. Aggregate emits corresponding completion event
//!
//! The aggregate remains pure; all async execution happens at boundaries.

use chrono::Utc;

use super::commands::QuerySessionCommand;
use super::errors::QuerySessionError;
use super::events::QuerySessionEvent;
use super::state::{QuerySessionState, QuerySessionStatus};
use crate::domain::aggregate::Aggregate;
use crate::domain::analytics::{ChartConfig, DatasetRef, QueryId, SqlQuery};

/// The QuerySession aggregate.
///
/// Manages the lifecycle of analytics query sessions. This is the entry point
/// for the Aggregate trait implementation. The actual state lives in
/// [`QuerySessionState`].
#[derive(Debug, Default)]
pub struct QuerySessionAggregate;

impl Aggregate for QuerySessionAggregate {
    const NAME: &'static str = "QuerySession";

    type State = QuerySessionState;
    type Command = QuerySessionCommand;
    type Event = QuerySessionEvent;
    type Error = QuerySessionError;

    fn handle_command(
        state: &Self::State,
        cmd: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match cmd {
            QuerySessionCommand::StartQuery {
                query_id,
                sql,
                dataset_ref,
                chart_config,
            } => handle_start_query(state, query_id, sql, dataset_ref, chart_config),

            QuerySessionCommand::BeginExecution { query_id } => {
                handle_begin_execution(state, query_id)
            }

            QuerySessionCommand::CompleteQuery {
                query_id,
                row_count,
                duration_ms,
            } => handle_complete_query(state, query_id, row_count, duration_ms),

            QuerySessionCommand::FailQuery { query_id, error } => {
                handle_fail_query(state, query_id, error)
            }

            QuerySessionCommand::CancelQuery { query_id, reason } => {
                handle_cancel_query(state, query_id, reason)
            }

            QuerySessionCommand::ResetSession => handle_reset_session(state),
        }
    }

    fn apply_event(mut state: Self::State, event: Self::Event) -> Self::State {
        match event {
            QuerySessionEvent::QueryStarted {
                query_id,
                sql,
                dataset_ref,
                chart_config,
                started_at,
            } => {
                state.status = QuerySessionStatus::Pending {
                    query_id,
                    sql,
                    dataset_ref,
                    chart_config,
                    started_at,
                };
            }

            QuerySessionEvent::ExecutionBegan { began_at, .. } => {
                // Transition from Pending to Executing, preserving query data
                if let QuerySessionStatus::Pending {
                    query_id,
                    sql,
                    dataset_ref,
                    chart_config,
                    started_at,
                } = state.status
                {
                    state.status = QuerySessionStatus::Executing {
                        query_id,
                        sql,
                        dataset_ref,
                        chart_config,
                        started_at,
                        began_at,
                    };
                }
            }

            QuerySessionEvent::QueryCompleted {
                query_id,
                row_count,
                duration_ms,
                completed_at,
            } => {
                state.status = QuerySessionStatus::Completed {
                    query_id,
                    row_count,
                    duration_ms,
                    completed_at,
                };
                state.query_count += 1;
            }

            QuerySessionEvent::QueryFailed {
                query_id,
                error,
                failed_at,
            } => {
                state.status = QuerySessionStatus::Failed {
                    query_id,
                    error,
                    failed_at,
                };
                state.query_count += 1;
            }

            QuerySessionEvent::QueryCancelled {
                query_id,
                reason,
                cancelled_at,
            } => {
                state.status = QuerySessionStatus::Cancelled {
                    query_id,
                    reason,
                    cancelled_at,
                };
            }

            QuerySessionEvent::SessionReset { .. } => {
                state.status = QuerySessionStatus::Idle;
            }
        }

        state
    }
}

// ============================================================================
// Command Handlers (private, pure functions)
// ============================================================================

/// Handle StartQuery command.
fn handle_start_query(
    state: &QuerySessionState,
    query_id: QueryId,
    sql: SqlQuery,
    dataset_ref: Option<DatasetRef>,
    chart_config: Option<ChartConfig>,
) -> Result<Vec<QuerySessionEvent>, QuerySessionError> {
    // Must be idle to start a new query
    if !state.is_idle() {
        if state.is_in_progress() {
            return Err(QuerySessionError::query_already_in_progress());
        }
        // Terminal states need reset first
        return Err(QuerySessionError::terminal_state(state.status.state_name()));
    }

    Ok(vec![QuerySessionEvent::QueryStarted {
        query_id,
        sql,
        dataset_ref,
        chart_config,
        started_at: Utc::now(),
    }])
}

/// Handle BeginExecution command.
fn handle_begin_execution(
    state: &QuerySessionState,
    query_id: QueryId,
) -> Result<Vec<QuerySessionEvent>, QuerySessionError> {
    match &state.status {
        QuerySessionStatus::Pending {
            query_id: pending_id,
            ..
        } => {
            if *pending_id != query_id {
                return Err(QuerySessionError::query_id_mismatch(*pending_id, query_id));
            }
            Ok(vec![QuerySessionEvent::ExecutionBegan {
                query_id,
                began_at: Utc::now(),
            }])
        }
        QuerySessionStatus::Idle => Err(QuerySessionError::no_query_in_progress()),
        _ => Err(QuerySessionError::invalid_transition(
            "begin execution",
            state.status.state_name(),
        )),
    }
}

/// Handle CompleteQuery command.
fn handle_complete_query(
    state: &QuerySessionState,
    query_id: QueryId,
    row_count: usize,
    duration_ms: u64,
) -> Result<Vec<QuerySessionEvent>, QuerySessionError> {
    match &state.status {
        QuerySessionStatus::Executing {
            query_id: executing_id,
            ..
        } => {
            if *executing_id != query_id {
                return Err(QuerySessionError::query_id_mismatch(
                    *executing_id,
                    query_id,
                ));
            }
            Ok(vec![QuerySessionEvent::QueryCompleted {
                query_id,
                row_count,
                duration_ms,
                completed_at: Utc::now(),
            }])
        }
        QuerySessionStatus::Idle => Err(QuerySessionError::no_query_in_progress()),
        _ => Err(QuerySessionError::invalid_transition(
            "complete query",
            state.status.state_name(),
        )),
    }
}

/// Handle FailQuery command.
fn handle_fail_query(
    state: &QuerySessionState,
    query_id: QueryId,
    error: String,
) -> Result<Vec<QuerySessionEvent>, QuerySessionError> {
    match &state.status {
        QuerySessionStatus::Executing {
            query_id: executing_id,
            ..
        } => {
            if *executing_id != query_id {
                return Err(QuerySessionError::query_id_mismatch(
                    *executing_id,
                    query_id,
                ));
            }
            Ok(vec![QuerySessionEvent::QueryFailed {
                query_id,
                error,
                failed_at: Utc::now(),
            }])
        }
        QuerySessionStatus::Idle => Err(QuerySessionError::no_query_in_progress()),
        _ => Err(QuerySessionError::invalid_transition(
            "fail query",
            state.status.state_name(),
        )),
    }
}

/// Handle CancelQuery command.
fn handle_cancel_query(
    state: &QuerySessionState,
    query_id: QueryId,
    reason: Option<String>,
) -> Result<Vec<QuerySessionEvent>, QuerySessionError> {
    match &state.status {
        QuerySessionStatus::Pending {
            query_id: pending_id,
            ..
        }
        | QuerySessionStatus::Executing {
            query_id: pending_id,
            ..
        } => {
            if *pending_id != query_id {
                return Err(QuerySessionError::query_id_mismatch(*pending_id, query_id));
            }
            Ok(vec![QuerySessionEvent::QueryCancelled {
                query_id,
                reason,
                cancelled_at: Utc::now(),
            }])
        }
        QuerySessionStatus::Idle => Err(QuerySessionError::no_query_in_progress()),
        _ => Err(QuerySessionError::terminal_state(state.status.state_name())),
    }
}

/// Handle ResetSession command.
fn handle_reset_session(
    state: &QuerySessionState,
) -> Result<Vec<QuerySessionEvent>, QuerySessionError> {
    // Can only reset from terminal states
    if !state.is_terminal() {
        if state.is_idle() {
            // Already idle, no-op (return empty events)
            return Ok(vec![]);
        }
        return Err(QuerySessionError::invalid_transition(
            "reset session",
            state.status.state_name(),
        ));
    }

    Ok(vec![QuerySessionEvent::SessionReset {
        reset_at: Utc::now(),
    }])
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::errors::QuerySessionErrorKind;
    use super::*;
    use crate::domain::aggregate::AggregateRoot;

    fn sample_query_id() -> QueryId {
        QueryId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_sql() -> SqlQuery {
        SqlQuery::new("SELECT * FROM test").unwrap()
    }

    // --- State Machine Tests ---

    mod state_transitions {
        use super::*;

        #[test]
        fn start_query_from_idle_succeeds() {
            let state = QuerySessionState::default();
            let cmd = QuerySessionCommand::StartQuery {
                query_id: sample_query_id(),
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
            };

            let events = QuerySessionAggregate::handle_command(&state, cmd).unwrap();

            assert_eq!(events.len(), 1);
            assert!(matches!(events[0], QuerySessionEvent::QueryStarted { .. }));
        }

        #[test]
        fn start_query_when_pending_fails() {
            let state = QuerySessionState {
                status: QuerySessionStatus::Pending {
                    query_id: sample_query_id(),
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                },
                query_count: 0,
            };
            let cmd = QuerySessionCommand::StartQuery {
                query_id: QueryId::new(), // Different ID
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
            };

            let result = QuerySessionAggregate::handle_command(&state, cmd);

            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().kind(),
                &QuerySessionErrorKind::QueryAlreadyInProgress
            );
        }

        #[test]
        fn begin_execution_from_pending_succeeds() {
            let query_id = sample_query_id();
            let state = QuerySessionState {
                status: QuerySessionStatus::Pending {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                },
                query_count: 0,
            };
            let cmd = QuerySessionCommand::BeginExecution { query_id };

            let events = QuerySessionAggregate::handle_command(&state, cmd).unwrap();

            assert_eq!(events.len(), 1);
            assert!(matches!(
                events[0],
                QuerySessionEvent::ExecutionBegan { .. }
            ));
        }

        #[test]
        fn begin_execution_with_wrong_id_fails() {
            let state = QuerySessionState {
                status: QuerySessionStatus::Pending {
                    query_id: sample_query_id(),
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                },
                query_count: 0,
            };
            let cmd = QuerySessionCommand::BeginExecution {
                query_id: QueryId::new(),
            };

            let result = QuerySessionAggregate::handle_command(&state, cmd);

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                QuerySessionErrorKind::QueryIdMismatch { .. }
            ));
        }

        #[test]
        fn complete_query_from_executing_succeeds() {
            let query_id = sample_query_id();
            let state = QuerySessionState {
                status: QuerySessionStatus::Executing {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                    began_at: Utc::now(),
                },
                query_count: 0,
            };
            let cmd = QuerySessionCommand::CompleteQuery {
                query_id,
                row_count: 100,
                duration_ms: 1500,
            };

            let events = QuerySessionAggregate::handle_command(&state, cmd).unwrap();

            assert_eq!(events.len(), 1);
            assert!(matches!(
                &events[0],
                QuerySessionEvent::QueryCompleted {
                    row_count: 100,
                    duration_ms: 1500,
                    ..
                }
            ));
        }

        #[test]
        fn fail_query_from_executing_succeeds() {
            let query_id = sample_query_id();
            let state = QuerySessionState {
                status: QuerySessionStatus::Executing {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                    began_at: Utc::now(),
                },
                query_count: 0,
            };
            let cmd = QuerySessionCommand::FailQuery {
                query_id,
                error: "Syntax error".to_string(),
            };

            let events = QuerySessionAggregate::handle_command(&state, cmd).unwrap();

            assert_eq!(events.len(), 1);
            assert!(matches!(
                &events[0],
                QuerySessionEvent::QueryFailed { error, .. } if error == "Syntax error"
            ));
        }

        #[test]
        fn cancel_pending_query_succeeds() {
            let query_id = sample_query_id();
            let state = QuerySessionState {
                status: QuerySessionStatus::Pending {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                },
                query_count: 0,
            };
            let cmd = QuerySessionCommand::CancelQuery {
                query_id,
                reason: Some("User cancelled".to_string()),
            };

            let events = QuerySessionAggregate::handle_command(&state, cmd).unwrap();

            assert_eq!(events.len(), 1);
            assert!(matches!(
                events[0],
                QuerySessionEvent::QueryCancelled { .. }
            ));
        }

        #[test]
        fn reset_from_completed_succeeds() {
            let state = QuerySessionState {
                status: QuerySessionStatus::Completed {
                    query_id: sample_query_id(),
                    row_count: 10,
                    duration_ms: 100,
                    completed_at: Utc::now(),
                },
                query_count: 1,
            };
            let cmd = QuerySessionCommand::ResetSession;

            let events = QuerySessionAggregate::handle_command(&state, cmd).unwrap();

            assert_eq!(events.len(), 1);
            assert!(matches!(events[0], QuerySessionEvent::SessionReset { .. }));
        }

        #[test]
        fn reset_from_idle_is_noop() {
            let state = QuerySessionState::default();
            let cmd = QuerySessionCommand::ResetSession;

            let events = QuerySessionAggregate::handle_command(&state, cmd).unwrap();

            // No events emitted for noop
            assert!(events.is_empty());
        }

        #[test]
        fn reset_from_executing_fails() {
            let state = QuerySessionState {
                status: QuerySessionStatus::Executing {
                    query_id: sample_query_id(),
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                    began_at: Utc::now(),
                },
                query_count: 0,
            };
            let cmd = QuerySessionCommand::ResetSession;

            let result = QuerySessionAggregate::handle_command(&state, cmd);

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                QuerySessionErrorKind::InvalidTransition { .. }
            ));
        }
    }

    // --- State Reconstruction Tests ---

    mod apply_events {
        use super::*;

        #[test]
        fn apply_query_started_sets_pending() {
            let event = QuerySessionEvent::QueryStarted {
                query_id: sample_query_id(),
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
                started_at: Utc::now(),
            };

            let state = QuerySessionAggregate::apply_event(QuerySessionState::default(), event);

            assert!(matches!(state.status, QuerySessionStatus::Pending { .. }));
        }

        #[test]
        fn apply_execution_began_transitions_to_executing() {
            let query_id = sample_query_id();
            let initial = QuerySessionState {
                status: QuerySessionStatus::Pending {
                    query_id,
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                },
                query_count: 0,
            };
            let event = QuerySessionEvent::ExecutionBegan {
                query_id,
                began_at: Utc::now(),
            };

            let state = QuerySessionAggregate::apply_event(initial, event);

            assert!(matches!(state.status, QuerySessionStatus::Executing { .. }));
        }

        #[test]
        fn apply_query_completed_increments_count() {
            let initial = QuerySessionState {
                status: QuerySessionStatus::Executing {
                    query_id: sample_query_id(),
                    sql: sample_sql(),
                    dataset_ref: None,
                    chart_config: None,
                    started_at: Utc::now(),
                    began_at: Utc::now(),
                },
                query_count: 5,
            };
            let event = QuerySessionEvent::QueryCompleted {
                query_id: sample_query_id(),
                row_count: 100,
                duration_ms: 500,
                completed_at: Utc::now(),
            };

            let state = QuerySessionAggregate::apply_event(initial, event);

            assert_eq!(state.query_count, 6);
            assert!(matches!(state.status, QuerySessionStatus::Completed { .. }));
        }

        #[test]
        fn apply_session_reset_returns_to_idle() {
            let initial = QuerySessionState {
                status: QuerySessionStatus::Failed {
                    query_id: sample_query_id(),
                    error: "Some error".to_string(),
                    failed_at: Utc::now(),
                },
                query_count: 3,
            };
            let event = QuerySessionEvent::SessionReset {
                reset_at: Utc::now(),
            };

            let state = QuerySessionAggregate::apply_event(initial, event);

            assert!(state.status.is_idle());
            assert_eq!(state.query_count, 3); // Count preserved
        }
    }

    // --- Integration with AggregateRoot ---

    mod aggregate_root_integration {
        use super::*;

        #[test]
        fn full_success_lifecycle() {
            let mut root = AggregateRoot::<QuerySessionAggregate>::new();
            let query_id = QueryId::new();

            // Start query
            let events = root
                .handle(QuerySessionCommand::StartQuery {
                    query_id,
                    sql: SqlQuery::new("SELECT 1").unwrap(),
                    dataset_ref: None,
                    chart_config: None,
                })
                .unwrap();
            root.apply_all(events);
            assert_eq!(root.version(), 1);
            assert!(root.state().is_in_progress());

            // Begin execution
            let events = root
                .handle(QuerySessionCommand::BeginExecution { query_id })
                .unwrap();
            root.apply_all(events);
            assert_eq!(root.version(), 2);
            assert!(matches!(
                root.state().status,
                QuerySessionStatus::Executing { .. }
            ));

            // Complete
            let events = root
                .handle(QuerySessionCommand::CompleteQuery {
                    query_id,
                    row_count: 1,
                    duration_ms: 10,
                })
                .unwrap();
            root.apply_all(events);
            assert_eq!(root.version(), 3);
            assert!(root.state().is_terminal());
            assert_eq!(root.state().query_count, 1);

            // Reset
            let events = root.handle(QuerySessionCommand::ResetSession).unwrap();
            root.apply_all(events);
            assert_eq!(root.version(), 4);
            assert!(root.state().is_idle());
        }

        #[test]
        fn failure_lifecycle() {
            let mut root = AggregateRoot::<QuerySessionAggregate>::new();
            let query_id = QueryId::new();

            // Start and begin execution
            let events = root
                .handle(QuerySessionCommand::StartQuery {
                    query_id,
                    sql: SqlQuery::new("SELECT * FROM missing").unwrap(),
                    dataset_ref: None,
                    chart_config: None,
                })
                .unwrap();
            root.apply_all(events);

            let events = root
                .handle(QuerySessionCommand::BeginExecution { query_id })
                .unwrap();
            root.apply_all(events);

            // Fail
            let events = root
                .handle(QuerySessionCommand::FailQuery {
                    query_id,
                    error: "Table not found: missing".to_string(),
                })
                .unwrap();
            root.apply_all(events);

            assert!(matches!(
                &root.state().status,
                QuerySessionStatus::Failed { error, .. } if error == "Table not found: missing"
            ));
            assert_eq!(root.state().query_count, 1);
        }
    }

    // --- Serialization Tests ---

    mod serialization {
        use super::*;

        #[test]
        fn command_serializes_with_type_tag() {
            let cmd = QuerySessionCommand::StartQuery {
                query_id: sample_query_id(),
                sql: sample_sql(),
                dataset_ref: None,
                chart_config: None,
            };
            let json = serde_json::to_value(&cmd).unwrap();

            assert_eq!(json["type"], "StartQuery");
        }

        #[test]
        fn event_serializes_with_type_tag() {
            let event = QuerySessionEvent::QueryCompleted {
                query_id: sample_query_id(),
                row_count: 42,
                duration_ms: 1000,
                completed_at: Utc::now(),
            };
            let json = serde_json::to_value(&event).unwrap();

            assert_eq!(json["type"], "QueryCompleted");
            assert_eq!(json["row_count"], 42);
        }

        #[test]
        fn command_roundtrips() {
            let cmd = QuerySessionCommand::CompleteQuery {
                query_id: sample_query_id(),
                row_count: 100,
                duration_ms: 2500,
            };
            let json = serde_json::to_string(&cmd).unwrap();
            let parsed: QuerySessionCommand = serde_json::from_str(&json).unwrap();

            assert!(matches!(
                parsed,
                QuerySessionCommand::CompleteQuery { row_count: 100, .. }
            ));
        }
    }

    // --- Error Display Tests ---

    mod error_display {
        use super::*;

        #[test]
        fn error_messages_are_descriptive() {
            assert_eq!(
                QuerySessionError::query_already_in_progress().to_string(),
                "query already in progress"
            );

            assert_eq!(
                QuerySessionError::terminal_state("completed").to_string(),
                "query is in terminal state: completed"
            );

            let mismatch =
                QuerySessionError::query_id_mismatch(sample_query_id(), sample_query_id());
            assert!(mismatch.to_string().contains("query ID mismatch"));
        }
    }
}
