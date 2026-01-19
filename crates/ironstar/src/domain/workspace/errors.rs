//! Domain error types for the Workspace aggregate.
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

/// Domain error for the Workspace aggregate with UUID tracking.
///
/// This struct wraps [`WorkspaceErrorKind`] variants with a unique identifier
/// for distributed tracing correlation and a backtrace for debugging.
#[derive(Debug)]
pub struct WorkspaceError {
    id: Uuid,
    kind: WorkspaceErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the Workspace decider.
///
/// These represent precondition failures when constructing value objects
/// or processing commands. Each variant maps to a specific business rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceErrorKind {
    /// Workspace already exists (create on existing).
    AlreadyExists,

    /// Workspace not found (operation on non-existent).
    NotFound,

    /// Invalid workspace name.
    InvalidName(String),
}

impl WorkspaceError {
    /// Creates a new `WorkspaceError` with a generated UUID and captured backtrace.
    pub fn new(kind: WorkspaceErrorKind) -> Self {
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
    pub fn kind(&self) -> &WorkspaceErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    // Convenience constructors for common error variants

    /// Creates an `AlreadyExists` error.
    pub fn already_exists() -> Self {
        Self::new(WorkspaceErrorKind::AlreadyExists)
    }

    /// Creates a `NotFound` error.
    pub fn not_found() -> Self {
        Self::new(WorkspaceErrorKind::NotFound)
    }

    /// Creates an `InvalidName` error with the given reason.
    pub fn invalid_name(reason: impl Into<String>) -> Self {
        Self::new(WorkspaceErrorKind::InvalidName(reason.into()))
    }
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            WorkspaceErrorKind::AlreadyExists => write!(f, "workspace already exists"),
            WorkspaceErrorKind::NotFound => write!(f, "workspace not found"),
            WorkspaceErrorKind::InvalidName(reason) => write!(f, "invalid workspace name: {reason}"),
        }
    }
}

impl PartialEq for WorkspaceError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for WorkspaceError {}

impl std::error::Error for WorkspaceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            WorkspaceError::already_exists().to_string(),
            "workspace already exists"
        );
        assert_eq!(
            WorkspaceError::not_found().to_string(),
            "workspace not found"
        );
        assert_eq!(
            WorkspaceError::invalid_name("cannot be empty").to_string(),
            "invalid workspace name: cannot be empty"
        );
    }

    #[test]
    fn error_has_unique_uuid() {
        let err1 = WorkspaceError::not_found();
        let err2 = WorkspaceError::not_found();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_kind_accessible() {
        let err = WorkspaceError::invalid_name("too long");
        assert_eq!(
            err.kind(),
            &WorkspaceErrorKind::InvalidName("too long".to_string())
        );
    }
}
