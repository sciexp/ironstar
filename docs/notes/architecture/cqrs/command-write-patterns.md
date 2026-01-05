# Command write patterns

This document covers command handling, aggregate patterns, testing with the given/when/then DSL, and event store implementation for ironstar's CQRS write path.
Commands trigger state transitions by emitting events, which are then persisted and broadcast to subscribers.

## Write path (command handling)

**Decision: Command → Event → Append → Broadcast, with immediate 202 Accepted response.**

### Pattern

```rust
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use datastar::axum::ReadSignals;  // Requires datastar = { features = ["axum"] }
use uuid::Uuid;
use chrono::Utc;

/// Command handler example
async fn handle_execute_query(
    State(app_state): State<AppState>,
    ReadSignals(signals): ReadSignals<ExecuteAnalyticsQuery>,
) -> impl IntoResponse {
    // Note: ValidationError type is defined in event-sourcing-core.md
    // 1. Validate command (pure function)
    let events = match validate_and_emit_events(signals) {
        Ok(events) => events,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    // 2. Append to event store (effect)
    for event in events {
        if let Err(e) = app_state.event_store.append(event.clone()).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }

        // 3. Broadcast to subscribers (effect)
        // Ignore send errors (no active subscribers is fine)
        let _ = app_state.event_bus.send(event);
    }

    // 4. Return immediately (do NOT wait for SSE update)
    StatusCode::ACCEPTED.into_response()
}

/// Pure command validation and event generation
/// See "Analytics query validation" section for detailed validation logic
fn validate_and_emit_events(cmd: ExecuteAnalyticsQuery) -> Result<Vec<DomainEvent>, ValidationError> {
    // Validate SQL safety, dataset URL, query complexity, and timeout
    // Full validation logic shown in "Analytics query validation" section below

    Ok(vec![DomainEvent::QueryStarted {
        query_id: Uuid::new_v4(),
        sql: cmd.sql,
        dataset_url: cmd.dataset_url,
        timeout_ms: cmd.timeout_ms.unwrap_or(30_000),
        started_at: Utc::now(),
    }])
}
```

### Loading indicator integration

Frontend adds loading class, backend removes it via SSE:

```html
<form data-on:submit.prevent="
    el.classList.add('loading');
    @post('/execute-query', {body: {sql: $querySql, dataset_url: $datasetUrl}})
">
    <textarea data-model="$querySql" placeholder="SELECT * FROM table"></textarea>
    <input data-model="$datasetUrl" placeholder="hf://datasets/owner/repo" />
    <button type="submit">
        Execute Query
        <span data-show="el.closest('form').classList.contains('loading')">Running...</span>
    </button>
</form>
<div id="query-results"><!-- SSE updates will morph this --></div>
```

Backend removes loading indicator:

```rust
fn render_query_results(state: &QueryResultsState) -> String {
    hypertext::html! {
        <div id="query-results">
            @if let Some(result) = &state.current_result {
                <table>
                    <thead><tr>@for col in &result.columns { <th>{col}</th> }</tr></thead>
                    <tbody>@for row in &result.rows {
                        <tr>@for cell in row { <td>{cell}</td> }</tr>
                    }</tbody>
                </table>
            }
        </div>
        <script data-effect="el.remove()">
            "document.querySelector('form').classList.remove('loading');"
        </script>
    }
}
```

This pattern ensures:
1. User sees immediate feedback (loading indicator)
2. POST returns quickly (no blocking)
3. SSE delivers the update and removes the loading indicator
4. No optimistic updates (backend is source of truth)

## Chart data streaming

Charts receive configuration updates via SSE using PatchSignals.
See `../frontend/ds-echarts-integration-guide.md` for complete ECharts Lit component implementation.

