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

## Aggregate trait

Aggregates are pure functions with no async or side effects.
External service calls happen in the command handler before invoking the aggregate.
This design, adapted from esrs, ensures aggregates are trivially testable and deterministic.

```rust
use std::error::Error;

/// Pure synchronous aggregate with no side effects.
///
/// Aggregates derive their state solely from their event stream.
/// Applying the same events in the same order always yields identical state.
pub trait Aggregate: Default + Send + Sync {
    /// Unique name for this aggregate type.
    /// Changing this breaks the link between existing aggregates and their events.
    const NAME: &'static str;

    /// Internal aggregate state, derived from events.
    type State: Default + Clone + Send + Sync;

    /// Commands represent requests to change state.
    type Command;

    /// Events represent facts that occurred in the domain.
    type Event: Clone;

    /// Domain errors from command validation.
    type Error: Error;

    /// Pure function: validates command against current state and emits events.
    /// No async, no I/O, no side effects.
    fn handle_command(state: &Self::State, cmd: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;

    /// Pure state transition: applies an event to produce new state.
    /// If the event cannot be applied (programmer error), this may panic.
    fn apply_event(state: Self::State, event: Self::Event) -> Self::State;

    /// Returns the current aggregate version (number of events applied).
    /// Used for optimistic locking when appending events to the event store.
    fn version(&self) -> u64;
}
```

The async command handler (in the application layer) orchestrates I/O around the pure aggregate:

```rust
use tokio::sync::broadcast;

/// Command error wrapping domain and infrastructure errors
#[derive(Debug)]
pub enum CommandError<E> {
    Domain(E),
    Persistence(sqlx::Error),
}

/// Async command handler orchestrating I/O around pure aggregate logic
pub async fn handle_command<A: Aggregate>(
    store: &SqliteEventStore,
    bus: &broadcast::Sender<StoredEvent>,
    aggregate_id: &str,
    command: A::Command,
) -> Result<Vec<StoredEvent>, CommandError<A::Error>> {
    // 1. Load events from store (async I/O)
    let events = store.query_aggregate(A::NAME, aggregate_id)
        .await
        .map_err(CommandError::Persistence)?;

    // 2. Reconstruct state by folding events
    let state = events.into_iter()
        .filter_map(|e| deserialize_event::<A>(&e))
        .fold(A::State::default(), A::apply_event);

    // 3. Handle command (pure, synchronous)
    let new_events = A::handle_command(&state, command)
        .map_err(CommandError::Domain)?;

    // 4. Persist new events (async I/O)
    let mut stored = Vec::with_capacity(new_events.len());
    for event in new_events {
        let sequence = store.append(serialize_event::<A>(aggregate_id, &event))
            .await
            .map_err(CommandError::Persistence)?;
        stored.push(StoredEvent { sequence, /* ... */ });
    }

    // 5. Publish to subscribers (fire and forget)
    for event in &stored {
        let _ = bus.send(event.clone());
    }

    Ok(stored)
}

// Helper functions (implementation details)
fn deserialize_event<A: Aggregate>(stored: &StoredEvent) -> Option<A::Event> {
    serde_json::from_value(stored.payload.clone()).ok()
}

fn serialize_event<A: Aggregate>(aggregate_id: &str, event: &A::Event) -> DomainEvent {
    // Convert to DomainEvent for storage
    todo!()
}
```

## Aggregate testing patterns

The given/when/then pattern provides declarative aggregate testing without persistence or I/O.
This pattern is adapted from cqrs-es TestFramework.

