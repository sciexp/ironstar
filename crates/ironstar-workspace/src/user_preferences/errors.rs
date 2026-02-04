//! Domain error types for the UserPreferences aggregate.
//!
//! Error types represent business rule violations in the user preferences
//! lifecycle. Infrastructure failures belong in the application layer.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Domain error for the UserPreferences aggregate with UUID tracking.
#[derive(Debug)]
pub struct UserPreferencesError {
    id: Uuid,
    kind: UserPreferencesErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the UserPreferences decider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserPreferencesErrorKind {
    /// Preferences already initialized for this user.
    AlreadyInitialized,

    /// Preferences not yet initialized (must initialize first).
    NotInitialized,
}

impl UserPreferencesError {
    /// Creates a new error with a generated UUID and captured backtrace.
    pub fn new(kind: UserPreferencesErrorKind) -> Self {
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
    pub fn kind(&self) -> &UserPreferencesErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// Create a new error with an explicit UUID, preserving error tracking.
    #[must_use]
    pub fn with_id(id: Uuid, kind: UserPreferencesErrorKind) -> Self {
        Self {
            id,
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn already_initialized() -> Self {
        Self::new(UserPreferencesErrorKind::AlreadyInitialized)
    }

    pub fn not_initialized() -> Self {
        Self::new(UserPreferencesErrorKind::NotInitialized)
    }
}

impl fmt::Display for UserPreferencesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            UserPreferencesErrorKind::AlreadyInitialized => {
                write!(f, "user preferences already initialized")
            }
            UserPreferencesErrorKind::NotInitialized => {
                write!(f, "user preferences not initialized")
            }
        }
    }
}

impl PartialEq for UserPreferencesError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for UserPreferencesError {}

impl std::error::Error for UserPreferencesError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            UserPreferencesError::already_initialized().to_string(),
            "user preferences already initialized"
        );
        assert_eq!(
            UserPreferencesError::not_initialized().to_string(),
            "user preferences not initialized"
        );
    }

    #[test]
    fn error_has_unique_uuid() {
        let err1 = UserPreferencesError::not_initialized();
        let err2 = UserPreferencesError::not_initialized();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_kind_accessible() {
        let err = UserPreferencesError::already_initialized();
        assert_eq!(err.kind(), &UserPreferencesErrorKind::AlreadyInitialized);
    }
}