```rust
async fn chart_data_sse(
    State(state): State<Arc<AppState>>,
    Path(chart_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let data = state.analytics.query_chart_data(&chart_id).await?;
        yield Ok(PatchSignals::new(
            serde_json::json!({"chartOption": build_echarts_option(&data)}).to_string()
        ).write_as_axum_sse_event());

        let mut sub = state.event_bus.subscribe(&format!("charts/{}/updates", chart_id)).await;
        while let Some(update) = sub.next().await {
            yield Ok(PatchSignals::new(
                serde_json::json!({"chartOption": build_echarts_option(&update)}).to_string()
            ).write_as_axum_sse_event());
        }
    };
    Sse::new(stream)
}
```

## Analytics query validation

Analytics queries require specialized validation to prevent SQL injection, resource exhaustion, and data corruption.
The validation layer combines SQL safety checks, query complexity limits, dataset URL validation, and timeout policies.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    UnsafeSQL,
    InvalidDatasetURL,
    QueryTooComplex,
    TimeoutTooLong,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsafeSQL => write!(f, "Query contains unsafe SQL (DROP, DELETE, etc.)"),
            Self::InvalidDatasetURL => write!(f, "Dataset URL must use hf:// protocol"),
            Self::QueryTooComplex => write!(f, "Query complexity exceeds maximum"),
            Self::TimeoutTooLong => write!(f, "Query timeout exceeds max (60s)"),
        }
    }
}

impl std::error::Error for ValidationError {}

// SQL safety: prevent destructive operations (defense-in-depth with read-only connections)
fn is_sql_safe(sql: &str) -> bool {
    let sql_upper = sql.to_uppercase();
    !sql_upper.contains("DROP ")
        && !sql_upper.contains("DELETE ")
        && !sql_upper.contains("TRUNCATE ")
        && !sql_upper.contains("ALTER ")
        && !sql_upper.contains("INSERT ")
        && !sql_upper.contains("UPDATE ")
}

// Query complexity: coarse-grained heuristic to prevent resource exhaustion
const MAX_QUERY_COMPLEXITY: u32 = 50;

fn estimate_query_complexity(sql: &str) -> u32 {
    let sql_upper = sql.to_uppercase();
    let mut complexity = 0;
    complexity += (sql_upper.matches("JOIN ").count() * 10) as u32;
    complexity += (sql_upper.matches("SELECT ").count().saturating_sub(1) * 5) as u32;
    complexity += (sql_upper.matches("GROUP BY").count() * 2) as u32;
    complexity
}

// Dataset URL: validate protocol (hf:// for HuggingFace, future: s3://, https://)
fn validate_dataset_url(url: &str) -> Result<(), ValidationError> {
    if !url.starts_with("hf://") {
        return Err(ValidationError::InvalidDatasetURL);
    }
    Ok(())
}

// Timeout policy: default 30s, max 60s
const DEFAULT_QUERY_TIMEOUT_MS: u64 = 30_000;
const MAX_QUERY_TIMEOUT_MS: u64 = 60_000;

fn validate_timeout(timeout_ms: Option<u64>) -> Result<u64, ValidationError> {
    let timeout = timeout_ms.unwrap_or(DEFAULT_QUERY_TIMEOUT_MS);
    if timeout > MAX_QUERY_TIMEOUT_MS {
        return Err(ValidationError::TimeoutTooLong);
    }
    Ok(timeout)
}

