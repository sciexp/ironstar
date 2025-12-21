# Error handling decisions

This document establishes error handling patterns for ironstar based on algebraic foundations, railway-oriented programming, and explicit effect boundaries.
Error handling in ironstar follows three principles: errors as sum types, effects explicit in signatures, and layer-appropriate error conversion.

## Design principles

From `design-principles.md`, ironstar's error handling must embody:

> "Side effects should be explicit in type signatures and isolated at boundaries to preserve compositionality."

This translates to concrete requirements:

| Principle | Error Handling Consequence |
|-----------|---------------------------|
| Algebraic data types | Errors as sum types (enum variants) |
| Explicit effects | `Result<T, E>` in all fallible signatures |
| Compositionality | Error types form a category with morphisms |
| Boundary isolation | Errors converted at layer boundaries |
| Type-level guarantees | Invalid error states unrepresentable |

## Error categories

Following Scott Wlaschin's classification from "Domain Modeling Made Functional" (see `~/.claude/commands/preferences/railway-oriented-programming.md`), ironstar recognizes three error categories:

### 1. Domain errors

Expected outcomes of domain operations that subject matter experts can describe.

**Examples**:
- Validation failures (invalid email format, negative quantity)
- Business rule violations (insufficient funds, order already shipped)
- Not found errors (aggregate doesn't exist)
- State transition errors (cannot cancel completed order)

**Modeling**: Explicit sum types with `Result<T, DomainError>`

### 2. Infrastructure errors

Technical failures outside domain logic, may be transient.

**Examples**:
- Database connection failures
- Network timeouts
- Serialization/deserialization errors
- File system errors

**Modeling**: Explicit sum types or transparent propagation via `anyhow`

### 3. Panics

Unrecoverable programmer errors indicating broken invariants.

**Examples**:
- Index out of bounds (logic error)
- Unwrap on None when value guaranteed (broken invariant)
- Type conversion failures that should be impossible

**Modeling**: `panic!` or `assert!` — do not catch in domain logic

## Error type hierarchy

Ironstar uses a layered error type hierarchy where each layer has its own error type that converts to the layer above.

### Foundation layer error types

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

### Domain layer errors

Pure domain errors with no infrastructure dependencies.

```rust
// ironstar-domain/src/error.rs
use common_enums::ErrorCode;
use std::fmt;

/// Domain-level validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyField { field: String },
    InvalidFormat { field: String, expected: String },
    OutOfRange { field: String, min: i64, max: i64, actual: i64 },
    TooLong { field: String, max_length: usize, actual_length: usize },
    TooShort { field: String, min_length: usize, actual_length: usize },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyField { field } =>
                write!(f, "{} cannot be empty", field),
            ValidationError::InvalidFormat { field, expected } =>
                write!(f, "{} has invalid format, expected: {}", field, expected),
            ValidationError::OutOfRange { field, min, max, actual } =>
                write!(f, "{} must be between {} and {}, got {}", field, min, max, actual),
            ValidationError::TooLong { field, max_length, actual_length } =>
                write!(f, "{} exceeds maximum length {} (got {})", field, max_length, actual_length),
            ValidationError::TooShort { field, min_length, actual_length } =>
                write!(f, "{} is shorter than minimum length {} (got {})", field, min_length, actual_length),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Domain business rule errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InvalidTransition { from: String, to: String },
    InsufficientFunds { available: i64, requested: i64 },
    AlreadyExists { aggregate_type: String, aggregate_id: String },
    NotFound { aggregate_type: String, aggregate_id: String },
    VersionConflict { expected: i64, actual: i64 },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::InvalidTransition { from, to } =>
                write!(f, "cannot transition from {} to {}", from, to),
            DomainError::InsufficientFunds { available, requested } =>
                write!(f, "insufficient funds: {} available, {} requested", available, requested),
            DomainError::AlreadyExists { aggregate_type, aggregate_id } =>
                write!(f, "{} {} already exists", aggregate_type, aggregate_id),
            DomainError::NotFound { aggregate_type, aggregate_id } =>
                write!(f, "{} {} not found", aggregate_type, aggregate_id),
            DomainError::VersionConflict { expected, actual } =>
                write!(f, "version conflict: expected {}, got {}", expected, actual),
        }
    }
}

impl std::error::Error for DomainError {}

impl DomainError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            DomainError::InvalidTransition { .. } => ErrorCode::ValidationFailed,
            DomainError::InsufficientFunds { .. } => ErrorCode::ValidationFailed,
            DomainError::AlreadyExists { .. } => ErrorCode::Conflict,
            DomainError::NotFound { .. } => ErrorCode::NotFound,
            DomainError::VersionConflict { .. } => ErrorCode::Conflict,
        }
    }
}
```

### Application layer errors

Aggregate errors combining validation and domain errors.

```rust
// ironstar-app/src/error.rs
use ironstar_domain::error::{DomainError, ValidationError};
use common_enums::ErrorCode;
use std::fmt;

/// Aggregate-level errors combining validation and domain errors
#[derive(Debug)]
pub enum AggregateError {
    Validation(ValidationError),
    Domain(DomainError),
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateError::Validation(e) => write!(f, "{}", e),
            AggregateError::Domain(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AggregateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AggregateError::Validation(e) => Some(e),
            AggregateError::Domain(e) => Some(e),
        }
    }
}

impl From<ValidationError> for AggregateError {
    fn from(e: ValidationError) -> Self {
        AggregateError::Validation(e)
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

### Infrastructure layer errors

Infrastructure errors for database, cache, and event bus failures.

```rust
// ironstar-interfaces/src/error.rs
use common_enums::ErrorCode;
use std::fmt;

/// Infrastructure errors from persistence, cache, or event bus
#[derive(Debug)]
pub enum InfrastructureError {
    Database(sqlx::Error),
    Serialization(serde_json::Error),
    EventBus(String),
    Cache(String),
    NotFound { resource: String, id: String },
}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfrastructureError::Database(e) => write!(f, "database error: {}", e),
            InfrastructureError::Serialization(e) => write!(f, "serialization error: {}", e),
            InfrastructureError::EventBus(msg) => write!(f, "event bus error: {}", msg),
            InfrastructureError::Cache(msg) => write!(f, "cache error: {}", msg),
            InfrastructureError::NotFound { resource, id } =>
                write!(f, "{} {} not found", resource, id),
        }
    }
}

