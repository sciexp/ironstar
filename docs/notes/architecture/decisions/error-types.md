# Error type definitions

This document defines the concrete error types used across ironstar's layered architecture.
For design principles, CQRS propagation patterns, and migration strategy, see `error-handling-decisions.md`.

## Error type hierarchy overview

Ironstar uses a layered error type hierarchy where each layer has its own error type that converts to the layer above.
The hierarchy enables type-safe error propagation across CQRS boundaries.

```
ValidationError ──┐
                  ├──> AggregateError ──┐
DomainError ──────┘                     ├──> AppError ──> HTTP Response
                                        │
InfrastructureError ────────────────────┘
```

## Foundation layer: ErrorCode

HTTP-compatible error codes shared across all layers.

```rust
// common-enums/src/lib.rs
use serde::{Deserialize, Serialize};

/// HTTP-compatible error codes for API responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // 4xx Client errors
    ValidationFailed,
    InvalidInput,
    NotFound,
    Conflict,
    Unauthorized,
    Forbidden,

    // 5xx Server errors
    InternalError,
    DatabaseError,
    ServiceUnavailable,
}

impl ErrorCode {
    /// Convert to HTTP status code
    pub fn http_status(&self) -> u16 {
        match self {
            ErrorCode::ValidationFailed => 400,
            ErrorCode::InvalidInput => 400,
            ErrorCode::NotFound => 404,
            ErrorCode::Conflict => 409,
            ErrorCode::Unauthorized => 401,
            ErrorCode::Forbidden => 403,
            ErrorCode::InternalError => 500,
            ErrorCode::DatabaseError => 500,
            ErrorCode::ServiceUnavailable => 503,
        }
    }
}
```

## Domain layer: ValidationError and DomainError

Pure domain errors with no infrastructure dependencies.
All domain errors include UUID tracking for distributed tracing correlation.

### ValidationError

Field-level validation failures with UUID tracking.

```rust
// ironstar-domain/src/error.rs
use common_enums::ErrorCode;
use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Domain-level validation errors with UUID tracking
#[derive(Debug)]
pub struct ValidationError {
    id: Uuid,
    kind: ValidationErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    EmptyField { field: String },
    InvalidFormat { field: String, expected: String },
    OutOfRange { field: String, min: i64, max: i64, actual: i64 },
    TooLong { field: String, max_length: usize, actual_length: usize },
    TooShort { field: String, min_length: usize, actual_length: usize },
}

impl ValidationError {
    pub fn new(kind: ValidationErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn error_id(&self) -> Uuid {
        self.id
    }

    pub fn kind(&self) -> &ValidationErrorKind {
        &self.kind
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ValidationErrorKind::EmptyField { field } =>
                write!(f, "{} cannot be empty", field),
            ValidationErrorKind::InvalidFormat { field, expected } =>
                write!(f, "{} has invalid format, expected: {}", field, expected),
            ValidationErrorKind::OutOfRange { field, min, max, actual } =>
                write!(f, "{} must be between {} and {}, got {}", field, min, max, actual),
            ValidationErrorKind::TooLong { field, max_length, actual_length } =>
                write!(f, "{} exceeds maximum length {} (got {})", field, max_length, actual_length),
            ValidationErrorKind::TooShort { field, min_length, actual_length } =>
                write!(f, "{} is shorter than minimum length {} (got {})", field, min_length, actual_length),
        }
    }
}

impl std::error::Error for ValidationError {}
```

### DomainError

Business rule violations and domain-specific failures with UUID tracking.

```rust
/// Domain business rule errors with UUID tracking
#[derive(Debug)]
pub struct DomainError {
    id: Uuid,
    kind: DomainErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainErrorKind {
    InvalidTransition { from: String, to: String },
    InsufficientFunds { available: i64, requested: i64 },
    AlreadyExists { aggregate_type: String, aggregate_id: String },
    NotFound { aggregate_type: String, aggregate_id: String },
    VersionConflict { expected: i64, actual: i64 },
}

impl DomainError {
    pub fn new(kind: DomainErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn error_id(&self) -> Uuid {
        self.id
    }

    pub fn kind(&self) -> &DomainErrorKind {
        &self.kind
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DomainErrorKind::InvalidTransition { from, to } =>
                write!(f, "cannot transition from {} to {}", from, to),
            DomainErrorKind::InsufficientFunds { available, requested } =>
                write!(f, "insufficient funds: {} available, {} requested", available, requested),
            DomainErrorKind::AlreadyExists { aggregate_type, aggregate_id } =>
                write!(f, "{} {} already exists", aggregate_type, aggregate_id),
            DomainErrorKind::NotFound { aggregate_type, aggregate_id } =>
                write!(f, "{} {} not found", aggregate_type, aggregate_id),
            DomainErrorKind::VersionConflict { expected, actual } =>
                write!(f, "version conflict: expected {}, got {}", expected, actual),
        }
    }
}

impl std::error::Error for DomainError {}

impl DomainError {
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
```