// Complete validation pipeline combining all checks
fn validate_and_emit_events(cmd: ExecuteAnalyticsQuery) -> Result<Vec<DomainEvent>, ValidationError> {
    if !is_sql_safe(&cmd.sql) {
        return Err(ValidationError::UnsafeSQL);
    }
    if let Some(dataset) = &cmd.dataset_url {
        validate_dataset_url(dataset)?;
    }
    if estimate_query_complexity(&cmd.sql) > MAX_QUERY_COMPLEXITY {
        return Err(ValidationError::QueryTooComplex);
    }
    let timeout_ms = validate_timeout(cmd.timeout_ms)?;

    Ok(vec![DomainEvent::QueryStarted {
        query_id: Uuid::new_v4(),
        sql: cmd.sql,
        dataset_url: cmd.dataset_url,
        timeout_ms,
        started_at: Utc::now(),
    }])
}
```

This validation pipeline runs synchronously before any I/O, returning 400 Bad Request on failure.

## Decider pattern (fmodel-rust)

Ironstar uses fmodel-rust's Decider pattern as the minimal algebraic interface for functional event sourcing.
The Decider enforces pure decision logic with no async or side effects through its type signature.
This implementation realizes the theoretical foundations from `~/.claude/commands/preferences/domain-modeling.md` and directly addresses ironstar's explicit rejection of mutable aggregate patterns.

See `../decisions/fmodel-rust-adoption-evaluation.md` for the complete evaluation and migration path.

### Decider structure

```rust
use fmodel_rust::decider::Decider;

/// Decider for Todo aggregate
pub fn todo_decider<'a>() -> Decider<'a, TodoCommand, Option<TodoState>, TodoEvent, TodoError> {
    Decider {
        // Pure command validation: (command, state) → Result<Vec<Event>, Error>
        decide: Box::new(|command, state| match command {
            TodoCommand::Create { id, text } => {
                if state.is_some() {
                    // Failure event, not error - preserves event log completeness
                    Ok(vec![TodoEvent::NotCreated {
                        id: id.clone(),
                        reason: "Todo already exists".into()
                    }])
                } else {
                    Ok(vec![TodoEvent::Created {
                        id: id.clone(),
                        text: text.clone(),
                        created_at: Utc::now()
                    }])
                }
            }
            TodoCommand::Complete => {
                match state {
                    Some(s) if s.status == TodoStatus::Active => {
                        Ok(vec![TodoEvent::Completed { completed_at: Utc::now() }])
                    }
                    _ => Ok(vec![TodoEvent::NotCompleted {
                        reason: "Cannot complete: invalid state".into()
                    }])
                }
            }
            // ... other commands
        }),

        // Pure state evolution: (state, event) → state
        evolve: Box::new(|state, event| match event {
            TodoEvent::Created { id, text, created_at } => Some(TodoState {
                id: id.clone(),
                text: text.clone(),
                created_at: *created_at,
                status: TodoStatus::Active,
                completed_at: None,
            }),
            TodoEvent::NotCreated { .. } => state.clone(),  // Failure events preserve state
            TodoEvent::Completed { completed_at } => state.clone().map(|mut s| {
                s.status = TodoStatus::Completed;
                s.completed_at = Some(*completed_at);
                s
            }),
            TodoEvent::NotCompleted { .. } => state.clone(),
            // ... other events
        }),

        // Initial state factory
        initial_state: Box::new(|| None),
    }
}
```

### Mapping from previous Aggregate trait

The Decider pattern maps directly to the previous Aggregate trait design:

| Previous Pattern | Decider Equivalent | Purpose |
|------------------|-------------------|---------|
| `Aggregate::handle_command(state, cmd)` | `Decider::decide` | Command validation and event generation |
| `Aggregate::apply_event(state, event)` | `Decider::evolve` | Pure state transition from event |
| `Aggregate::State::default()` | `Decider::initial_state` | Factory for empty state |

The key difference: Decider enforces purity through boxed function pointers with no async, no mutable state, and referential transparency guaranteed by type signature.
This directly implements Hoffman's Law 7: "Work is a side effect."

> **Semantic foundation**: The `decide` function returns `Result<Vec<Event>, Error>`, which is a Kleisli arrow in the Result monad.
> Composition via `?` operator short-circuits on error.
> See [semantic-model.md § Kleisli composition](../core/semantic-model.md#command-handling-as-kleisli-composition).

### EventSourcedAggregate for async I/O orchestration

The async command handler uses fmodel-rust's `EventSourcedAggregate` to orchestrate I/O around the pure Decider:

```rust
use fmodel_rust::aggregate::EventSourcedAggregate;
use zenoh::Session;