impl std::error::Error for InfrastructureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InfrastructureError::Database(e) => Some(e),
            InfrastructureError::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for InfrastructureError {
    fn from(e: sqlx::Error) -> Self {
        InfrastructureError::Database(e)
    }
}

impl From<serde_json::Error> for InfrastructureError {
    fn from(e: serde_json::Error) -> Self {
        InfrastructureError::Serialization(e)
    }
}

impl InfrastructureError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            InfrastructureError::Database(_) => ErrorCode::DatabaseError,
            InfrastructureError::Serialization(_) => ErrorCode::InternalError,
            InfrastructureError::EventBus(_) => ErrorCode::ServiceUnavailable,
            InfrastructureError::Cache(_) => ErrorCode::InternalError,
            InfrastructureError::NotFound { .. } => ErrorCode::NotFound,
        }
    }
}
```

### Application boundary error (AppError)

The top-level error type used at HTTP boundaries.

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
use std::fmt;

/// Top-level application error unifying all error categories
#[derive(Debug)]
pub enum AppError {
    Validation(ValidationError),
    Domain(DomainError),
    Infrastructure(InfrastructureError),
    NotFound { resource: String, id: String },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Validation(e) => write!(f, "{}", e),
            AppError::Domain(e) => write!(f, "{}", e),
            AppError::Infrastructure(e) => write!(f, "{}", e),
            AppError::NotFound { resource, id } =>
                write!(f, "{} {} not found", resource, id),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Validation(e) => Some(e),
            AppError::Domain(e) => Some(e),
            AppError::Infrastructure(e) => Some(e),
            AppError::NotFound { .. } => None,
        }
    }
}

/// JSON error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl AppError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            AppError::Validation(_) => ErrorCode::ValidationFailed,
            AppError::Domain(e) => e.error_code(),
            AppError::Infrastructure(e) => e.error_code(),
            AppError::NotFound { .. } => ErrorCode::NotFound,
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
            details: None,
        }
    }
}

// Conversions from lower layers
impl From<ValidationError> for AppError {
    fn from(e: ValidationError) -> Self {
        AppError::Validation(e)
    }
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        AppError::Domain(e)
    }
}

impl From<AggregateError> for AppError {
    fn from(e: AggregateError) -> Self {
        match e {
            AggregateError::Validation(v) => AppError::Validation(v),
            AggregateError::Domain(d) => AppError::Domain(d),
        }
    }
}

impl From<InfrastructureError> for AppError {
    fn from(e: InfrastructureError) -> Self {
        AppError::Infrastructure(e)
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
```

