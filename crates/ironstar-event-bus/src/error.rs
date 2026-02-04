//! Event bus error types.
//!
//! Crate-specific errors for event bus operations.
//! The binary crate's `InfrastructureError` provides `From<EventBusError>`
//! for unified error handling at the composition root.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

use ironstar_core::ErrorCode;

/// Event bus errors from Zenoh publish/subscribe operations with UUID tracking.
#[derive(Debug)]
pub struct EventBusError {
    id: Uuid,
    kind: EventBusErrorKind,
    backtrace: Backtrace,
}

/// Specific event bus failure kinds.
#[derive(Debug)]
pub enum EventBusErrorKind {
    /// Event bus operation failed (zenoh error).
    EventBus(String),
    /// JSON serialization failed.
    Serialization(serde_json::Error),
}

impl EventBusError {
    /// Create a new event bus error with automatic UUID and backtrace.
    #[must_use]
    pub fn new(kind: EventBusErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Create an event bus error from a message.
    #[must_use]
    pub fn event_bus(message: impl Into<String>) -> Self {
        Self::new(EventBusErrorKind::EventBus(message.into()))
    }

    /// Get the unique error ID for tracing correlation.
    #[must_use]
    pub fn error_id(&self) -> Uuid {
        self.id
    }

    /// Get the specific error kind.
    #[must_use]
    pub fn kind(&self) -> &EventBusErrorKind {
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
            EventBusErrorKind::EventBus(_) => ErrorCode::ServiceUnavailable,
            EventBusErrorKind::Serialization(_) => ErrorCode::InternalError,
        }
    }
}

impl fmt::Display for EventBusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            EventBusErrorKind::EventBus(msg) => write!(f, "event bus error: {msg}"),
            EventBusErrorKind::Serialization(e) => {
                write!(f, "event bus serialization error: {e}")
            }
        }
    }
}

impl std::error::Error for EventBusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            EventBusErrorKind::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for EventBusError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(EventBusErrorKind::Serialization(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_unique_id() {
        let err1 = EventBusError::event_bus("connection lost");
        let err2 = EventBusError::event_bus("connection lost");
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_code_mapping() {
        assert_eq!(
            EventBusError::event_bus("test").error_code(),
            ErrorCode::ServiceUnavailable
        );
    }

    #[test]
    fn display_formatting() {
        let err = EventBusError::event_bus("zenoh connection failed");
        assert_eq!(err.to_string(), "event bus error: zenoh connection failed");
    }
}