## Application layer: AggregateError

Aggregate errors combining validation and domain errors for command handling.

```rust
// ironstar-app/src/error.rs
use ironstar_domain::error::{DomainError, ValidationError};
use common_enums::ErrorCode;
use std::fmt;

/// Aggregate-level errors combining validation and domain errors.
///
/// Note: The 'Aggregate' in AggregateError refers to error aggregation (combining
/// multiple validation errors into a single result), not domain aggregates or
/// event sourcing aggregates.
/// This error type is used equally with fmodel-rust's Decider pattern and works
/// at the application layer boundary.
///
/// The `Validation` variant holds a `Vec` to support applicative validation:
/// collect all field errors rather than failing on the first.
/// The `Domain` variant holds a single error for monadic (fail-fast) semantics.
#[derive(Debug)]
pub enum AggregateError {
    /// Multiple validation errors (applicative style - collect all errors)
    Validation(Vec<ValidationError>),
    /// Single domain logic error (monadic style - fail fast)
    Domain(DomainError),
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateError::Validation(errors) => {
                let messages: Vec<_> = errors.iter().map(|e| e.to_string()).collect();
                write!(f, "{}", messages.join("; "))
            }
            AggregateError::Domain(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AggregateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AggregateError::Validation(errors) => errors.first().map(|e| e as &dyn std::error::Error),
            AggregateError::Domain(e) => Some(e),
        }
    }
}

impl From<ValidationError> for AggregateError {
    fn from(e: ValidationError) -> Self {
        AggregateError::Validation(vec![e])
    }
}

impl From<Vec<ValidationError>> for AggregateError {
    fn from(errors: Vec<ValidationError>) -> Self {
        AggregateError::Validation(errors)
    }
}

impl From<DomainError> for AggregateError {
    fn from(e: DomainError) -> Self {
        AggregateError::Domain(e)
    }
}

impl AggregateError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            AggregateError::Validation(_) => ErrorCode::ValidationFailed,
            AggregateError::Domain(e) => e.error_code(),
        }
    }
}
```

## Infrastructure layer: InfrastructureError

Infrastructure errors for database, cache, and event bus failures with UUID tracking.

```rust
// ironstar-interfaces/src/error.rs
use common_enums::ErrorCode;
use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Infrastructure errors from persistence, cache, or event bus with UUID tracking
#[derive(Debug)]
pub struct InfrastructureError {
    id: Uuid,
    kind: InfrastructureErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
pub enum InfrastructureErrorKind {
    Database(sqlx::Error),
    Serialization(serde_json::Error),
    EventBus(String),
    Cache(String),
    NotFound { resource: String, id: String },
}

impl InfrastructureError {
    pub fn new(kind: InfrastructureErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn error_id(&self) -> Uuid {
        self.id
    }

    pub fn kind(&self) -> &InfrastructureErrorKind {
        &self.kind
    }
}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            InfrastructureErrorKind::Database(e) => write!(f, "database error: {}", e),
            InfrastructureErrorKind::Serialization(e) => write!(f, "serialization error: {}", e),
            InfrastructureErrorKind::EventBus(msg) => write!(f, "event bus error: {}", msg),
            InfrastructureErrorKind::Cache(msg) => write!(f, "cache error: {}", msg),
            InfrastructureErrorKind::NotFound { resource, id } =>
                write!(f, "{} {} not found", resource, id),
        }
    }
}

impl std::error::Error for InfrastructureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            InfrastructureErrorKind::Database(e) => Some(e),
            InfrastructureErrorKind::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for InfrastructureError {
    fn from(e: sqlx::Error) -> Self {
        InfrastructureError::new(InfrastructureErrorKind::Database(e))
    }
}

impl From<serde_json::Error> for InfrastructureError {
    fn from(e: serde_json::Error) -> Self {
        InfrastructureError::new(InfrastructureErrorKind::Serialization(e))
    }
}

impl InfrastructureError {
    pub fn error_code(&self) -> ErrorCode {
        match &self.kind {
            InfrastructureErrorKind::Database(_) => ErrorCode::DatabaseError,
            InfrastructureErrorKind::Serialization(_) => ErrorCode::InternalError,
            InfrastructureErrorKind::EventBus(_) => ErrorCode::ServiceUnavailable,
            InfrastructureErrorKind::Cache(_) => ErrorCode::InternalError,
            InfrastructureErrorKind::NotFound { .. } => ErrorCode::NotFound,
        }
    }
}
```