## Error propagation across CQRS layers

### Command handler flow

The complete error flow from command validation through aggregate execution to HTTP response.

```rust
// ironstar-web/src/handlers/commands.rs
use axum::{extract::State, response::IntoResponse, Json};
use ironstar_app::command_handlers::handle_command;
use ironstar_domain::aggregates::TodoAggregate;
use ironstar_domain::commands::TodoCommand;
use ironstar_web::error::AppError;
use std::sync::Arc;

/// POST handler for todo commands
pub async fn handle_add_todo(
    State(state): State<Arc<AppState>>,
    Json(cmd): Json<TodoCommand>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Command handler returns Result<Vec<Event>, AggregateError>
    let events = handle_command::<TodoAggregate>(
        &state.event_store,
        &state.event_bus,
        &cmd.aggregate_id,
        cmd,
    )
    .await?; // AggregateError -> AppError via From impl

    // 2. Return 202 Accepted (SSE will deliver the update)
    Ok(axum::http::StatusCode::ACCEPTED)
}
```

### Aggregate pattern

Pure aggregate with domain-specific error type.

```rust
// ironstar-domain/src/aggregates/todo.rs
use ironstar_domain::error::{DomainError, ValidationError};
use ironstar_domain::events::TodoEvent;
use ironstar_domain::commands::TodoCommand;
use ironstar_app::error::AggregateError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoState {
    pub text: String,
    pub completed: bool,
}

impl Default for TodoState {
    fn default() -> Self {
        Self {
            text: String::new(),
            completed: false,
        }
    }
}

pub struct TodoAggregate;

impl TodoAggregate {
    /// Pure command handler - returns domain errors only
    pub fn handle_command(
        state: &TodoState,
        cmd: TodoCommand,
    ) -> Result<Vec<TodoEvent>, AggregateError> {
        match cmd {
            TodoCommand::Add { text } => {
                // Validation errors
                if text.is_empty() {
                    return Err(ValidationError::EmptyField {
                        field: "text".to_string(),
                    }
                    .into());
                }

                if text.len() > 500 {
                    return Err(ValidationError::TooLong {
                        field: "text".to_string(),
                        max_length: 500,
                        actual_length: text.len(),
                    }
                    .into());
                }

                Ok(vec![TodoEvent::Added { text }])
            }
            TodoCommand::Toggle => {
                // Domain errors
                if state.text.is_empty() {
                    return Err(DomainError::NotFound {
                        aggregate_type: "Todo".to_string(),
                        aggregate_id: "".to_string(),
                    }
                    .into());
                }

                Ok(vec![TodoEvent::Toggled {
                    completed: !state.completed,
                }])
            }
        }
    }

    pub fn apply_event(mut state: TodoState, event: TodoEvent) -> TodoState {
        match event {
            TodoEvent::Added { text } => {
                state.text = text;
                state
            }
            TodoEvent::Toggled { completed } => {
                state.completed = completed;
                state
            }
        }
    }
}
```

### Application layer orchestration

Command handler orchestrating I/O around pure aggregate.