/// Axum handler for Todo commands
pub async fn handle_todo_command(
    State(app_state): State<AppState>,
    Json(command): Json<TodoCommand>,
) -> impl IntoResponse {
    // Create EventSourcedAggregate wiring repository + decider
    let aggregate = EventSourcedAggregate::new(
        app_state.event_repository.clone(),
        todo_decider(),
    );

    // Handle command (fetch events, fold to state, decide, persist)
    match aggregate.handle(&command).await {
        Ok(events) => {
            // Publish to Zenoh for SSE broadcast (replaces tokio::broadcast)
            for (event, _version) in &events {
                let key = format!("events/Todo/{}", event.identifier());
                let payload = serde_json::to_vec(event).unwrap();
                app_state.zenoh.put(key, payload).await.ok();
            }
            (StatusCode::ACCEPTED, Json(events)).into_response()
        }
        Err(err) => {
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}
```

The `EventSourcedAggregate::handle()` method performs the following steps:
1. Fetch events from repository (async I/O)
2. Fold events using `evolve` to reconstruct state (pure)
3. Call `decide` with command and state (pure)
4. Persist new events via repository (async I/O)
5. Return events with version metadata

This pattern cleanly separates pure decision logic (Decider) from effectful I/O (EventRepository), with Zenoh handling event broadcast to SSE subscribers.

// Helper functions remain unchanged
fn deserialize_event<E: serde::de::DeserializeOwned>(stored: &StoredEvent) -> Option<E> {
    serde_json::from_value(stored.payload.clone()).ok()
}

fn serialize_event<E: serde::Serialize + EventType + DeciderType>(
    aggregate_id: &str,
    event: &E,
) -> DomainEvent {
    DomainEvent {
        event_id: Uuid::new_v4(),
        aggregate_type: E::decider_type().to_string(),
        aggregate_id: aggregate_id.to_string(),
        event_type: E::event_type().to_string(),
        payload: serde_json::to_value(event).expect("event serialization should not fail"),
        created_at: Utc::now(),
    }
}
```

## Decider testing with DeciderTestSpecification

fmodel-rust provides a built-in given/when/then DSL for declarative Decider testing without persistence or I/O.
The `DeciderTestSpecification` API provides fluent test composition with state verification.

```rust
#[cfg(test)]
mod tests {
    use fmodel_rust::decider::DeciderTestSpecification;
    use super::*;

    #[test]
    fn test_create_todo() {
        let id = TodoId::new();
        let text = TodoText::new("Buy groceries").unwrap();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])  // No prior events (fresh aggregate)
            .when(TodoCommand::Create { id: id.clone(), text: text.clone() })
            .then(vec![TodoEvent::Created {
                id: id.clone(),
                text: text.clone(),
                created_at: /* timestamp captured during test */
            }]);
    }

    #[test]
    fn test_create_existing_todo_emits_failure_event() {
        let id = TodoId::new();
        let text = TodoText::new("Buy groceries").unwrap();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id: id.clone(),
                    text: text.clone(),
                    created_at: Utc::now()
                }
            ])
            .when(TodoCommand::Create { id: id.clone(), text: text.clone() })
            .then(vec![TodoEvent::NotCreated {
                id: id.clone(),
                reason: "Todo already exists".into(),
            }]);
    }

    #[test]
    fn test_complete_active_todo() {
        let id = TodoId::new();
        let text = TodoText::new("Buy groceries").unwrap();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id: id.clone(),
                    text: text.clone(),
                    created_at: Utc::now()
                }
            ])
            .when(TodoCommand::Complete)
            .then_state(Some(TodoState {
                id,
                text: TodoText::new("Buy groceries").unwrap(),
                status: TodoStatus::Completed,
                completed_at: Some(/* timestamp captured */),
                created_at: /* timestamp captured */,
            }));
    }

    #[test]
    fn test_cannot_complete_deleted_todo() {
        let id = TodoId::new();
        let text = TodoText::new("Buy groceries").unwrap();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created { id: id.clone(), text: text.clone(), created_at: Utc::now() },
                TodoEvent::Deleted { deleted_at: Utc::now() },
            ])
            .when(TodoCommand::Complete)
            .then(vec![TodoEvent::NotCompleted {
                reason: "Cannot complete: invalid state".into(),
            }]);
    }
}
```

### DeciderTestSpecification API

The test DSL provides three primary assertion methods:

- `.then(events)`: Assert the command produced exactly these events
- `.then_state(state)`: Assert the final state after applying emitted events
- `.then_error(error)`: Assert the command produced a domain error (not failure event)

Note: ironstar's Decider pattern emits failure events rather than returning errors for business rule violations.
This preserves event log completeness (see Hoffman's Laws in `~/.claude/commands/preferences/event-sourcing.md`).
Use `.then(vec![FailureEvent])` for business rule violations, `.then_error()` only for programmer errors.

This pattern tests Decider logic in isolation without persistence or I/O.
The pure synchronous design makes tests fast, deterministic, and easy to reason about.

## EventRepository trait (fmodel-rust)

Ironstar implements fmodel-rust's `EventRepository<C, E, Version, Error>` trait for SQLite persistence.
This trait defines the async boundary between pure Decider logic and effectful storage operations.

```rust
use fmodel_rust::aggregate::EventRepository;

/// fmodel-rust EventRepository trait signature
pub trait EventRepository<C, E, Version, Error> {
    /// Fetch all events for the aggregate identified by command
    fn fetch_events(&self, command: &C) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;

    /// Persist new events and return with assigned versions
    fn save(&self, events: &[E]) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;

    /// Get the latest version for an aggregate (for optimistic locking)
    fn version_provider(&self, event: &E) -> impl Future<Output = Result<Option<Version>, Error>> + Send;
}
```

### SQLite implementation

Ironstar provides a custom SQLite implementation of this trait.
The implementation follows the schema from `../decisions/fmodel-rust-adoption-evaluation.md` section "Recommended SQLite schema" (lines 135-198).

See the evaluation document lines 232-315 for the complete `SqliteEventRepository` implementation pattern.

Key design points:
- `Version` type is `Uuid` for optimistic locking via `previous_id` unique constraint
- Events store as JSON with `event_version` metadata for schema evolution
- Global `offset INTEGER PRIMARY KEY AUTOINCREMENT` provides SSE Last-Event-ID semantics
- Triggers enforce event stream immutability and ordering constraints

### Additional query methods for SSE and projections

Beyond the fmodel-rust EventRepository trait, ironstar's SQLite implementation provides supplementary query methods:

```rust
impl SqliteEventRepository {
    /// Query all events (for projection rebuild)
    pub async fn query_all(&self) -> Result<Vec<StoredEvent>, Error>;

    /// Query events since sequence number (for SSE replay)
    pub async fn query_since_sequence(&self, since: i64) -> Result<Vec<StoredEvent>, Error>;

    /// Returns the earliest sequence number in the store.
    /// Used for bounded replay when snapshots are unavailable.
    pub async fn earliest_sequence(&self) -> Result<Option<i64>, Error>;

    /// Returns the latest sequence number in the store.
    /// Used for consistency checks and SSE Last-Event-ID validation.
    pub async fn latest_sequence(&self) -> Result<Option<i64>, Error>;
}
```

Note: Error, DomainEvent, and StoredEvent types are defined in event-sourcing-core.md.

## Related documentation

- `event-sourcing-core.md`: Master index and architecture overview
- `sse-connection-lifecycle.md`: SSE connection phases and debugging
- `event-replay-consistency.md`: Event replay and consistency boundaries
- `projection-patterns.md`: Projection caching strategies
- `performance-tuning.md`: Performance optimization strategies
- `../infrastructure/zenoh-event-bus.md`: Zenoh integration for distributed event bus
