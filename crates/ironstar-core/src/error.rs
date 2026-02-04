//! Domain layer error types with UUID tracking for distributed tracing.
//!
//! These are pure domain errors with no infrastructure dependencies. Every error
//! includes a unique identifier for correlation across logs and traces.
//!
//! # Error types
//!
//! - [`ValidationError`]: Field-level validation failures (e.g., empty field, invalid format)
//! - [`DomainError`]: Business rule violations (e.g., invalid state transition, version conflict)
//!
//! # Design principles
//!
//! - **Pure types**: No side effects, no infrastructure dependencies
//! - **UUID tracking**: Every error instance gets a unique ID for tracing
//! - **Backtrace capture**: Backtraces are captured at error creation for debugging
//! - **Informative messages**: Display implementations provide user-friendly messages

use crate::error_code::ErrorCode;
use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Field-level validation errors with UUID tracking.
///
/// Use this type when validating user input at the domain boundary. Each variant
/// captures specific validation failure details that enable informative error messages.
///
/// # Examples
///
/// ```rust,ignore
/// use ironstar_core::error::{ValidationError, ValidationErrorKind};
///
/// let error = ValidationError::new(ValidationErrorKind::EmptyField {
///     field: "title".to_string(),
/// });
/// assert!(!error.error_id().is_nil());
/// ```
#[derive(Debug)]
pub struct ValidationError {
    id: Uuid,
    kind: ValidationErrorKind,
    backtrace: Backtrace,
}

/// Specific validation failure kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    /// Required field is empty or whitespace-only.
    EmptyField { field: String },
    /// Field value doesn't match expected format.
    InvalidFormat { field: String, expected: String },
    /// Numeric value outside allowed range.
    OutOfRange {
        field: String,
        min: i64,
        max: i64,
        actual: i64,
    },
    /// String exceeds maximum length.
    TooLong {
        field: String,
        max_length: usize,
        actual_length: usize,
    },
    /// String shorter than minimum length.
    TooShort {
        field: String,
        min_length: usize,
        actual_length: usize,
    },
}

impl ValidationError {
    /// Create a new validation error with automatic UUID and backtrace.
    #[must_use]
    pub fn new(kind: ValidationErrorKind) -> Self {
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

    /// Get the specific validation failure kind.
    #[must_use]
    pub fn kind(&self) -> &ValidationErrorKind {
        &self.kind
    }

    /// Get the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ValidationErrorKind::EmptyField { field } => write!(f, "{field} cannot be empty"),
            ValidationErrorKind::InvalidFormat { field, expected } => {
                write!(f, "{field} has invalid format, expected: {expected}")
            }
            ValidationErrorKind::OutOfRange {
                field,
                min,
                max,
                actual,
            } => write!(f, "{field} must be between {min} and {max}, got {actual}"),
            ValidationErrorKind::TooLong {
                field,
                max_length,
                actual_length,
            } => write!(
                f,
                "{field} exceeds maximum length {max_length} (got {actual_length})"
            ),
            ValidationErrorKind::TooShort {
                field,
                min_length,
                actual_length,
            } => write!(
                f,
                "{field} is shorter than minimum length {min_length} (got {actual_length})"
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Business rule violation errors with UUID tracking.
///
/// Use this type for domain-level invariant violations that aren't simple field
/// validation (those belong in [`ValidationError`]). Domain errors represent
/// failures in business logic, state transitions, or aggregate invariants.
///
/// # Examples
///
/// ```rust,ignore
/// use ironstar_core::error::{DomainError, DomainErrorKind};
///
/// let error = DomainError::new(DomainErrorKind::InvalidTransition {
///     from: "active".to_string(),
///     to: "active".to_string(),
/// });
/// ```
#[derive(Debug)]
pub struct DomainError {
    id: Uuid,
    kind: DomainErrorKind,
    backtrace: Backtrace,
}

/// Specific domain error kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainErrorKind {
    /// State transition is not allowed.
    InvalidTransition { from: String, to: String },
    /// Operation requires more funds than available.
    InsufficientFunds { available: i64, requested: i64 },
    /// Aggregate with this ID already exists.
    AlreadyExists {
        aggregate_type: String,
        aggregate_id: String,
    },
    /// Aggregate with this ID was not found.
    NotFound {
        aggregate_type: String,
        aggregate_id: String,
    },
    /// Expected version doesn't match actual version (optimistic locking).
    VersionConflict { expected: i64, actual: i64 },
}

impl DomainError {
    /// Create a new domain error with automatic UUID and backtrace.
    #[must_use]
    pub fn new(kind: DomainErrorKind) -> Self {
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

    /// Get the specific domain error kind.
    #[must_use]
    pub fn kind(&self) -> &DomainErrorKind {
        &self.kind
    }

    /// Get the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// Map domain error kind to HTTP-compatible error code.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match &self.kind {
            DomainErrorKind::InvalidTransition { .. } => ErrorCode::ValidationFailed,
            DomainErrorKind::InsufficientFunds { .. } => ErrorCode::ValidationFailed,
            DomainErrorKind::AlreadyExists { .. } => ErrorCode::Conflict,
            DomainErrorKind::NotFound { .. } => ErrorCode::NotFound,
            DomainErrorKind::VersionConflict { .. } => ErrorCode::Conflict,
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DomainErrorKind::InvalidTransition { from, to } => {
                write!(f, "cannot transition from {from} to {to}")
            }
            DomainErrorKind::InsufficientFunds {
                available,
                requested,
            } => {
                write!(
                    f,
                    "insufficient funds: {available} available, {requested} requested"
                )
            }
            DomainErrorKind::AlreadyExists {
                aggregate_type,
                aggregate_id,
            } => write!(f, "{aggregate_type} {aggregate_id} already exists"),
            DomainErrorKind::NotFound {
                aggregate_type,
                aggregate_id,
            } => write!(f, "{aggregate_type} {aggregate_id} not found"),
            DomainErrorKind::VersionConflict { expected, actual } => {
                write!(f, "version conflict: expected {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_has_unique_id() {
        let err1 = ValidationError::new(ValidationErrorKind::EmptyField {
            field: "title".to_string(),
        });
        let err2 = ValidationError::new(ValidationErrorKind::EmptyField {
            field: "title".to_string(),
        });
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn validation_error_display() {
        let err = ValidationError::new(ValidationErrorKind::TooLong {
            field: "description".to_string(),
            max_length: 100,
            actual_length: 150,
        });
        assert_eq!(
            err.to_string(),
            "description exceeds maximum length 100 (got 150)"
        );
    }

    #[test]
    fn domain_error_has_unique_id() {
        let err1 = DomainError::new(DomainErrorKind::NotFound {
            aggregate_type: "Todo".to_string(),
            aggregate_id: "123".to_string(),
        });
        let err2 = DomainError::new(DomainErrorKind::NotFound {
            aggregate_type: "Todo".to_string(),
            aggregate_id: "123".to_string(),
        });
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn domain_error_code_mapping() {
        assert_eq!(
            DomainError::new(DomainErrorKind::NotFound {
                aggregate_type: "Todo".to_string(),
                aggregate_id: "123".to_string(),
            })
            .error_code(),
            ErrorCode::NotFound
        );
        assert_eq!(
            DomainError::new(DomainErrorKind::VersionConflict {
                expected: 1,
                actual: 2
            })
            .error_code(),
            ErrorCode::Conflict
        );
    }
}