```rust
use std::fmt::Debug;
use std::marker::PhantomData;

/// Test framework for aggregate behavior verification
pub struct AggregateTestFramework<A: Aggregate> {
    _phantom: PhantomData<A>,
}

impl<A: Aggregate> AggregateTestFramework<A> {
    /// Start a test with existing events (aggregate has prior state)
    pub fn given(events: Vec<A::Event>) -> AggregateTestExecutor<A> {
        let state = events.into_iter().fold(A::State::default(), A::apply_event);
        AggregateTestExecutor { state, _phantom: PhantomData }
    }

    /// Start a test with no prior events (fresh aggregate)
    pub fn given_no_previous_events() -> AggregateTestExecutor<A> {
        AggregateTestExecutor {
            state: A::State::default(),
            _phantom: PhantomData,
        }
    }
}

/// Executes a command against the test state
pub struct AggregateTestExecutor<A: Aggregate> {
    state: A::State,
    _phantom: PhantomData<A>,
}

impl<A: Aggregate> AggregateTestExecutor<A> {
    /// Execute a command and capture the result for validation
    pub fn when(self, command: A::Command) -> AggregateTestResult<A> {
        let result = A::handle_command(&self.state, command);
        AggregateTestResult { result }
    }

    /// Add more events to the test state before executing command
    pub fn and(mut self, events: Vec<A::Event>) -> Self {
        for event in events {
            self.state = A::apply_event(self.state, event);
        }
        self
    }
}

/// Validates command results
pub struct AggregateTestResult<A: Aggregate> {
    result: Result<Vec<A::Event>, A::Error>,
}

impl<A: Aggregate> AggregateTestResult<A>
where
    A::Event: PartialEq + Debug,
{
    /// Assert the command produced the expected events
    pub fn then_expect_events(self, expected: Vec<A::Event>) {
        let events = self.result.unwrap_or_else(|err| {
            panic!("expected success, received error: '{err}'");
        });
        assert_eq!(events, expected);
    }
}

impl<A: Aggregate> AggregateTestResult<A>
where
    A::Error: PartialEq + Debug,
{
    /// Assert the command produced the expected error
    pub fn then_expect_error(self, expected: A::Error) {
        match self.result {
            Ok(events) => panic!("expected error, received events: '{events:?}'"),
            Err(err) => assert_eq!(err, expected),
        }
    }
}

impl<A: Aggregate> AggregateTestResult<A> {
    /// Assert the command produced an error with the expected message
    pub fn then_expect_error_message(self, expected_message: &str) {
        match self.result {
            Ok(events) => panic!("expected error, received events: '{events:?}'"),
            Err(err) => assert_eq!(err.to_string(), expected_message),
        }
    }

    /// Get the raw result for custom assertions
    pub fn inspect_result(self) -> Result<Vec<A::Event>, A::Error> {
        self.result
    }
}
```

Example usage with a concrete aggregate:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Assume Order aggregate with OrderCommand, OrderEvent, OrderError, OrderState

    #[test]
    fn test_order_placement() {
        AggregateTestFramework::<Order>::given_no_previous_events()
            .when(OrderCommand::Place {
                customer_id: "cust-123".into(),
                items: vec![LineItem { sku: "SKU-1".into(), qty: 2 }],
            })
            .then_expect_events(vec![
                OrderEvent::Placed {
                    customer_id: "cust-123".into(),
                    items: vec![LineItem { sku: "SKU-1".into(), qty: 2 }],
                }
            ]);
    }

    #[test]
    fn test_cannot_ship_unpaid_order() {
        AggregateTestFramework::<Order>::given(vec![
            OrderEvent::Placed {
                customer_id: "cust-123".into(),
                items: vec![LineItem { sku: "SKU-1".into(), qty: 2 }],
            },
        ])
        .when(OrderCommand::Ship)
        .then_expect_error(OrderError::NotPaid);
    }

    #[test]
    fn test_complete_order_flow() {
        AggregateTestFramework::<Order>::given_no_previous_events()
            .when(OrderCommand::Place { /* ... */ })
            .then_expect_events(vec![OrderEvent::Placed { /* ... */ }]);

        // Test with accumulated state
        AggregateTestFramework::<Order>::given(vec![
            OrderEvent::Placed { /* ... */ },
        ])
        .and(vec![
            OrderEvent::Paid { amount: 100 },
        ])
        .when(OrderCommand::Ship)
        .then_expect_events(vec![OrderEvent::Shipped]);
    }
}
```

This pattern tests aggregate logic in isolation without persistence or I/O.
The pure synchronous design makes tests fast, deterministic, and easy to reason about.

## Event store trait

```rust
use async_trait::async_trait;
use sqlx::SqlitePool;

