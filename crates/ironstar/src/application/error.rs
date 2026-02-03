//! Application layer error types for command handling.
//!
//! The application layer sits between domain and infrastructure, combining
//! domain errors into a single type suitable for command handler return types.
//!
//! # Naming clarification
//!
//! The 'Aggregate' in [`AggregateError`] refers to *error aggregation* (combining
//! multiple validation errors into a single result), not domain aggregates or
//! event sourcing aggregates. This error type works equally well with fmodel-rust's
//! Decider pattern and traditional aggregate patterns.

use crate::common::ErrorCode;
use crate::domain::error::{DomainError, ValidationError};
use std::fmt;

/// Application-level errors combining validation and domain errors.
///
/// This type supports two error propagation styles:
///
/// - **Applicative** (collect all): The `Validation` variant holds a `Vec` to
///   support collecting all field errors before returning, enabling better UX.
/// - **Monadic** (fail fast): The `Domain` variant holds a single error for
///   immediate failure on business rule violations.
///
/// # Examples
///
/// ```rust,ignore
/// use ironstar::application::error::AggregateError;
/// use ironstar::domain::error::{ValidationError, ValidationErrorKind};
///
/// // Collect multiple validation errors
/// let errors = vec![
///     ValidationError::new(ValidationErrorKind::EmptyField { field: "title".to_string() }),
///     ValidationError::new(ValidationErrorKind::EmptyField { field: "description".to_string() }),
/// ];
/// let aggregate_err: AggregateError = errors.into();
/// ```
#[derive(Debug)]
pub enum AggregateError {
    /// Multiple validation errors (applicative style - collect all errors).
    Validation(Vec<ValidationError>),
    /// Single domain logic error (monadic style - fail fast).
    Domain(DomainError),
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(errors) => {
                let messages: Vec<_> = errors.iter().map(ToString::to_string).collect();
                write!(f, "{}", messages.join("; "))
            }
            Self::Domain(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AggregateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Validation(errors) => errors.first().map(|e| -> &dyn std::error::Error { e }),
            Self::Domain(e) => Some(e),
        }
    }
}

impl From<ValidationError> for AggregateError {
    fn from(e: ValidationError) -> Self {
        Self::Validation(vec![e])
    }
}

impl From<Vec<ValidationError>> for AggregateError {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self::Validation(errors)
    }
}

impl From<DomainError> for AggregateError {
    fn from(e: DomainError) -> Self {
        Self::Domain(e)
    }
}

impl AggregateError {
    /// Map to HTTP-compatible error code.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match self {
            Self::Validation(_) => ErrorCode::ValidationFailed,
            Self::Domain(e) => e.error_code(),
        }
    }

    /// Check if this error contains validation errors.
    #[must_use]
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Check if this error is a domain error.
    #[must_use]
    pub fn is_domain(&self) -> bool {
        matches!(self, Self::Domain(_))
    }

    /// Get all validation errors if this is a validation error.
    #[must_use]
    pub fn validation_errors(&self) -> Option<&[ValidationError]> {
        match self {
            Self::Validation(errors) => Some(errors),
            Self::Domain(_) => None,
        }
    }
}

// =============================================================================
// CommandPipelineError: Unified error type for EventSourcedAggregate pipeline
// =============================================================================

use crate::domain::query_session::QuerySessionError;
use crate::domain::todo::TodoError;
use crate::infrastructure::error::InfrastructureError;
use uuid::Uuid;

/// Error type for EventSourcedAggregate command pipeline.
///
/// Unifies domain and infrastructure failures for fmodel-rust type alignment.
/// Each wired aggregate gets an explicit variant rather than a generic fallback,
/// enabling exhaustive pattern matching and precise error handling.
///
/// # Design
///
/// The `EventSourcedAggregate::handle` method requires repository and decider to
/// share the same error type. This enum bridges the gap:
///
/// - Domain errors (from Decider) are mapped via `map_error` before wiring
/// - Infrastructure errors (from EventRepository) are wrapped via From impl
///
/// # UUID tracking
///
/// All variants preserve error_id from the source layer for distributed tracing.
/// The `error_id()` method returns the original error ID, enabling correlation
/// across async operations and service boundaries.
///
/// # Future aggregates
///
/// Add new variants as aggregates are wired:
/// - `Session(SessionError)` for ironstar-507
/// - `Workspace(WorkspaceError)` for ironstar-7a2
#[derive(Debug)]
pub enum CommandPipelineError {
    /// Todo aggregate domain error (preserves UUID from TodoError).
    Todo(TodoError),
    /// QuerySession aggregate domain error (preserves UUID from QuerySessionError).
    QuerySession(QuerySessionError),
    // Session(SessionError),      // future: ironstar-507
    // Workspace(WorkspaceError),  // future: ironstar-7a2
    /// Infrastructure failure (from EventRepository adapter).
    Infrastructure(InfrastructureError),
}