```rust
// ironstar-app/src/command_handlers.rs
use ironstar_interfaces::{EventStore, EventBus};
use ironstar_domain::aggregates::Aggregate;
use ironstar_app::error::AggregateError;
use ironstar_interfaces::error::InfrastructureError;

/// Orchestrate command handling with error propagation
pub async fn handle_command<A: Aggregate>(
    event_store: &dyn EventStore,
    event_bus: &dyn EventBus,
    aggregate_id: &str,
    command: A::Command,
) -> Result<Vec<A::Event>, CommandError> {
    // 1. Load events (can fail with InfrastructureError)
    let events = event_store
        .query_aggregate(A::NAME, aggregate_id)
        .await?;

    // 2. Reconstruct state
    let state = events
        .into_iter()
        .filter_map(|e| deserialize_event::<A>(&e))
        .fold(A::State::default(), A::apply_event);

    // 3. Handle command (can fail with AggregateError)
    let new_events = A::handle_command(&state, command)?;

    // 4. Persist events (can fail with InfrastructureError)
    for event in &new_events {
        event_store.append(serialize_event::<A>(aggregate_id, event)).await?;
    }

    // 5. Publish to event bus (fire and forget)
    for event in &new_events {
        let _ = event_bus.publish(event);
    }

    Ok(new_events)
}

/// Command error unifying aggregate and infrastructure errors
#[derive(Debug)]
pub enum CommandError {
    Aggregate(AggregateError),
    Infrastructure(InfrastructureError),
}

impl From<AggregateError> for CommandError {
    fn from(e: AggregateError) -> Self {
        CommandError::Aggregate(e)
    }
}

impl From<InfrastructureError> for CommandError {
    fn from(e: InfrastructureError) -> Self {
        CommandError::Infrastructure(e)
    }
}

impl From<CommandError> for crate::error::AppError {
    fn from(e: CommandError) -> Self {
        match e {
            CommandError::Aggregate(a) => a.into(),
            CommandError::Infrastructure(i) => i.into(),
        }
    }
}
```

## Railway-oriented programming integration

Ironstar's error handling integrates with railway-oriented programming patterns (see `~/.claude/commands/preferences/railway-oriented-programming.md`).

### Monadic bind for sequential operations

Chain operations that can fail, short-circuiting on first error.

```rust
use ironstar_domain::error::ValidationError;

/// Validate email format
fn validate_email(email: &str) -> Result<String, ValidationError> {
    if !email.contains('@') {
        return Err(ValidationError::InvalidFormat {
            field: "email".to_string(),
            expected: "email@domain.com".to_string(),
        });
    }
    Ok(email.to_lowercase())
}

/// Validate email not already taken (requires I/O)
async fn check_email_available(
    email: &str,
    store: &dyn EventStore,
) -> Result<String, InfrastructureError> {
    // Database lookup
    let exists = store.query_by_email(email).await?;
    if exists {
        return Err(InfrastructureError::NotFound {
            resource: "email".to_string(),
            id: email.to_string(),
        });
    }
    Ok(email.to_string())
}

/// Railway-oriented pipeline
async fn register_user(
    email: &str,
    store: &dyn EventStore,
) -> Result<String, AppError> {
    // Pure validation (sync)
    let validated_email = validate_email(email)?;

    // External validation (async I/O)
    let available_email = check_email_available(&validated_email, store).await?;

    Ok(available_email)
}
```

### Applicative validation for collecting errors

Validate multiple fields independently and collect all errors.

```rust
use ironstar_domain::error::ValidationError;

/// Validate user registration collecting all errors
fn validate_user_registration(
    email: &str,
    name: &str,
    age: i64,
) -> Result<(String, String, i64), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate email
    if !email.contains('@') {
        errors.push(ValidationError::InvalidFormat {
            field: "email".to_string(),
            expected: "email@domain.com".to_string(),
        });
    }

    // Validate name
    if name.is_empty() {
        errors.push(ValidationError::EmptyField {
            field: "name".to_string(),
        });
    }

    // Validate age
    if age < 0 || age > 150 {
        errors.push(ValidationError::OutOfRange {
            field: "age".to_string(),
            min: 0,
            max: 150,
            actual: age,
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok((email.to_lowercase(), name.to_string(), age))
}
```

## API error responses

### Structured error format

HTTP responses include structured error codes and messages.

```rust
// Example error response JSON
{
  "code": "VALIDATION_FAILED",
  "message": "text cannot be empty",
  "details": null
}
```

### Error response with details

For validation errors with multiple fields:

```rust
impl AppError {
    pub fn validation_with_details(errors: Vec<ValidationError>) -> Self {
        let details = errors
            .iter()
            .map(|e| {
                serde_json::json!({
                    "message": e.to_string(),
                })
            })
            .collect::<Vec<_>>();

        AppError::Validation(errors[0].clone()) // Use first as primary
    }
}
```

## Logging hooks for errors

Structured logging integration with tracing crate.

