//! Domain error types.
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
//! - **Serializable**: Errors may need to flow through SSE for client display.

use thiserror::Error;

/// Validation errors for the Todo aggregate.
///
/// These represent precondition failures when constructing value objects
/// or processing commands. Each variant maps to a specific business rule.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TodoError {
    /// Todo text cannot be empty.
    #[error("todo text cannot be empty")]
    EmptyText,

    /// Todo text exceeds maximum length.
    #[error("todo text cannot exceed {max} characters (got {actual})")]
    TextTooLong { max: usize, actual: usize },

    /// Attempted to complete an already-completed todo.
    #[error("todo is already completed")]
    AlreadyCompleted,

    /// Attempted to uncomplete a todo that isn't completed.
    #[error("todo is not completed")]
    NotCompleted,

    /// Attempted to operate on a deleted todo.
    #[error("todo has been deleted")]
    Deleted,

    /// Invalid state transition attempted.
    #[error("invalid state transition: cannot {action} when {state}")]
    InvalidTransition {
        action: &'static str,
        state: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            TodoError::EmptyText.to_string(),
            "todo text cannot be empty"
        );
        assert_eq!(
            TodoError::TextTooLong {
                max: 500,
                actual: 512
            }
            .to_string(),
            "todo text cannot exceed 500 characters (got 512)"
        );
        assert_eq!(
            TodoError::InvalidTransition {
                action: "complete",
                state: "deleted"
            }
            .to_string(),
            "invalid state transition: cannot complete when deleted"
        );
    }
}