impl CommandPipelineError {
    /// Get the unique error ID for tracing correlation.
    ///
    /// Returns the error_id from the underlying error type, preserving
    /// the original UUID from domain or infrastructure layer.
    #[must_use]
    pub fn error_id(&self) -> Uuid {
        match self {
            Self::Todo(e) => e.error_id(),
            Self::QuerySession(e) => e.error_id(),
            Self::Infrastructure(e) => e.error_id(),
        }
    }
}

impl fmt::Display for CommandPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Todo(e) => write!(f, "Todo: {e}"),
            Self::QuerySession(e) => write!(f, "QuerySession: {e}"),
            Self::Infrastructure(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CommandPipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Todo(e) => Some(e),
            Self::QuerySession(e) => Some(e),
            Self::Infrastructure(e) => Some(e),
        }
    }
}

impl From<InfrastructureError> for CommandPipelineError {
    fn from(e: InfrastructureError) -> Self {
        Self::Infrastructure(e)
    }
}

impl From<TodoError> for CommandPipelineError {
    fn from(e: TodoError) -> Self {
        Self::Todo(e)
    }
}

impl From<QuerySessionError> for CommandPipelineError {
    fn from(e: QuerySessionError) -> Self {
        Self::QuerySession(e)
    }
}

/// Conversion from `&QuerySessionError` for fmodel-rust's `map_error` which passes references.
///
/// Creates a new `QuerySessionError` with the same kind, preserving the error_id from the
/// original. Since `QuerySessionError` contains non-Clone fields (Backtrace), we create a fresh
/// backtrace but preserve the UUID for distributed tracing correlation.
impl From<&QuerySessionError> for CommandPipelineError {
    fn from(e: &QuerySessionError) -> Self {
        Self::QuerySession(QuerySessionError::with_id(
            e.error_id(),
            e.kind().clone(),
        ))
    }
}

/// Conversion from `&TodoError` for fmodel-rust's `map_error` which passes references.
///
/// This creates a new `TodoError` with the same kind, preserving the error_id from the
/// original. Since `TodoError` contains non-Clone fields (Backtrace), we create a fresh
/// backtrace but preserve the UUID for distributed tracing correlation.
impl From<&TodoError> for CommandPipelineError {
    fn from(e: &TodoError) -> Self {
        Self::Todo(TodoError::with_id(e.error_id(), e.kind().clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::{DomainErrorKind, ValidationErrorKind};

    #[test]
    fn from_single_validation_error() {
        let err = ValidationError::new(ValidationErrorKind::EmptyField {
            field: "title".to_string(),
        });
        let agg: AggregateError = err.into();
        assert!(agg.is_validation());
        assert_eq!(agg.validation_errors().unwrap().len(), 1);
    }

    #[test]
    fn from_multiple_validation_errors() {
        let errors = vec![
            ValidationError::new(ValidationErrorKind::EmptyField {
                field: "title".to_string(),
            }),
            ValidationError::new(ValidationErrorKind::EmptyField {
                field: "description".to_string(),
            }),
        ];
        let agg: AggregateError = errors.into();
        assert!(agg.is_validation());
        assert_eq!(agg.validation_errors().unwrap().len(), 2);
    }

    #[test]
    fn from_domain_error() {
        let err = DomainError::new(DomainErrorKind::NotFound {
            aggregate_type: "Todo".to_string(),
            aggregate_id: "123".to_string(),
        });
        let agg: AggregateError = err.into();
        assert!(agg.is_domain());
        assert_eq!(agg.error_code(), ErrorCode::NotFound);
    }

    #[test]
    fn display_multiple_validation_errors() {
        let errors = vec![
            ValidationError::new(ValidationErrorKind::EmptyField {
                field: "title".to_string(),
            }),
            ValidationError::new(ValidationErrorKind::EmptyField {
                field: "body".to_string(),
            }),
        ];
        let agg: AggregateError = errors.into();
        let display = agg.to_string();
        assert!(display.contains("title cannot be empty"));
        assert!(display.contains("body cannot be empty"));
        assert!(display.contains("; "));
    }
}
