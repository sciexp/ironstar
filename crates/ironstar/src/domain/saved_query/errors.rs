//! Domain error types for the SavedQuery aggregate.
//!
//! Error types represent business rule violations in the saved query
//! lifecycle. Infrastructure failures belong in the application layer.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Domain error for the SavedQuery aggregate with UUID tracking.
#[derive(Debug)]
pub struct SavedQueryError {
    id: Uuid,
    kind: SavedQueryErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the SavedQuery decider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavedQueryErrorKind {
    /// A saved query already exists with this ID.
    AlreadyExists,

    /// No saved query exists with this ID.
    NotFound,
}

impl SavedQueryError {
    /// Creates a new error with a generated UUID and captured backtrace.
    pub fn new(kind: SavedQueryErrorKind) -> Self {
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
    pub fn kind(&self) -> &SavedQueryErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    pub fn already_exists() -> Self {
        Self::new(SavedQueryErrorKind::AlreadyExists)
    }

    pub fn not_found() -> Self {
        Self::new(SavedQueryErrorKind::NotFound)
    }
}

impl fmt::Display for SavedQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SavedQueryErrorKind::AlreadyExists => {
                write!(f, "saved query already exists")
            }
            SavedQueryErrorKind::NotFound => {
                write!(f, "saved query not found")
            }
        }
    }
}

impl PartialEq for SavedQueryError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for SavedQueryError {}

impl std::error::Error for SavedQueryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            SavedQueryError::already_exists().to_string(),
            "saved query already exists"
        );
        assert_eq!(
            SavedQueryError::not_found().to_string(),
            "saved query not found"
        );
    }

    #[test]
    fn error_has_unique_uuid() {
        let err1 = SavedQueryError::not_found();
        let err2 = SavedQueryError::not_found();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_kind_accessible() {
        let err = SavedQueryError::already_exists();
        assert_eq!(err.kind(), &SavedQueryErrorKind::AlreadyExists);
    }
}
