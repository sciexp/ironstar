//! Validation errors for analytics value objects.
//!
//! These are validation errors for analytics value objects, not operational
//! errors like query execution failures. Operational errors will be handled
//! by the UUID-correlated AnalyticsError type (see ironstar-2nt.17).
//!
//! # Design principles
//!
//! - **UUID tracking**: Every error instance gets a unique ID for distributed tracing
//! - **Backtrace capture**: Backtraces are captured at error creation for debugging
//! - **Kind enum for pattern matching**: The `AnalyticsValidationErrorKind` enum enables
//!   exhaustive matching without losing UUID/backtrace context

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Validation error for analytics value objects with UUID tracking.
///
/// This struct wraps [`AnalyticsValidationErrorKind`] variants with a unique
/// identifier for distributed tracing correlation and a backtrace for debugging.
#[derive(Debug)]
pub struct AnalyticsValidationError {
    id: Uuid,
    kind: AnalyticsValidationErrorKind,
    backtrace: Backtrace,
}

/// Error variants for analytics validation failures.
///
/// These represent precondition failures when constructing value objects.
/// Each variant maps to a specific validation rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalyticsValidationErrorKind {
    /// SQL query cannot be empty.
    EmptySql,

    /// SQL query exceeds maximum length.
    SqlTooLong { max: usize, actual: usize },

    /// Dataset reference cannot be empty.
    EmptyDatasetRef,

    /// Dataset reference exceeds maximum length.
    DatasetRefTooLong { max: usize, actual: usize },

    /// Dataset reference has invalid format.
    InvalidDatasetRefFormat { reason: &'static str },

    /// Chart configuration is invalid.
    InvalidChartConfig { reason: &'static str },
}

impl AnalyticsValidationError {
    /// Creates a new error with a generated UUID and captured backtrace.
    pub fn new(kind: AnalyticsValidationErrorKind) -> Self {
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
    pub fn kind(&self) -> &AnalyticsValidationErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    // Convenience constructors

    /// Creates an `EmptySql` error.
    pub fn empty_sql() -> Self {
        Self::new(AnalyticsValidationErrorKind::EmptySql)
    }

    /// Creates a `SqlTooLong` error.
    pub fn sql_too_long(max: usize, actual: usize) -> Self {
        Self::new(AnalyticsValidationErrorKind::SqlTooLong { max, actual })
    }

    /// Creates an `EmptyDatasetRef` error.
    pub fn empty_dataset_ref() -> Self {
        Self::new(AnalyticsValidationErrorKind::EmptyDatasetRef)
    }

    /// Creates a `DatasetRefTooLong` error.
    pub fn dataset_ref_too_long(max: usize, actual: usize) -> Self {
        Self::new(AnalyticsValidationErrorKind::DatasetRefTooLong { max, actual })
    }

    /// Creates an `InvalidDatasetRefFormat` error.
    pub fn invalid_dataset_ref_format(reason: &'static str) -> Self {
        Self::new(AnalyticsValidationErrorKind::InvalidDatasetRefFormat { reason })
    }

    /// Creates an `InvalidChartConfig` error.
    pub fn invalid_chart_config(reason: &'static str) -> Self {
        Self::new(AnalyticsValidationErrorKind::InvalidChartConfig { reason })
    }
}

impl fmt::Display for AnalyticsValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AnalyticsValidationErrorKind::EmptySql => write!(f, "SQL query cannot be empty"),
            AnalyticsValidationErrorKind::SqlTooLong { max, actual } => {
                write!(f, "SQL query cannot exceed {max} characters (got {actual})")
            }
            AnalyticsValidationErrorKind::EmptyDatasetRef => {
                write!(f, "dataset reference cannot be empty")
            }
            AnalyticsValidationErrorKind::DatasetRefTooLong { max, actual } => {
                write!(
                    f,
                    "dataset reference cannot exceed {max} characters (got {actual})"
                )
            }
            AnalyticsValidationErrorKind::InvalidDatasetRefFormat { reason } => {
                write!(f, "invalid dataset reference format: {reason}")
            }
            AnalyticsValidationErrorKind::InvalidChartConfig { reason } => {
                write!(f, "invalid chart configuration: {reason}")
            }
        }
    }
}

impl std::error::Error for AnalyticsValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_unique_uuid() {
        let err1 = AnalyticsValidationError::empty_sql();
        let err2 = AnalyticsValidationError::empty_sql();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(
            AnalyticsValidationError::empty_sql().to_string(),
            "SQL query cannot be empty"
        );
        assert_eq!(
            AnalyticsValidationError::sql_too_long(1000, 1500).to_string(),
            "SQL query cannot exceed 1000 characters (got 1500)"
        );
        assert_eq!(
            AnalyticsValidationError::invalid_chart_config("missing series").to_string(),
            "invalid chart configuration: missing series"
        );
    }

    #[test]
    fn error_kind_accessible() {
        let err = AnalyticsValidationError::sql_too_long(100, 150);
        assert_eq!(
            err.kind(),
            &AnalyticsValidationErrorKind::SqlTooLong {
                max: 100,
                actual: 150
            }
        );
    }
}
