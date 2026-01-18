//! Domain errors for the QuerySession aggregate.
//!
//! These are business rule violations during command processing, not validation
//! errors for value object construction (those are in [`AnalyticsValidationError`]).
//!
//! # Design principles
//!
//! - **UUID tracking**: Every error instance gets a unique ID for distributed tracing
//! - **Backtrace capture**: Backtraces are captured at error creation for debugging
//! - **Kind enum for pattern matching**: The `QuerySessionErrorKind` enum enables
//!   exhaustive matching without losing UUID/backtrace context
//!
//! [`AnalyticsValidationError`]: super::super::analytics::AnalyticsValidationError

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

use super::super::analytics::QueryId;

/// Domain error for the QuerySession aggregate with UUID tracking.
///
/// This struct wraps [`QuerySessionErrorKind`] variants with a unique identifier
/// for distributed tracing correlation and a backtrace for debugging.
#[derive(Debug)]
pub struct QuerySessionError {
    id: Uuid,
    kind: QuerySessionErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the QuerySession aggregate.
///
/// These represent business rule violations during command processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuerySessionErrorKind {
    /// Cannot start a query when one is already in progress.
    QueryAlreadyInProgress,

    /// Cannot operate on a session that has no active query.
    NoQueryInProgress,

    /// The query ID doesn't match the current query.
    QueryIdMismatch { expected: QueryId, actual: QueryId },

    /// Cannot modify a query in a terminal state.
    TerminalState { state: &'static str },

    /// Invalid state transition attempted.
    InvalidTransition {
        action: &'static str,
        state: &'static str,
    },
}

impl QuerySessionError {
    /// Creates a new error with a generated UUID and captured backtrace.
    pub fn new(kind: QuerySessionErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Returns the unique error identifier for distributed tracing.
    pub fn error_id(&self) -> Uuid {
        self.id
    }

    /// Returns a reference to the error kind.
    pub fn kind(&self) -> &QuerySessionErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    // Convenience constructors

    /// Creates a `QueryAlreadyInProgress` error.
    pub fn query_already_in_progress() -> Self {
        Self::new(QuerySessionErrorKind::QueryAlreadyInProgress)
    }

    /// Creates a `NoQueryInProgress` error.
    pub fn no_query_in_progress() -> Self {
        Self::new(QuerySessionErrorKind::NoQueryInProgress)
    }

    /// Creates a `QueryIdMismatch` error.
    pub fn query_id_mismatch(expected: QueryId, actual: QueryId) -> Self {
        Self::new(QuerySessionErrorKind::QueryIdMismatch { expected, actual })
    }

    /// Creates a `TerminalState` error.
    pub fn terminal_state(state: &'static str) -> Self {
        Self::new(QuerySessionErrorKind::TerminalState { state })
    }

    /// Creates an `InvalidTransition` error.
    pub fn invalid_transition(action: &'static str, state: &'static str) -> Self {
        Self::new(QuerySessionErrorKind::InvalidTransition { action, state })
    }
}

impl fmt::Display for QuerySessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            QuerySessionErrorKind::QueryAlreadyInProgress => {
                write!(f, "query already in progress")
            }
            QuerySessionErrorKind::NoQueryInProgress => write!(f, "no query in progress"),
            QuerySessionErrorKind::QueryIdMismatch { expected, actual } => {
                write!(f, "query ID mismatch: expected {expected}, got {actual}")
            }
            QuerySessionErrorKind::TerminalState { state } => {
                write!(f, "query is in terminal state: {state}")
            }
            QuerySessionErrorKind::InvalidTransition { action, state } => {
                write!(f, "invalid state transition: cannot {action} when {state}")
            }
        }
    }
}

impl std::error::Error for QuerySessionError {}

/// PartialEq compares errors by kind only, ignoring UUID and backtrace.
/// This enables testing with DeciderTestSpecification::then_error().
impl PartialEq for QuerySessionError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_unique_uuid() {
        let err1 = QuerySessionError::no_query_in_progress();
        let err2 = QuerySessionError::no_query_in_progress();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(
            QuerySessionError::query_already_in_progress().to_string(),
            "query already in progress"
        );
        assert_eq!(
            QuerySessionError::terminal_state("completed").to_string(),
            "query is in terminal state: completed"
        );
        assert_eq!(
            QuerySessionError::invalid_transition("cancel", "completed").to_string(),
            "invalid state transition: cannot cancel when completed"
        );
    }

    #[test]
    fn error_kind_accessible() {
        let err = QuerySessionError::terminal_state("failed");
        assert_eq!(
            err.kind(),
            &QuerySessionErrorKind::TerminalState { state: "failed" }
        );
    }
}
