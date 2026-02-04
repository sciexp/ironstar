//! Infrastructure layer error types with UUID tracking.
//!
//! Infrastructure errors represent failures in external systems: databases,
//! caches, message buses, and serialization. Each error captures a unique ID
//! for distributed tracing correlation.
//!
//! # Design notes
//!
//! Infrastructure errors wrap underlying library errors (sqlx, serde_json) while
//! adding UUID tracking and backtrace capture. The `From` implementations enable
//! ergonomic error propagation with the `?` operator.

use crate::common::ErrorCode;
use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Infrastructure errors from persistence, cache, or event bus with UUID tracking.
///
/// This type wraps errors from external dependencies while adding tracing support.
/// The backtrace is captured at error creation time for debugging.
///
/// # Examples
///
/// ```rust,ignore
/// use ironstar::infrastructure::error::{InfrastructureError, InfrastructureErrorKind};
///
/// // Create from sqlx error
/// async fn load_events(pool: &sqlx::SqlitePool) -> Result<Vec<Event>, InfrastructureError> {
///     sqlx::query_as("SELECT * FROM events")
///         .fetch_all(pool)
///         .await
///         .map_err(InfrastructureError::from)
/// }
/// ```
#[derive(Debug)]
pub struct InfrastructureError {
    id: Uuid,
    kind: InfrastructureErrorKind,
    backtrace: Backtrace,
}

/// Specific infrastructure failure kinds.
#[derive(Debug)]
pub enum InfrastructureErrorKind {
    /// Database operation failed (sqlx error).
    Database(sqlx::Error),
    /// Database operation failed (message-based).
    DatabaseMessage(String),
    /// JSON serialization/deserialization failed.
    Serialization(serde_json::Error),
    /// Event bus operation failed.
    EventBus(String),
    /// Cache operation failed.
    Cache(String),
    /// Analytics query failed (DuckDB).
    Analytics(String),
    /// Resource not found in infrastructure layer.
    NotFound { resource: String, id: String },
    /// Optimistic locking conflict - concurrent modification detected.
    OptimisticLockingConflict {
        aggregate_type: String,
        aggregate_id: String,
    },
}

impl InfrastructureError {
    /// Create a new infrastructure error with automatic UUID and backtrace.
    #[must_use]
    pub fn new(kind: InfrastructureErrorKind) -> Self {
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

    /// Get the specific infrastructure error kind.
    #[must_use]
    pub fn kind(&self) -> &InfrastructureErrorKind {
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
            InfrastructureErrorKind::Database(_) | InfrastructureErrorKind::DatabaseMessage(_) => {
                ErrorCode::DatabaseError
            }
            InfrastructureErrorKind::Serialization(_) => ErrorCode::InternalError,
            InfrastructureErrorKind::EventBus(_) => ErrorCode::ServiceUnavailable,
            InfrastructureErrorKind::Cache(_) => ErrorCode::InternalError,
            InfrastructureErrorKind::Analytics(_) => ErrorCode::ServiceUnavailable,
            InfrastructureErrorKind::NotFound { .. } => ErrorCode::NotFound,
            InfrastructureErrorKind::OptimisticLockingConflict { .. } => ErrorCode::Conflict,
        }
    }

    /// Create a database error with a message.
    #[must_use]
    pub fn database(message: impl Into<String>) -> Self {
        Self::new(InfrastructureErrorKind::DatabaseMessage(message.into()))
    }

    /// Create an event bus error.
    #[must_use]
    pub fn event_bus(message: impl Into<String>) -> Self {
        Self::new(InfrastructureErrorKind::EventBus(message.into()))
    }

    /// Create a cache error.
    #[must_use]
    pub fn cache(message: impl Into<String>) -> Self {
        Self::new(InfrastructureErrorKind::Cache(message.into()))
    }

    /// Create an analytics error.
    #[must_use]
    pub fn analytics(message: impl Into<String>) -> Self {
        Self::new(InfrastructureErrorKind::Analytics(message.into()))
    }

    /// Create a not found error.
    #[must_use]
    pub fn not_found(resource: impl Into<String>, id: impl Into<String>) -> Self {
        Self::new(InfrastructureErrorKind::NotFound {
            resource: resource.into(),
            id: id.into(),
        })
    }

