//! Session store error types.
//!
//! Crate-specific errors for session persistence operations.
//! The binary crate's `InfrastructureError` provides `From<SessionStoreError>`
//! for unified error handling at the composition root.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

use ironstar_core::ErrorCode;

/// Session store errors from persistence operations with UUID tracking.
#[derive(Debug)]
pub struct SessionStoreError {
    id: Uuid,
    kind: SessionStoreErrorKind,
    backtrace: Backtrace,
}

/// Specific session store failure kinds.
#[derive(Debug)]
pub enum SessionStoreErrorKind {
    /// Database operation failed (sqlx error).
    Database(sqlx::Error),
    /// Database operation failed (message-based).
    DatabaseMessage(String),
    /// JSON serialization/deserialization failed.
    Serialization(serde_json::Error),
}

impl SessionStoreError {
    /// Create a new session store error with automatic UUID and backtrace.
    #[must_use]
    pub fn new(kind: SessionStoreErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Get the unique error ID for tracing correlation.
    #[must_use]
    pub fn error_id(&self) -> Uuid {
        self.id
    }

    /// Get the specific error kind.
    #[must_use]
    pub fn kind(&self) -> &SessionStoreErrorKind {
        &self.kind
    }

    /// Get the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// Map to HTTP-compatible error code.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match &self.kind {
            SessionStoreErrorKind::Database(_) | SessionStoreErrorKind::DatabaseMessage(_) => {
                ErrorCode::DatabaseError
            }
            SessionStoreErrorKind::Serialization(_) => ErrorCode::InternalError,
        }
    }

    /// Create a database error with a message.
    #[must_use]
    pub fn database(message: impl Into<String>) -> Self {
        Self::new(SessionStoreErrorKind::DatabaseMessage(message.into()))
    }
}

impl fmt::Display for SessionStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SessionStoreErrorKind::Database(e) => write!(f, "session store database error: {e}"),
            SessionStoreErrorKind::DatabaseMessage(msg) => {
                write!(f, "session store database error: {msg}")
            }
            SessionStoreErrorKind::Serialization(e) => {
                write!(f, "session store serialization error: {e}")
            }
        }
    }
}

impl std::error::Error for SessionStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            SessionStoreErrorKind::Database(e) => Some(e),
            SessionStoreErrorKind::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for SessionStoreError {
    fn from(e: sqlx::Error) -> Self {
        Self::new(SessionStoreErrorKind::Database(e))
    }
}

impl From<serde_json::Error> for SessionStoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(SessionStoreErrorKind::Serialization(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_unique_id() {
        let err1 = SessionStoreError::database("connection lost");
        let err2 = SessionStoreError::database("connection lost");
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_code_mapping() {
        assert_eq!(
            SessionStoreError::database("test").error_code(),
            ErrorCode::DatabaseError
        );
    }

    #[test]
    fn display_formatting() {
        let err = SessionStoreError::database("timeout");
        assert_eq!(err.to_string(), "session store database error: timeout");
    }
}
