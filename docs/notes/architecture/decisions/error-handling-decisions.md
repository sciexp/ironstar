# Error handling decisions

This document establishes error handling patterns for ironstar based on algebraic foundations, railway-oriented programming, and explicit effect boundaries.
Error handling in ironstar follows three principles: errors as sum types, effects explicit in signatures, and layer-appropriate error conversion.

## Design principles

From `../core/design-principles.md`, ironstar's error handling must embody:

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

**Modeling**: Explicit sum types with UUID tracking for distributed tracing

### 3. Panics

Unrecoverable programmer errors indicating broken invariants.

**Examples**:
- Index out of bounds (logic error)
- Unwrap on None when value guaranteed (broken invariant)
- Type conversion failures that should be impossible

**Modeling**: `panic!` or `assert!` — do not catch in domain logic

## Error type hierarchy

Ironstar uses a layered error type hierarchy where each layer has its own error type that converts to the layer above.
For complete type definitions, see `error-types.md`.

The hierarchy follows this conversion flow:

```
ValidationError ──┐
                  ├──> AggregateError ──┐
DomainError ──────┘                     ├──> AppError ──> HTTP Response
                                        │
InfrastructureError ────────────────────┘
```

| Layer | Error Type | Location |
|-------|------------|----------|
| Foundation | `ErrorCode` | `common-enums` |
| Domain | `ValidationError`, `DomainError` | `ironstar-domain` |
| Application | `AggregateError` | `ironstar-app` |
| Infrastructure | `InfrastructureError` | `ironstar-interfaces` |
| Presentation | `AppError` | `ironstar-web` |

Each error type implements `From` conversions enabling idiomatic `?` propagation across layer boundaries.

## UUID-tracked errors for distributed tracing

All error types in ironstar include a UUID identifier to enable correlation across async operations and distributed components.
This is essential for debugging event-sourced systems where errors may originate in background projections or async event handlers.

### DomainError with UUID tracking

```rust
use std::backtrace::Backtrace;
use uuid::Uuid;

pub struct DomainError {
    id: Uuid,
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl DomainError {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),  // Auto-generates UUID
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn error_id(&self) -> Uuid {
        self.id
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidTransition { from: String, to: String },
    InsufficientFunds { available: i64, requested: i64 },
    AlreadyExists { aggregate_type: String, aggregate_id: String },
    NotFound { aggregate_type: String, aggregate_id: String },
    VersionConflict { expected: i64, actual: i64 },
}
```

### Why UUID tracking is essential

UUID tracking enables critical debugging capabilities in event-sourced systems:

1. **Async operation correlation**: Background tasks (projections, analytics updates) need to correlate errors back to the originating command or event.
2. **Log aggregation**: Log systems can group all log entries related to a single error across distributed components.
3. **User-facing error messages**: Error IDs can be included in HTTP responses, allowing users to reference specific errors in support tickets.
4. **Distributed tracing**: Error IDs integrate with OpenTelemetry span IDs for end-to-end request tracing.

Example structured log with error correlation:

```rust
use tracing::error;

impl DomainError {
    pub fn log(&self) {
        error!(
            error.id = %self.id,
            error.kind = ?self.kind,
            error.backtrace = ?self.backtrace,
            "domain error occurred"
        );
    }
}
```

Example user-facing error response:

```json
{
  "code": "DOMAIN_ERROR",
  "message": "cannot transition from shipped to pending",
  "error_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

When the user reports this error, support can search logs for `error.id = 550e8400-e29b-41d4-a716-446655440000` to find the complete context including the backtrace, request trace, and all related events.

## Error propagation across CQRS layers

### Command handler flow

The complete error flow from command validation through aggregate execution to HTTP response.

```rust
// ironstar-web/src/handlers/commands.rs
use axum::{extract::State, response::IntoResponse, Json};
use ironstar_app::command_handlers::handle_todo_command;
use ironstar_domain::commands::TodoCommand;
use ironstar_web::error::AppError;
use std::sync::Arc;

/// POST handler for todo commands
pub async fn handle_add_todo(
    State(state): State<Arc<AppState>>,
    Json(cmd): Json<TodoCommand>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Call application layer handler (uses EventSourcedAggregate + todo_decider internally)
    let events = handle_todo_command(
        state.event_repository.clone(),
        state.event_bus.clone(),
        cmd,
    )
    .await?; // CommandError -> AppError via From impl

    // 2. Return 202 Accepted (SSE will deliver the update)
    Ok(axum::http::StatusCode::ACCEPTED)
}
```

### Aggregate pattern

Pure decision logic using fmodel-rust's Decider pattern.

```rust
// ironstar-domain/src/aggregates/todo.rs
use fmodel_rust::decider::Decider;
use ironstar_domain::error::{DomainError, ValidationError};
use ironstar_domain::events::TodoEvent;
use ironstar_domain::commands::TodoCommand;
use ironstar_app::error::AggregateError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoState {
    pub text: String,
    pub completed: bool,
}

