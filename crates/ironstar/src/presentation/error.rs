//! Presentation layer error types with HTTP response integration.
//!
//! This module defines [`AppError`], the top-level error type used at HTTP
//! boundaries. It unifies all error categories from lower layers and implements
//! axum's [`IntoResponse`] for automatic HTTP response generation.
//!
//! # Error flow
//!
//! ```text
//! ValidationError ──┐
//!                   ├──> AggregateError ──┐
//! DomainError ──────┘                     ├──> AppError ──> HTTP Response
//!                                         │
//! InfrastructureError ────────────────────┘
//! ```
//!
//! # HTTP response format
//!
//! All errors are serialized to JSON with consistent structure:
//!
//! ```json
//! {
//!   "code": "VALIDATION_FAILED",
//!   "message": "title cannot be empty",
//!   "errorId": "550e8400-e29b-41d4-a716-446655440000"
//! }
//! ```

use crate::application::error::{AggregateError, CommandPipelineError};
use crate::common::ErrorCode;
use crate::domain::error::{DomainError, DomainErrorKind, ValidationError, ValidationErrorKind};
use crate::domain::todo::TodoErrorKind;
use crate::infrastructure::error::InfrastructureError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Top-level application error unifying all error categories with UUID tracking.
///
/// This is the error type returned from HTTP handlers. It captures errors from
/// all lower layers and converts them to appropriate HTTP responses.
///
/// # From implementations
///
/// Errors can be converted from any lower layer using `?` or `.into()`:
///
/// ```rust,ignore
/// async fn create_todo(
///     State(services): State<AppServices>,
///     Json(cmd): Json<CreateTodoRequest>,
/// ) -> Result<impl IntoResponse, AppError> {
///     let events = services.handle_create_todo(cmd).await?; // AggregateError -> AppError
///     Ok(Json(events))
/// }
/// ```
#[derive(Debug)]
pub struct AppError {
    id: Uuid,
    kind: AppErrorKind,
    backtrace: Backtrace,
}

/// Specific application error kinds.
#[derive(Debug)]
pub enum AppErrorKind {
    /// Validation error from domain layer.
    Validation(ValidationError),
    /// Business rule violation from domain layer.
    Domain(DomainError),
    /// Infrastructure failure (database, cache, event bus).
    Infrastructure(InfrastructureError),
    /// Resource not found at presentation layer.
    NotFound { resource: String, id: String },
}

impl AppError {
    /// Create a new application error with automatic UUID and backtrace.
    #[must_use]
    pub fn new(kind: AppErrorKind) -> Self {
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

    /// Get the specific error kind.
    #[must_use]
    pub fn kind(&self) -> &AppErrorKind {
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
            AppErrorKind::Validation(_) => ErrorCode::ValidationFailed,
            AppErrorKind::Domain(e) => e.error_code(),
            AppErrorKind::Infrastructure(e) => e.error_code(),
            AppErrorKind::NotFound { .. } => ErrorCode::NotFound,
        }
    }

    /// Convert to HTTP status code.
    #[must_use]
    pub fn http_status(&self) -> StatusCode {
        StatusCode::from_u16(self.error_code().http_status())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Convert to JSON error response.
    #[must_use]
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            error_id: self.id,
            details: None,
        }
    }

    /// Create a not found error.
    #[must_use]
    pub fn not_found(resource: impl Into<String>, id: impl Into<String>) -> Self {
        Self::new(AppErrorKind::NotFound {
            resource: resource.into(),
            id: id.into(),
        })
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AppErrorKind::Validation(e) => write!(f, "{e}"),
            AppErrorKind::Domain(e) => write!(f, "{e}"),
            AppErrorKind::Infrastructure(e) => write!(f, "{e}"),
            AppErrorKind::NotFound { resource, id } => write!(f, "{resource} {id} not found"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            AppErrorKind::Validation(e) => Some(e),
            AppErrorKind::Domain(e) => Some(e),
            AppErrorKind::Infrastructure(e) => Some(e),
            AppErrorKind::NotFound { .. } => None,
        }
    }
}

/// JSON error response structure with error ID for correlation.
///
/// This structure is serialized to JSON for HTTP error responses. The `errorId`
/// field enables clients to report errors with correlation IDs for debugging.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// HTTP-compatible error code.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Unique error ID for tracing correlation.
    pub error_id: Uuid,
    /// Optional additional details (field errors, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

// Conversions from lower layers

