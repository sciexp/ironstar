//! Validation errors for analytics value objects.

use thiserror::Error;

/// Validation errors for analytics value objects.
///
/// These represent precondition failures when constructing value objects.
/// Each variant maps to a specific validation rule. Note that these are
/// *validation* errors, not operational errors like query execution failures.
/// Operational errors will be handled by the UUID-correlated AnalyticsError
/// type (see ironstar-2nt.17).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AnalyticsValidationError {
    /// SQL query cannot be empty.
    #[error("SQL query cannot be empty")]
    EmptySql,

    /// SQL query exceeds maximum length.
    #[error("SQL query cannot exceed {max} characters (got {actual})")]
    SqlTooLong { max: usize, actual: usize },

    /// Dataset reference cannot be empty.
    #[error("dataset reference cannot be empty")]
    EmptyDatasetRef,

    /// Dataset reference exceeds maximum length.
    #[error("dataset reference cannot exceed {max} characters (got {actual})")]
    DatasetRefTooLong { max: usize, actual: usize },

    /// Dataset reference has invalid format.
    #[error("invalid dataset reference format: {reason}")]
    InvalidDatasetRefFormat { reason: &'static str },

    /// Chart configuration is invalid.
    #[error("invalid chart configuration: {reason}")]
    InvalidChartConfig { reason: &'static str },
}
