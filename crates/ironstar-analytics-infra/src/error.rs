//! Analytics infrastructure error types.
//!
//! Crate-specific errors for DuckDB analytics, moka caching, and rkyv serialization.
//! The binary crate's `InfrastructureError` provides `From<AnalyticsInfraError>`
//! for unified error handling at the composition root.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

use ironstar_core::ErrorCode;

/// Analytics infrastructure errors with UUID tracking.
#[derive(Debug)]
pub struct AnalyticsInfraError {
    id: Uuid,
    kind: AnalyticsInfraErrorKind,
    backtrace: Backtrace,
}

/// Specific analytics infrastructure failure kinds.
#[derive(Debug)]
pub enum AnalyticsInfraErrorKind {
    /// DuckDB analytics query failed.
    Analytics(String),
    /// Moka cache operation failed.
    Cache(String),
    /// rkyv serialization/deserialization failed.
    Serialization(String),
    /// Resource not found in analytics infrastructure.
    NotFound { resource: String, id: String },
}

impl AnalyticsInfraError {
    /// Create a new analytics infrastructure error with automatic UUID and backtrace.
    #[must_use]
    pub fn new(kind: AnalyticsInfraErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Create a DuckDB analytics error.
    #[must_use]
    pub fn analytics(message: impl Into<String>) -> Self {
        Self::new(AnalyticsInfraErrorKind::Analytics(message.into()))
    }

    /// Create a cache error.
    #[must_use]
    pub fn cache(message: impl Into<String>) -> Self {
        Self::new(AnalyticsInfraErrorKind::Cache(message.into()))
    }

    /// Create a serialization error.
    #[must_use]
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::new(AnalyticsInfraErrorKind::Serialization(message.into()))
    }

    /// Create a not found error.
    #[must_use]
    pub fn not_found(resource: impl Into<String>, id: impl Into<String>) -> Self {
        Self::new(AnalyticsInfraErrorKind::NotFound {
            resource: resource.into(),
            id: id.into(),
        })
    }

    /// Get the unique error ID for tracing correlation.
    #[must_use]
    pub fn error_id(&self) -> Uuid {
        self.id
    }

    /// Get the specific error kind.
    #[must_use]
    pub fn kind(&self) -> &AnalyticsInfraErrorKind {
        &self.kind
    }

    /// Get the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// Map to HTTP-compatible error code.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match &self.kind {
            AnalyticsInfraErrorKind::Analytics(_) => ErrorCode::ServiceUnavailable,
            AnalyticsInfraErrorKind::Cache(_) => ErrorCode::InternalError,
            AnalyticsInfraErrorKind::Serialization(_) => ErrorCode::InternalError,
            AnalyticsInfraErrorKind::NotFound { .. } => ErrorCode::NotFound,
        }
    }
}

impl fmt::Display for AnalyticsInfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AnalyticsInfraErrorKind::Analytics(msg) => write!(f, "analytics error: {msg}"),
            AnalyticsInfraErrorKind::Cache(msg) => write!(f, "cache error: {msg}"),
            AnalyticsInfraErrorKind::Serialization(msg) => {
                write!(f, "serialization error: {msg}")
            }
            AnalyticsInfraErrorKind::NotFound { resource, id } => {
                write!(f, "{resource} {id} not found")
            }
        }
    }
}

impl std::error::Error for AnalyticsInfraError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_unique_id() {
        let err1 = AnalyticsInfraError::analytics("query timeout");
        let err2 = AnalyticsInfraError::analytics("query timeout");
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_code_mapping() {
        assert_eq!(
            AnalyticsInfraError::analytics("test").error_code(),
            ErrorCode::ServiceUnavailable
        );
        assert_eq!(
            AnalyticsInfraError::cache("test").error_code(),
            ErrorCode::InternalError
        );
        assert_eq!(
            AnalyticsInfraError::serialization("test").error_code(),
            ErrorCode::InternalError
        );
        assert_eq!(
            AnalyticsInfraError::not_found("Event", "123").error_code(),
            ErrorCode::NotFound
        );
    }

    #[test]
    fn display_formatting() {
        let err = AnalyticsInfraError::analytics("query timeout");
        assert_eq!(err.to_string(), "analytics error: query timeout");

        let err = AnalyticsInfraError::cache("eviction failed");
        assert_eq!(err.to_string(), "cache error: eviction failed");

        let err = AnalyticsInfraError::serialization("rkyv failed");
        assert_eq!(err.to_string(), "serialization error: rkyv failed");

        let err = AnalyticsInfraError::not_found("Catalog", "space");
        assert_eq!(err.to_string(), "Catalog space not found");
    }
}