```rust
use tracing::{error, warn, info};

impl AppError {
    /// Log error with appropriate level based on severity
    pub fn log(&self) {
        match self {
            AppError::Validation(e) => {
                warn!(
                    error.type = "validation",
                    error.message = %e,
                    "validation error"
                );
            }
            AppError::Domain(e) => {
                info!(
                    error.type = "domain",
                    error.code = ?e.error_code(),
                    error.message = %e,
                    "domain error"
                );
            }
            AppError::Infrastructure(e) => {
                error!(
                    error.type = "infrastructure",
                    error.code = ?e.error_code(),
                    error.message = %e,
                    error.source = ?e.source(),
                    "infrastructure error"
                );
            }
            AppError::NotFound { resource, id } => {
                info!(
                    error.type = "not_found",
                    resource = resource,
                    id = id,
                    "resource not found"
                );
            }
        }
    }
}

// Use in handler
pub async fn handle_command_with_logging(
    State(state): State<Arc<AppState>>,
    Json(cmd): Json<TodoCommand>,
) -> Result<impl IntoResponse, AppError> {
    match handle_add_todo(State(state), Json(cmd)).await {
        Ok(response) => Ok(response),
        Err(e) => {
            e.log(); // Log before returning
            Err(e)
        }
    }
}
```

## Error handling testing

### Testing aggregate error conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ironstar_domain::error::ValidationError;

    #[test]
    fn test_empty_text_validation() {
        let state = TodoState::default();
        let cmd = TodoCommand::Add {
            text: String::new(),
        };

        let result = TodoAggregate::handle_command(&state, cmd);

        assert!(matches!(
            result,
            Err(AggregateError::Validation(ValidationError::EmptyField { .. }))
        ));
    }

    #[test]
    fn test_too_long_text_validation() {
        let state = TodoState::default();
        let cmd = TodoCommand::Add {
            text: "a".repeat(501),
        };

        let result = TodoAggregate::handle_command(&state, cmd);

        assert!(matches!(
            result,
            Err(AggregateError::Validation(ValidationError::TooLong { .. }))
        ));
    }
}
```

### Testing error conversions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_converts_to_app_error() {
        let validation_err = ValidationError::EmptyField {
            field: "email".to_string(),
        };
        let app_err: AppError = validation_err.into();

        assert_eq!(app_err.error_code(), ErrorCode::ValidationFailed);
        assert_eq!(app_err.http_status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_domain_error_converts_to_app_error() {
        let domain_err = DomainError::NotFound {
            aggregate_type: "Todo".to_string(),
            aggregate_id: "123".to_string(),
        };
        let app_err: AppError = domain_err.into();

        assert_eq!(app_err.error_code(), ErrorCode::NotFound);
        assert_eq!(app_err.http_status(), StatusCode::NOT_FOUND);
    }
}
```

## Migration strategy

For projects starting as a single crate and later migrating to multi-crate workspace:

### Phase 1: Single crate structure

```
src/
├── error.rs              # All error types in one module
│   ├── ValidationError
│   ├── DomainError
│   ├── InfrastructureError
│   └── AppError
├── domain/
│   └── aggregates/
│       └── todo.rs       # Uses error::ValidationError, error::DomainError
├── infrastructure/
│   └── event_store.rs    # Uses error::InfrastructureError
└── presentation/
    └── handlers/
        └── commands.rs   # Uses error::AppError
```

### Phase 2: Multi-crate workspace

Extract error types to appropriate crates following the layered structure documented in `crate-architecture.md`:

```
crates/
├── common-enums/
│   └── src/lib.rs                    # ErrorCode
├── ironstar-domain/
│   └── src/error.rs                  # ValidationError, DomainError
├── ironstar-app/
│   └── src/error.rs                  # AggregateError
├── ironstar-interfaces/
│   └── src/error.rs                  # InfrastructureError
└── ironstar-web/
    └── src/error.rs                  # AppError
```

Update imports:

```rust
// Before (single crate)
use crate::error::{AppError, ValidationError};

// After (multi-crate)
use ironstar_web::error::AppError;
use ironstar_domain::error::ValidationError;
```

## Related documentation

- **Design principles**: `design-principles.md` — Algebraic foundations, effect boundaries
- **Railway-oriented programming**: `~/.claude/commands/preferences/railway-oriented-programming.md` — Result types, bind, apply patterns
- **Command write patterns**: `command-write-patterns.md` — Aggregate error handling in CQRS
- **Crate architecture**: `crate-architecture.md` — Multi-crate decomposition plan
- **Rust error handling**: `~/.claude/commands/preferences/rust-development/02-error-handling.md` — Language-specific patterns
