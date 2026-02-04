//! HTTP-compatible error codes shared across all layers.

use serde::{Deserialize, Serialize};

/// HTTP-compatible error codes for API responses.
///
/// These codes provide a stable interface for clients to handle errors
/// programmatically, independent of error message text. Each code maps
/// to a specific HTTP status code range (4xx for client errors, 5xx for
/// server errors).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // 4xx Client errors
    /// Request validation failed (malformed input, constraint violations).
    ValidationFailed,
    /// Input is syntactically valid but semantically incorrect.
    InvalidInput,
    /// Requested resource does not exist.
    NotFound,
    /// Operation conflicts with current resource state.
    Conflict,
    /// Authentication required or credentials invalid.
    Unauthorized,
    /// Authenticated but not authorized for this operation.
    Forbidden,

    // 5xx Server errors
    /// Unexpected server error.
    InternalError,
    /// Database operation failed.
    DatabaseError,
    /// Required service is temporarily unavailable.
    ServiceUnavailable,
}

impl ErrorCode {
    /// Convert to HTTP status code.
    #[must_use]
    pub const fn http_status(&self) -> u16 {
        match self {
            Self::ValidationFailed => 400,
            Self::InvalidInput => 400,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::InternalError => 500,
            Self::DatabaseError => 500,
            Self::ServiceUnavailable => 503,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_status_codes() {
        assert_eq!(ErrorCode::ValidationFailed.http_status(), 400);
        assert_eq!(ErrorCode::NotFound.http_status(), 404);
        assert_eq!(ErrorCode::Conflict.http_status(), 409);
        assert_eq!(ErrorCode::Unauthorized.http_status(), 401);
        assert_eq!(ErrorCode::Forbidden.http_status(), 403);
        assert_eq!(ErrorCode::InternalError.http_status(), 500);
        assert_eq!(ErrorCode::DatabaseError.http_status(), 500);
        assert_eq!(ErrorCode::ServiceUnavailable.http_status(), 503);
    }

    #[test]
    fn serde_serialization() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::ValidationFailed).unwrap(),
            "\"VALIDATION_FAILED\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::DatabaseError).unwrap(),
            "\"DATABASE_ERROR\""
        );
    }
}
