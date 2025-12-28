//! Domain errors for the QuerySession aggregate.

use thiserror::Error;

use super::super::analytics::QueryId;

/// Domain errors for the QuerySession aggregate.
///
/// These represent business rule violations during command processing.
/// Validation errors for value object construction are in [`AnalyticsValidationError`].
///
/// [`AnalyticsValidationError`]: super::super::analytics::AnalyticsValidationError
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum QuerySessionError {
    /// Cannot start a query when one is already in progress.
    #[error("query already in progress")]
    QueryAlreadyInProgress,

    /// Cannot operate on a session that has no active query.
    #[error("no query in progress")]
    NoQueryInProgress,

    /// The query ID doesn't match the current query.
    #[error("query ID mismatch: expected {expected}, got {actual}")]
    QueryIdMismatch { expected: QueryId, actual: QueryId },

    /// Cannot modify a query in a terminal state.
    #[error("query is in terminal state: {state}")]
    TerminalState { state: &'static str },

    /// Invalid state transition attempted.
    #[error("invalid state transition: cannot {action} when {state}")]
    InvalidTransition {
        action: &'static str,
        state: &'static str,
    },
}