#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append event and return assigned sequence number
    async fn append(&self, event: DomainEvent) -> Result<i64, Error>;

    /// Query all events (for projection rebuild)
    async fn query_all(&self) -> Result<Vec<StoredEvent>, Error>;

    /// Query events since sequence number (for SSE replay)
    async fn query_since_sequence(&self, since: i64) -> Result<Vec<StoredEvent>, Error>;

    /// Query events for specific aggregate (for debugging)
    async fn query_aggregate(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<Vec<StoredEvent>, Error>;

    /// Returns the earliest sequence number in the store.
    /// Used for bounded replay when snapshots are unavailable.
    /// Returns None if the store is empty.
    async fn earliest_sequence(&self) -> Result<Option<i64>, Error>;

    /// Returns the latest sequence number in the store.
    /// Used for consistency checks and SSE Last-Event-ID validation.
    /// Returns None if the store is empty.
    async fn latest_sequence(&self) -> Result<Option<i64>, Error>;
}
// Note: Error, DomainEvent, and StoredEvent types are defined in event-sourcing-core.md

/// SQLite implementation
pub struct SqliteEventStore {
    pool: sqlx::SqlitePool,
}

impl SqliteEventStore {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = sqlx::SqlitePool::connect(database_url).await?;

        // Create table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                global_sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                id TEXT NOT NULL UNIQUE,
                aggregate_type TEXT NOT NULL,
                aggregate_id TEXT NOT NULL,
                aggregate_sequence INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                event_version TEXT NOT NULL DEFAULT '1.0.0',
                payload TEXT NOT NULL,
                metadata TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                UNIQUE(aggregate_type, aggregate_id, aggregate_sequence)
            ) STRICT;
            CREATE INDEX IF NOT EXISTS idx_aggregate
                ON events(aggregate_type, aggregate_id, aggregate_sequence);
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn append(&self, event: DomainEvent) -> Result<i64, Error> {
        let stored = StoredEvent::from_domain(event);

        let result = sqlx::query(
            r#"
            INSERT INTO events (aggregate_type, aggregate_id, event_type, payload, metadata)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&stored.aggregate_type)
        .bind(&stored.aggregate_id)
        .bind(&stored.event_type)
        .bind(&stored.payload)
        .bind(&stored.metadata)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn query_all(&self) -> Result<Vec<StoredEvent>, Error> {
        let events = sqlx::query_as::<_, StoredEvent>(
            "SELECT * FROM events ORDER BY sequence ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    async fn query_since_sequence(&self, since: i64) -> Result<Vec<StoredEvent>, Error> {
        let events = sqlx::query_as::<_, StoredEvent>(
            "SELECT * FROM events WHERE sequence > ? ORDER BY sequence ASC",
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    async fn query_aggregate(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<Vec<StoredEvent>, Error> {
        let events = sqlx::query_as::<_, StoredEvent>(
            r#"
            SELECT * FROM events
            WHERE aggregate_type = ? AND aggregate_id = ?
            ORDER BY sequence ASC
            "#,
        )
        .bind(aggregate_type)
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    async fn earliest_sequence(&self) -> Result<Option<i64>, Error> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT MIN(sequence) FROM events",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(seq,)| seq))
    }

    async fn latest_sequence(&self) -> Result<Option<i64>, Error> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT MAX(sequence) FROM events",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(seq,)| seq))
    }
}
```

## Related documentation

- `event-sourcing-core.md`: Master index and architecture overview
- `sse-connection-lifecycle.md`: SSE connection phases and debugging
- `event-replay-consistency.md`: Event replay and consistency boundaries
- `projection-patterns.md`: Projection caching strategies
- `performance-tuning.md`: Performance optimization strategies
- `../infrastructure/zenoh-event-bus.md`: Zenoh integration for distributed event bus