    /// Create an optimistic locking conflict error.
    #[must_use]
    pub fn optimistic_locking_conflict(
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
    ) -> Self {
        Self::new(InfrastructureErrorKind::OptimisticLockingConflict {
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
        })
    }
}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            InfrastructureErrorKind::Database(e) => write!(f, "database error: {e}"),
            InfrastructureErrorKind::DatabaseMessage(msg) => write!(f, "database error: {msg}"),
            InfrastructureErrorKind::Serialization(e) => write!(f, "serialization error: {e}"),
            InfrastructureErrorKind::EventBus(msg) => write!(f, "event bus error: {msg}"),
            InfrastructureErrorKind::Cache(msg) => write!(f, "cache error: {msg}"),
            InfrastructureErrorKind::Analytics(msg) => write!(f, "analytics error: {msg}"),
            InfrastructureErrorKind::NotFound { resource, id } => {
                write!(f, "{resource} {id} not found")
            }
            InfrastructureErrorKind::OptimisticLockingConflict {
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

impl std::error::Error for InfrastructureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            InfrastructureErrorKind::Database(e) => Some(e),
            InfrastructureErrorKind::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for InfrastructureError {
    fn from(e: sqlx::Error) -> Self {
        Self::new(InfrastructureErrorKind::Database(e))
    }
}

impl From<serde_json::Error> for InfrastructureError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(InfrastructureErrorKind::Serialization(e))
    }
}

impl From<ironstar_event_store::EventStoreError> for InfrastructureError {
    fn from(e: ironstar_event_store::EventStoreError) -> Self {
        match e.kind() {
            ironstar_event_store::EventStoreErrorKind::Database(_)
            | ironstar_event_store::EventStoreErrorKind::DatabaseMessage(_) => {
                Self::database(e.to_string())
            }
            ironstar_event_store::EventStoreErrorKind::Serialization(_) => {
                Self::new(InfrastructureErrorKind::DatabaseMessage(e.to_string()))
            }
            ironstar_event_store::EventStoreErrorKind::OptimisticLockingConflict {
                aggregate_type,
                aggregate_id,
            } => Self::optimistic_locking_conflict(aggregate_type, aggregate_id),
        }
    }
}

impl From<ironstar_event_bus::EventBusError> for InfrastructureError {
    fn from(e: ironstar_event_bus::EventBusError) -> Self {
        Self::event_bus(e.to_string())
    }
}

impl From<ironstar_session_store::SessionStoreError> for InfrastructureError {
    fn from(e: ironstar_session_store::SessionStoreError) -> Self {
        match e.kind() {
            ironstar_session_store::SessionStoreErrorKind::Database(_) => {
                Self::database(e.to_string())
            }
            ironstar_session_store::SessionStoreErrorKind::DatabaseMessage(_) => {
                Self::database(e.to_string())
            }
            ironstar_session_store::SessionStoreErrorKind::Serialization(_) => {
                Self::new(InfrastructureErrorKind::DatabaseMessage(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_unique_id() {
        let err1 = InfrastructureError::event_bus("connection lost");
        let err2 = InfrastructureError::event_bus("connection lost");
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_code_mapping() {
        assert_eq!(
            InfrastructureError::event_bus("test").error_code(),
            ErrorCode::ServiceUnavailable
        );
        assert_eq!(
            InfrastructureError::cache("test").error_code(),
            ErrorCode::InternalError
        );
        assert_eq!(
            InfrastructureError::analytics("test").error_code(),
            ErrorCode::ServiceUnavailable
        );
        assert_eq!(
            InfrastructureError::not_found("Event", "123").error_code(),
            ErrorCode::NotFound
        );
    }

    #[test]
    fn display_formatting() {
        let err = InfrastructureError::not_found("Event", "abc-123");
        assert_eq!(err.to_string(), "Event abc-123 not found");

        let err = InfrastructureError::event_bus("zenoh connection failed");
        assert_eq!(err.to_string(), "event bus error: zenoh connection failed");

        let err = InfrastructureError::analytics("query timeout");
        assert_eq!(err.to_string(), "analytics error: query timeout");
    }
}
