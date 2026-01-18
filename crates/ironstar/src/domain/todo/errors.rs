//! Domain error types for the Todo aggregate.
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

/// Domain error for the Todo aggregate with UUID tracking.
///
/// This struct wraps [`TodoErrorKind`] variants with a unique identifier
/// for distributed tracing correlation and a backtrace for debugging.
#[derive(Debug)]
pub struct TodoError {
    id: Uuid,
    kind: TodoErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the Todo decider.
///
/// These represent precondition failures when constructing value objects
/// or processing commands. Each variant maps to a specific business rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodoErrorKind {
    /// Todo text cannot be empty.
    EmptyText,

    /// Todo text exceeds maximum length.
    TextTooLong { max: usize, actual: usize },

    /// Todo already exists (create on existing).
    AlreadyExists,

    /// Todo not found (operation on non-existent).
    NotFound,

    /// Cannot complete (wrong state).
    CannotComplete,

    /// Cannot uncomplete (wrong state).
    CannotUncomplete,

    /// Cannot delete (wrong state).
    CannotDelete,

    /// Attempted to complete an already-completed todo.
    AlreadyCompleted,

    /// Attempted to uncomplete a todo that isn't completed.
    NotCompleted,

    /// Attempted to operate on a deleted todo.
    Deleted,

    /// Invalid state transition attempted.
    InvalidTransition {
        action: &'static str,
        state: &'static str,
    },
}

impl TodoError {
    /// Creates a new `TodoError` with a generated UUID and captured backtrace.
    pub fn new(kind: TodoErrorKind) -> Self {
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
    pub fn kind(&self) -> &TodoErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    // Convenience constructors for common error variants

    /// Creates an `EmptyText` error.
    pub fn empty_text() -> Self {
        Self::new(TodoErrorKind::EmptyText)
    }

    /// Creates a `TextTooLong` error.
    pub fn text_too_long(max: usize, actual: usize) -> Self {
        Self::new(TodoErrorKind::TextTooLong { max, actual })
    }

    /// Creates an `AlreadyCompleted` error.
    pub fn already_completed() -> Self {
        Self::new(TodoErrorKind::AlreadyCompleted)
    }

    /// Creates a `NotCompleted` error.
    pub fn not_completed() -> Self {
        Self::new(TodoErrorKind::NotCompleted)
    }

    /// Creates a `Deleted` error.
    pub fn deleted() -> Self {
        Self::new(TodoErrorKind::Deleted)
    }

    /// Creates an `InvalidTransition` error.
    pub fn invalid_transition(action: &'static str, state: &'static str) -> Self {
        Self::new(TodoErrorKind::InvalidTransition { action, state })
    }

    /// Creates an `AlreadyExists` error.
    pub fn already_exists() -> Self {
        Self::new(TodoErrorKind::AlreadyExists)
    }

    /// Creates a `NotFound` error.
    pub fn not_found() -> Self {
        Self::new(TodoErrorKind::NotFound)
    }

    /// Creates a `CannotComplete` error.
    pub fn cannot_complete() -> Self {
        Self::new(TodoErrorKind::CannotComplete)
    }

    /// Creates a `CannotUncomplete` error.
    pub fn cannot_uncomplete() -> Self {
        Self::new(TodoErrorKind::CannotUncomplete)
    }

    /// Creates a `CannotDelete` error.
    pub fn cannot_delete() -> Self {
        Self::new(TodoErrorKind::CannotDelete)
    }
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TodoErrorKind::EmptyText => write!(f, "todo text cannot be empty"),
            TodoErrorKind::TextTooLong { max, actual } => {
                write!(f, "todo text cannot exceed {max} characters (got {actual})")
            }
            TodoErrorKind::AlreadyExists => write!(f, "todo already exists"),
            TodoErrorKind::NotFound => write!(f, "todo not found"),
            TodoErrorKind::CannotComplete => {
                write!(f, "cannot complete todo: invalid state")
            }
            TodoErrorKind::CannotUncomplete => {
                write!(f, "cannot uncomplete todo: invalid state")
            }
            TodoErrorKind::CannotDelete => {
                write!(f, "cannot delete todo: invalid state")
            }
            TodoErrorKind::AlreadyCompleted => write!(f, "todo is already completed"),
            TodoErrorKind::NotCompleted => write!(f, "todo is not completed"),
            TodoErrorKind::Deleted => write!(f, "todo has been deleted"),
            TodoErrorKind::InvalidTransition { action, state } => {
                write!(f, "invalid state transition: cannot {action} when {state}")
            }
        }
    }
}

impl PartialEq for TodoError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for TodoError {}

impl std::error::Error for TodoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            TodoError::empty_text().to_string(),
            "todo text cannot be empty"
        );
        assert_eq!(
            TodoError::text_too_long(500, 512).to_string(),
            "todo text cannot exceed 500 characters (got 512)"
        );
        assert_eq!(
            TodoError::invalid_transition("complete", "deleted").to_string(),
            "invalid state transition: cannot complete when deleted"
        );
    }

    #[test]
    fn error_has_unique_uuid() {
        let err1 = TodoError::empty_text();
        let err2 = TodoError::empty_text();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_kind_accessible() {
        let err = TodoError::text_too_long(100, 150);
        assert_eq!(
            err.kind(),
            &TodoErrorKind::TextTooLong {
                max: 100,
                actual: 150
            }
        );
    }
}