impl From<ValidationError> for AppError {
    fn from(e: ValidationError) -> Self {
        Self::new(AppErrorKind::Validation(e))
    }
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        Self::new(AppErrorKind::Domain(e))
    }
}

impl From<AggregateError> for AppError {
    fn from(e: AggregateError) -> Self {
        match e {
            // Take first validation error for AppError
            // Future: extend AppError to hold Vec<ValidationError> if needed
            AggregateError::Validation(mut v) => {
                // SAFETY: AggregateError::Validation is only constructed with non-empty Vec
                // via From<ValidationError> which creates vec![e] or From<Vec<ValidationError>>
                // which should not be called with empty vec (caller invariant)
                #[allow(clippy::expect_used)]
                let first = v.pop().expect("validation errors non-empty");
                Self::new(AppErrorKind::Validation(first))
            }
            AggregateError::Domain(d) => Self::new(AppErrorKind::Domain(d)),
        }
    }
}

impl From<InfrastructureError> for AppError {
    fn from(e: InfrastructureError) -> Self {
        Self::new(AppErrorKind::Infrastructure(e))
    }
}

impl From<CommandPipelineError> for AppError {
    fn from(e: CommandPipelineError) -> Self {
        match e {
            CommandPipelineError::Todo(kind) => {
                // Map TodoErrorKind to HTTP-semantic AppErrorKind
                match kind {
                    // Validation-like errors → ValidationError
                    TodoErrorKind::EmptyText => Self::new(AppErrorKind::Validation(
                        ValidationError::new(ValidationErrorKind::EmptyField {
                            field: "text".to_string(),
                        }),
                    )),
                    TodoErrorKind::TextTooLong { max, actual } => {
                        Self::new(AppErrorKind::Validation(ValidationError::new(
                            ValidationErrorKind::TooLong {
                                field: "text".to_string(),
                                max_length: max,
                                actual_length: actual,
                            },
                        )))
                    }
                    // NotFound → DomainError::NotFound
                    TodoErrorKind::NotFound => Self::new(AppErrorKind::Domain(DomainError::new(
                        DomainErrorKind::NotFound {
                            aggregate_type: "Todo".to_string(),
                            aggregate_id: "unknown".to_string(),
                        },
                    ))),
                    // State conflict errors → DomainError with appropriate kind
                    TodoErrorKind::AlreadyExists => {
                        Self::new(AppErrorKind::Domain(DomainError::new(
                            DomainErrorKind::AlreadyExists {
                                aggregate_type: "Todo".to_string(),
                                aggregate_id: "unknown".to_string(),
                            },
                        )))
                    }
                    // State transition errors → DomainError::InvalidTransition
                    TodoErrorKind::CannotComplete
                    | TodoErrorKind::CannotUncomplete
                    | TodoErrorKind::CannotDelete
                    | TodoErrorKind::AlreadyCompleted
                    | TodoErrorKind::NotCompleted
                    | TodoErrorKind::Deleted
                    | TodoErrorKind::InvalidTransition { .. } => {
                        Self::new(AppErrorKind::Domain(DomainError::new(
                            DomainErrorKind::InvalidTransition {
                                from: "current".to_string(),
                                to: format!("{kind}"),
                            },
                        )))
                    }
                }
            }
            CommandPipelineError::Infrastructure(infra) => {
                // Delegate to existing InfrastructureError mapping
                Self::from(infra)
            }
        }
    }
}

// Axum IntoResponse integration

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.http_status();
        let body = Json(self.to_response());
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::{DomainErrorKind, ValidationErrorKind};

    #[test]
    fn app_error_has_unique_id() {
        let err1 = AppError::not_found("Todo", "123");
        let err2 = AppError::not_found("Todo", "123");
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn from_validation_error() {
        let validation = ValidationError::new(ValidationErrorKind::EmptyField {
            field: "title".to_string(),
        });
        let app: AppError = validation.into();
        assert_eq!(app.error_code(), ErrorCode::ValidationFailed);
        assert_eq!(app.http_status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn from_domain_error() {
        let domain = DomainError::new(DomainErrorKind::NotFound {
            aggregate_type: "Todo".to_string(),
            aggregate_id: "abc".to_string(),
        });
        let app: AppError = domain.into();
        assert_eq!(app.error_code(), ErrorCode::NotFound);
        assert_eq!(app.http_status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_response_serialization() {
        let err = AppError::not_found("Todo", "123");
        let response = err.to_response();
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"code\":\"NOT_FOUND\""));
        assert!(json.contains("\"errorId\""));
        assert!(json.contains("Todo 123 not found"));
    }
}
