# Code review checklist

This checklist ensures code changes adhere to ironstar's architectural principles.
Use it during self-review before commits and during PR reviews.

## Effect boundary enforcement

The async/sync boundary is the effect boundary.
Violations indicate misplaced logic.

### Domain layer (`crates/ironstar/src/domain/`)

Domain code must be synchronous and pure.
Clippy cannot automatically enforce this, so review manually.

| Check | Description |
|-------|-------------|
| No `async fn` | Functions must be synchronous |
| No `.await` | No awaiting futures anywhere |
| No `tokio::` imports | No async runtime dependencies |
| No `sqlx::` imports | No database dependencies |
| No `reqwest::` imports | No HTTP client dependencies |
| No `std::fs` | No file system operations |
| No `std::net` | No network operations |
| No `std::io::Read/Write` | No I/O traits (except for derive macros) |

If you need async in what seems like domain logic, the logic belongs in the application layer, or the async call should happen before/after calling the pure domain function.

### Application layer (`crates/ironstar/src/application/`)

Application code orchestrates async I/O around sync domain logic.

| Check | Description |
|-------|-------------|
| Handlers are `async fn` | Coordination requires async for I/O |
| Domain calls are sync | Calls to domain functions don't use `.await` |
| Infrastructure calls use `.await` | All I/O is explicitly awaited |
| No SQL queries inline | Database access goes through infrastructure |

### Infrastructure layer (`crates/ironstar/src/infrastructure/`)

Infrastructure code implements I/O effects.

| Check | Description |
|-------|-------------|
| Public functions are `async fn` | All I/O operations are async |
| Implements port traits | Adapters implement interfaces from ironstar-interfaces |
| No domain logic | Business rules belong in domain layer |

### Presentation layer (`crates/ironstar/src/presentation/`)

Presentation code handles HTTP concerns.

| Check | Description |
|-------|-------------|
| Handlers are `async fn` | HTTP handlers are async |
| Calls application layer | Not infrastructure directly (except extractors) |
| No business logic | Domain rules stay in domain layer |

## Type safety

| Check | Description |
|-------|-------------|
| Value objects enforce invariants | Use smart constructors, not raw primitives |
| Sum types model states | Use enums for mutually exclusive states |
| Errors are typed | Use domain-specific error types, not `anyhow` in domain |
| No `.unwrap()` in domain | Use `Result` or `Option` propagation |

## Event sourcing

| Check | Description |
|-------|-------------|
| Events are past tense | `TodoCreated`, not `CreateTodo` |
| Events are immutable | No mutable fields on event types |
| Aggregates are pure | `decide` and `evolve` functions are sync and deterministic |
| State from events | State is reconstructed via fold, not stored directly |

## Naming conventions

| Check | Description |
|-------|-------------|
| Commands are imperative | `CreateTodo`, `CompleteTodo` |
| Events are past tense | `TodoCreated`, `TodoCompleted` |
| Aggregates use domain nouns | `TodoAggregate`, `QuerySessionAggregate` |
| Value objects describe values | `TodoText`, `QueryId`, `DatasetRef` |

## Documentation

| Check | Description |
|-------|-------------|
| Module docs explain purpose | Each `mod.rs` has `//!` documentation |
| Public items have docs | Use `///` for public types and functions |
| Examples use `ignore` or `no_run` | Doctests are disabled; use integration tests |

## Testing

| Check | Description |
|-------|-------------|
| Domain tests are sync | No `#[tokio::test]` in domain tests |
| Use DeciderTestSpecification | given/when/then DSL for aggregate tests |
| Infrastructure tests are async | Use `#[tokio::test]` for I/O tests |
| Integration tests in `tests/` | Runnable examples as integration tests |

## Quick reference: where does this belong?

| If you're writing... | It belongs in... |
|---------------------|------------------|
| A new aggregate type | `domain/` |
| A command or event type | `domain/` |
| A value object with validation | `domain/` |
| A pure state transition | `domain/` |
| An async workflow | `application/` |
| A command handler | `application/` |
| A query handler | `application/` |
| A database query | `infrastructure/` |
| A cache implementation | `infrastructure/` |
| An HTTP handler | `presentation/` |
| An HTML template | `presentation/` |
| An SSE stream | `presentation/` |