/// Decider factory for todo aggregates
pub fn todo_decider<'a>() -> Decider<'a, TodoCommand, Option<TodoState>, TodoEvent, AggregateError> {
    Decider {
        decide: Box::new(|command, state| {
            match command {
                TodoCommand::Add { text } => {
                    // Validation errors
                    if text.is_empty() {
                        return Err(ValidationError::new(
                            ValidationErrorKind::EmptyField {
                                field: "text".to_string(),
                            }
                        ).into());
                    }

                    if text.len() > 500 {
                        return Err(ValidationError::new(
                            ValidationErrorKind::TooLong {
                                field: "text".to_string(),
                                max_length: 500,
                                actual_length: text.len(),
                            }
                        ).into());
                    }

                    // Check if already exists
                    if state.is_some() {
                        return Err(DomainError::new(
                            DomainErrorKind::AlreadyExists {
                                aggregate_type: "Todo".to_string(),
                                aggregate_id: text.clone(),
                            }
                        ).into());
                    }

                    Ok(vec![TodoEvent::Added { text: text.clone() }])
                }
                TodoCommand::Toggle => {
                    // Domain errors
                    match state {
                        Some(s) => Ok(vec![TodoEvent::Toggled {
                            completed: !s.completed,
                        }]),
                        None => Err(DomainError::new(
                            DomainErrorKind::NotFound {
                                aggregate_type: "Todo".to_string(),
                                aggregate_id: "".to_string(),
                            }
                        ).into()),
                    }
                }
            }
        }),

        evolve: Box::new(|state, event| {
            match event {
                TodoEvent::Added { text } => Some(TodoState {
                    text: text.clone(),
                    completed: false,
                }),
                TodoEvent::Toggled { completed } => state.clone().map(|mut s| {
                    s.completed = *completed;
                    s
                }),
            }
        }),

        initial_state: Box::new(|| None),
    }
}
```

### Application layer orchestration

Command handler orchestrating I/O around pure Decider using fmodel-rust's EventSourcedAggregate.

```rust
// ironstar-app/src/command_handlers.rs
use fmodel_rust::aggregate::EventSourcedAggregate;
use ironstar_domain::aggregates::todo::todo_decider;
use ironstar_domain::commands::TodoCommand;
use ironstar_domain::events::TodoEvent;
use ironstar_app::error::AggregateError;
use ironstar_interfaces::error::InfrastructureError;
use ironstar_interfaces::EventBus;
use uuid::Uuid;

/// Orchestrate todo command handling with error propagation
pub async fn handle_todo_command(
    event_repository: Arc<SqliteEventRepository>,
    event_bus: Arc<dyn EventBus>,
    command: TodoCommand,
) -> Result<Vec<(TodoEvent, Uuid)>, CommandError> {
    // 1. Create EventSourcedAggregate wrapping pure Decider and EventRepository
    let aggregate = EventSourcedAggregate::new(
        event_repository.clone(),
        todo_decider(),
    );

    // 2. Handle command - fetches events, applies Decider, persists results
    // Can fail with AggregateError (from Decider) or InfrastructureError (from EventRepository)
    let events = aggregate.handle(&command).await?;

    // 3. Publish to event bus (fire and forget)
    for (event, version) in &events {
        let key = format!("events/Todo/{}", command.aggregate_id());
        let _ = event_bus.publish(&key, event).await;
    }

    Ok(events)
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

### Testing Decider error conditions

Using fmodel-rust's DeciderTestSpecification for given/when/then style testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use fmodel_rust::decider::DeciderTestSpecification;
    use ironstar_domain::error::{ValidationError, ValidationErrorKind};

    #[test]
    fn test_empty_text_validation() {
        let cmd = TodoCommand::Add {
            text: String::new(),
        };

        // DeciderTestSpecification validates error types
        let result = (todo_decider().decide)(&cmd, &None);

        assert!(matches!(
            result,
            Err(AggregateError::Validation(errors))
            if errors.iter().any(|e| matches!(
                e.kind(),
                ValidationErrorKind::EmptyField { field } if field == "text"
            ))
        ));
    }

    #[test]
    fn test_too_long_text_validation() {
        let cmd = TodoCommand::Add {
            text: "a".repeat(501),
        };

        let result = (todo_decider().decide)(&cmd, &None);

        assert!(matches!(
            result,
            Err(AggregateError::Validation(errors))
            if errors.iter().any(|e| matches!(
                e.kind(),
                ValidationErrorKind::TooLong { max_length, .. } if *max_length == 500
            ))
        ));
    }

    #[test]
    fn test_toggle_nonexistent_todo() {
        // Using DeciderTestSpecification for behavior testing
        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])  // No prior events
            .when(TodoCommand::Toggle)
            .then_error(|err| {
                matches!(err, AggregateError::Domain(e)
                    if matches!(e.kind(), DomainErrorKind::NotFound { .. }))
            });
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

Extract error types to appropriate crates following the layered structure documented in `../core/crate-architecture.md`:

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

- **Error type definitions**: `error-types.md` — Complete error type hierarchy with code examples
- **Design principles**: `../core/design-principles.md` — Algebraic foundations, effect boundaries
- **Railway-oriented programming**: `~/.claude/commands/preferences/railway-oriented-programming.md` — Result types, bind, apply patterns
- **Command write patterns**: `../cqrs/command-write-patterns.md` — Aggregate error handling in CQRS
- **Crate architecture**: `../core/crate-architecture.md` — Multi-crate decomposition plan
- **Rust error handling**: `~/.claude/commands/preferences/rust-development/02-error-handling.md` — Language-specific patterns
