//! Error types for analytics domain.
//!
//! This module provides two error types for different error categories:
//!
//! - [`AnalyticsValidationError`]: Input validation failures when constructing value objects
//! - [`AnalyticsError`]: Operational errors during query execution, caching, timeouts, etc.
//!
//! # Design principles
//!
//! - **UUID tracking**: Every error instance gets a unique ID for distributed tracing
//! - **Backtrace capture**: Backtraces are captured at error creation for debugging
//! - **Kind enum for pattern matching**: Error kind enums enable exhaustive matching
//!   without losing UUID/backtrace context
//! - **Error composition**: `AnalyticsError` can wrap `AnalyticsValidationError` for
//!   unified error handling in workflows

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

// ============================================================================
// AnalyticsError - Operational errors
// ============================================================================

/// Operational error for analytics workflows with UUID tracking.
///
/// Unlike [`AnalyticsValidationError`] which represents input validation failures,
/// this type represents runtime operational errors: query execution failures,
/// cache errors, timeouts, resource exhaustion, etc.
///
/// Each error instance carries a unique UUID for distributed tracing correlation
/// and a backtrace for debugging.
#[derive(Debug)]
pub struct AnalyticsError {
    id: Uuid,
    kind: AnalyticsErrorKind,
    backtrace: Backtrace,
}

/// Operational error variants for analytics workflows.
///
/// These represent runtime failures during analytics operations, as opposed
/// to validation errors which occur during value object construction.
#[derive(Debug)]
pub enum AnalyticsErrorKind {
    /// Query execution failed.
    QueryExecution { message: String },

    /// Cache operation failed.
    Cache { message: String },

    /// Query timed out.
    Timeout { query_id: Uuid, elapsed_ms: u64 },

    /// Resource exhausted (memory, connections, etc.).
    ResourceExhausted { resource: String },

    /// Validation error (wraps AnalyticsValidationError).
    Validation(AnalyticsValidationError),
}

impl AnalyticsError {
    /// Creates a new error with a generated UUID and captured backtrace.
    pub fn new(kind: AnalyticsErrorKind) -> Self {
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
    pub fn kind(&self) -> &AnalyticsErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    // Convenience constructors

    /// Creates a `QueryExecution` error.
    pub fn query_execution(message: impl Into<String>) -> Self {
        Self::new(AnalyticsErrorKind::QueryExecution {
            message: message.into(),
        })
    }

    /// Creates a `Cache` error.
    pub fn cache(message: impl Into<String>) -> Self {
        Self::new(AnalyticsErrorKind::Cache {
            message: message.into(),
        })
    }

    /// Creates a `Timeout` error.
    pub fn timeout(query_id: Uuid, elapsed_ms: u64) -> Self {
        Self::new(AnalyticsErrorKind::Timeout {
            query_id,
            elapsed_ms,
        })
    }

    /// Creates a `ResourceExhausted` error.
    pub fn resource_exhausted(resource: impl Into<String>) -> Self {
        Self::new(AnalyticsErrorKind::ResourceExhausted {
            resource: resource.into(),
        })
    }

    /// Creates a `Validation` error wrapping an `AnalyticsValidationError`.
    pub fn validation(err: AnalyticsValidationError) -> Self {
        Self::new(AnalyticsErrorKind::Validation(err))
    }
}

impl fmt::Display for AnalyticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AnalyticsErrorKind::QueryExecution { message } => {
                write!(f, "query execution failed: {message}")
            }
            AnalyticsErrorKind::Cache { message } => {
                write!(f, "cache operation failed: {message}")
            }
            AnalyticsErrorKind::Timeout {
                query_id,
                elapsed_ms,
            } => {
                write!(f, "query {query_id} timed out after {elapsed_ms}ms")
            }
            AnalyticsErrorKind::ResourceExhausted { resource } => {
                write!(f, "resource exhausted: {resource}")
            }
            AnalyticsErrorKind::Validation(err) => {
                write!(f, "validation error: {err}")
            }
        }
    }
}

impl std::error::Error for AnalyticsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            AnalyticsErrorKind::Validation(err) => Some(err),
            _ => None,
        }
    }
}

impl From<AnalyticsValidationError> for AnalyticsError {
    fn from(err: AnalyticsValidationError) -> Self {
        Self::validation(err)
    }
}

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

    // AnalyticsError tests

    #[test]
    fn operational_error_has_unique_uuid() {
        let err1 = AnalyticsError::query_execution("failed");
        let err2 = AnalyticsError::query_execution("failed");
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn operational_error_display_messages() {
        assert_eq!(
            AnalyticsError::query_execution("syntax error").to_string(),
            "query execution failed: syntax error"
        );
        assert_eq!(
            AnalyticsError::cache("connection lost").to_string(),
            "cache operation failed: connection lost"
        );

        let query_id = Uuid::new_v4();
        let timeout_err = AnalyticsError::timeout(query_id, 5000);
        assert_eq!(
            timeout_err.to_string(),
            format!("query {query_id} timed out after 5000ms")
        );

        assert_eq!(
            AnalyticsError::resource_exhausted("memory").to_string(),
            "resource exhausted: memory"
        );

        let validation_err = AnalyticsValidationError::empty_sql();
        let wrapped = AnalyticsError::validation(validation_err);
        assert_eq!(
            wrapped.to_string(),
            "validation error: SQL query cannot be empty"
        );
    }

    #[test]
    fn operational_error_kind_accessible() {
        let err = AnalyticsError::timeout(Uuid::new_v4(), 3000);
        assert!(matches!(
            err.kind(),
            AnalyticsErrorKind::Timeout { elapsed_ms: 3000, .. }
        ));
    }

    #[test]
    fn validation_error_converts_to_operational() {
        let validation_err = AnalyticsValidationError::empty_sql();
        let operational: AnalyticsError = validation_err.into();
        assert!(matches!(
            operational.kind(),
            AnalyticsErrorKind::Validation(_)
        ));
    }

    #[test]
    fn operational_error_source_returns_validation_error() {
        use std::error::Error;

        let validation_err = AnalyticsValidationError::empty_sql();
        let operational = AnalyticsError::validation(validation_err);

        assert!(operational.source().is_some());
        if let Some(source) = operational.source() {
            assert!(source.to_string().contains("SQL query cannot be empty"));
        }

        // Non-validation errors should return None
        let query_err = AnalyticsError::query_execution("failed");
        assert!(query_err.source().is_none());
    }
}
