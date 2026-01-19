//! Domain error types for the Session aggregate.
//!
//! Error types in the domain layer represent business rule violations, not
//! infrastructure failures. They are pure data (no side effects) and should
//! be informative enough for the presentation layer to render user-friendly
//! messages.
//!
//! # Design principles
//!
//! - **Specific over generic**: Each aggregate has its own error enum rather
//!   than a catch-all `DomainError`. This enables exhaustive pattern matching.
//! - **No infrastructure concerns**: Database errors, network failures, etc.
//!   belong in the application or infrastructure layer.
//! - **UUID tracking**: All errors include a unique identifier for distributed
//!   tracing correlation across service boundaries.
//! - **Backtrace capture**: Errors capture backtraces at creation for debugging.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Domain error for the Session aggregate with UUID tracking.
///
/// This struct wraps [`SessionErrorKind`] variants with a unique identifier
/// for distributed tracing correlation and a backtrace for debugging.
#[derive(Debug)]
pub struct SessionError {
    id: Uuid,
    kind: SessionErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the Session decider.
///
/// These represent precondition failures when processing commands.
/// Each variant maps to a specific business rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionErrorKind {
    /// Session already exists in an active state.
    AlreadyActive,

    /// No active session exists to perform operation on.
    NoActiveSession,

    /// Session has expired and cannot be operated on.
    SessionExpired,

    /// Session has been invalidated and cannot be operated on.
    SessionInvalidated,
}

impl SessionError {
    /// Creates a new `SessionError` with a generated UUID and captured backtrace.
    pub fn new(kind: SessionErrorKind) -> Self {
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
    pub fn kind(&self) -> &SessionErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    // Convenience constructors for common error variants

    /// Creates an `AlreadyActive` error.
    pub fn already_active() -> Self {
        Self::new(SessionErrorKind::AlreadyActive)
    }

    /// Creates a `NoActiveSession` error.
    pub fn no_active_session() -> Self {
        Self::new(SessionErrorKind::NoActiveSession)
    }

    /// Creates a `SessionExpired` error.
    pub fn session_expired() -> Self {
        Self::new(SessionErrorKind::SessionExpired)
    }

    /// Creates a `SessionInvalidated` error.
    pub fn session_invalidated() -> Self {
        Self::new(SessionErrorKind::SessionInvalidated)
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SessionErrorKind::AlreadyActive => {
                write!(f, "session is already active")
            }
            SessionErrorKind::NoActiveSession => {
                write!(f, "no active session exists")
            }
            SessionErrorKind::SessionExpired => {
                write!(f, "session has expired")
            }
            SessionErrorKind::SessionInvalidated => {
                write!(f, "session has been invalidated")
            }
        }
    }
}

impl PartialEq for SessionError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for SessionError {}

impl std::error::Error for SessionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            SessionError::already_active().to_string(),
            "session is already active"
        );
        assert_eq!(
            SessionError::no_active_session().to_string(),
            "no active session exists"
        );
        assert_eq!(
            SessionError::session_expired().to_string(),
            "session has expired"
        );
        assert_eq!(
            SessionError::session_invalidated().to_string(),
            "session has been invalidated"
        );
    }

    #[test]
    fn error_has_unique_uuid() {
        let err1 = SessionError::already_active();
        let err2 = SessionError::already_active();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_kind_accessible() {
        let err = SessionError::session_expired();
        assert_eq!(err.kind(), &SessionErrorKind::SessionExpired);
    }

    #[test]
    fn errors_with_same_kind_are_equal() {
        let err1 = SessionError::no_active_session();
        let err2 = SessionError::no_active_session();
        assert_eq!(err1, err2);
    }

    #[test]
    fn errors_with_different_kinds_are_not_equal() {
        let err1 = SessionError::already_active();
        let err2 = SessionError::session_expired();
        assert_ne!(err1, err2);
    }
}
