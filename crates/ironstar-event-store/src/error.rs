//! Event store error types.
//!
//! Crate-specific errors for event persistence operations.
//! The binary crate's `InfrastructureError` provides `From<EventStoreError>`
//! for unified error handling at the composition root.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

use ironstar_core::ErrorCode;

/// Event store errors from persistence operations with UUID tracking.
#[derive(Debug)]
pub struct EventStoreError {
    id: Uuid,
    kind: EventStoreErrorKind,
    backtrace: Backtrace,
}

/// Specific event store failure kinds.
#[derive(Debug)]
pub enum EventStoreErrorKind {
    /// Database operation failed (sqlx error).
    Database(sqlx::Error),
    /// Database operation failed (message-based).
    DatabaseMessage(String),
    /// JSON serialization/deserialization failed.
    Serialization(serde_json::Error),
    /// Optimistic locking conflict - concurrent modification detected.
    OptimisticLockingConflict {
        aggregate_type: String,
        aggregate_id: String,
    },
}

impl EventStoreError {
    /// Create a new event store error with automatic UUID and backtrace.
    #[must_use]
    pub fn new(kind: EventStoreErrorKind) -> Self {
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
    pub fn kind(&self) -> &EventStoreErrorKind {
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
            EventStoreErrorKind::Database(_) | EventStoreErrorKind::DatabaseMessage(_) => {
                ErrorCode::DatabaseError
            }
            EventStoreErrorKind::Serialization(_) => ErrorCode::InternalError,
            EventStoreErrorKind::OptimisticLockingConflict { .. } => ErrorCode::Conflict,
        }
    }

    /// Create a database error with a message.
    #[must_use]
    pub fn database(message: impl Into<String>) -> Self {
        Self::new(EventStoreErrorKind::DatabaseMessage(message.into()))
    }

    /// Create an optimistic locking conflict error.
    #[must_use]
    pub fn optimistic_locking_conflict(
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
    ) -> Self {
        Self::new(EventStoreErrorKind::OptimisticLockingConflict {
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
        })
    }
}

impl fmt::Display for EventStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            EventStoreErrorKind::Database(e) => write!(f, "event store database error: {e}"),
            EventStoreErrorKind::DatabaseMessage(msg) => {
                write!(f, "event store database error: {msg}")
            }
            EventStoreErrorKind::Serialization(e) => {
                write!(f, "event store serialization error: {e}")
            }
            EventStoreErrorKind::OptimisticLockingConflict {
                aggregate_type,
                aggregate_id,
            } => {
                write!(
                    f,
                    "optimistic locking conflict for {aggregate_type}/{aggregate_id}"
                )
            }
        }
    }
}

impl std::error::Error for EventStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            EventStoreErrorKind::Database(e) => Some(e),
            EventStoreErrorKind::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for EventStoreError {
    fn from(e: sqlx::Error) -> Self {
        Self::new(EventStoreErrorKind::Database(e))
    }
}

impl From<serde_json::Error> for EventStoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(EventStoreErrorKind::Serialization(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_unique_id() {
        let err1 = EventStoreError::database("connection lost");
        let err2 = EventStoreError::database("connection lost");
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_code_mapping() {
        assert_eq!(
            EventStoreError::database("test").error_code(),
            ErrorCode::DatabaseError
        );
        assert_eq!(
            EventStoreError::optimistic_locking_conflict("Todo", "123").error_code(),
            ErrorCode::Conflict
        );
    }

    #[test]
    fn display_formatting() {
        let err = EventStoreError::optimistic_locking_conflict("Todo", "todo-123");
        assert_eq!(
            err.to_string(),
            "optimistic locking conflict for Todo/todo-123"
        );
    }

    #[test]
    fn optimistic_locking_conflict_fields() {
        let err = EventStoreError::optimistic_locking_conflict("Todo", "todo-123");
        match err.kind() {
            EventStoreErrorKind::OptimisticLockingConflict {
                aggregate_type,
                aggregate_id,
            } => {
                assert_eq!(aggregate_type, "Todo");
                assert_eq!(aggregate_id, "todo-123");
            }
            _ => panic!("Expected OptimisticLockingConflict variant"),
        }
    }
}