## Presentation layer: AppError

The top-level error type used at HTTP boundaries, unifying all error categories with UUID tracking.

```rust
// ironstar-web/src/error.rs
use ironstar_app::error::AggregateError;
use ironstar_interfaces::error::InfrastructureError;
use ironstar_domain::error::{DomainError, ValidationError};
use common_enums::ErrorCode;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Top-level application error unifying all error categories with UUID tracking
#[derive(Debug)]
pub struct AppError {
    id: Uuid,
    kind: AppErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
pub enum AppErrorKind {
    Validation(ValidationError),
    Domain(DomainError),
    Infrastructure(InfrastructureError),
    NotFound { resource: String, id: String },
}

impl AppError {
    pub fn new(kind: AppErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn error_id(&self) -> Uuid {
        self.id
    }

    pub fn kind(&self) -> &AppErrorKind {
        &self.kind
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AppErrorKind::Validation(e) => write!(f, "{}", e),
            AppErrorKind::Domain(e) => write!(f, "{}", e),
            AppErrorKind::Infrastructure(e) => write!(f, "{}", e),
            AppErrorKind::NotFound { resource, id } =>
                write!(f, "{} {} not found", resource, id),
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
```

### ErrorResponse structure

JSON error response for HTTP APIs with error ID for correlation.

```rust
/// JSON error response structure with error ID for correlation
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    pub error_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
```

### AppError methods and HTTP integration

```rust
impl AppError {
    pub fn error_code(&self) -> ErrorCode {
        match &self.kind {
            AppErrorKind::Validation(_) => ErrorCode::ValidationFailed,
            AppErrorKind::Domain(e) => e.error_code(),
            AppErrorKind::Infrastructure(e) => e.error_code(),
            AppErrorKind::NotFound { .. } => ErrorCode::NotFound,
        }
    }

    pub fn http_status(&self) -> StatusCode {
        StatusCode::from_u16(self.error_code().http_status())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            error_id: self.id,
            details: None,
        }
    }
}
```

### From implementations for AppError

These implementations enable the `?` operator to propagate errors across layer boundaries.

```rust
// Conversions from lower layers
impl From<ValidationError> for AppError {
    fn from(e: ValidationError) -> Self {
        AppError::new(AppErrorKind::Validation(e))
    }
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        AppError::new(AppErrorKind::Domain(e))
    }
}

impl From<AggregateError> for AppError {
    fn from(e: AggregateError) -> Self {
        match e {
            // Take first validation error for AppError (or extend AppError to hold Vec)
            AggregateError::Validation(v) => {
                AppError::new(AppErrorKind::Validation(
                    v.into_iter().next().expect("validation errors non-empty")
                ))
            }
            AggregateError::Domain(d) => AppError::new(AppErrorKind::Domain(d)),
        }
    }
}

impl From<InfrastructureError> for AppError {
    fn from(e: InfrastructureError) -> Self {
        AppError::new(AppErrorKind::Infrastructure(e))
    }
}
```

### Axum IntoResponse integration

```rust
// Axum IntoResponse integration
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.http_status();
        let body = Json(self.to_response());
        (status, body).into_response()
    }
}
```

## Related documentation

- **Error handling decisions**: `error-handling-decisions.md` — Design principles, CQRS propagation patterns, migration strategy
- **Railway-oriented programming**: `~/.claude/commands/preferences/railway-oriented-programming.md` — Result types, bind, apply patterns
- **Rust error handling**: `~/.claude/commands/preferences/rust-development/02-error-handling.md` — Language-specific patterns
