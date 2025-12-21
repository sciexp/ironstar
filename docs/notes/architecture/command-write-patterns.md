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
async fn handle_add_todo(
    State(app_state): State<AppState>,
    ReadSignals(signals): ReadSignals<AddTodoCommand>,
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
fn validate_and_emit_events(cmd: AddTodoCommand) -> Result<Vec<DomainEvent>, ValidationError> {
    if cmd.text.is_empty() {
        return Err(ValidationError::EmptyText);
    }

    Ok(vec![DomainEvent::TodoAdded {
        id: Uuid::new_v4(),
        text: cmd.text,
        created_at: Utc::now(),
    }])
}
```

### Loading indicator integration

Frontend pattern (Datastar):

```html
<div id="main" data-init="@get('/feed')">
    <form data-on:submit.prevent="
        el.classList.add('loading');
        @post('/add-todo', {body: {text: $todoText}})
    ">
        <input data-model="$todoText" />
        <button type="submit">
            Add Todo
            <span data-show="el.closest('form').classList.contains('loading')">
                Saving...
            </span>
        </button>
    </form>

    <ul id="todo-list">
        <!-- SSE updates will morph this -->
    </ul>
</div>
```

Backend removes loading indicator via SSE update:

```rust
fn render_todo_list(state: &TodoListState) -> String {
    hypertext::html! {
        <ul id="todo-list">
            @for todo in &state.todos {
                <li id={"todo-" (&todo.id)}>{&todo.text}</li>
            }
        </ul>
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

Charts receive configuration updates via SSE using the same PatchSignals pattern.
For the complete ECharts Lit component implementation with Light DOM rendering and Open Props token integration, see `ds-echarts-integration-guide.md`.

```rust
async fn chart_data_sse(
    State(state): State<Arc<AppState>>,
    Path(chart_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        // Initial chart data from DuckDB
        let data = state.analytics.query_chart_data(&chart_id).await?;
        let option = build_echarts_option(&data);

        yield Ok(PatchSignals::new(
            serde_json::json!({"chartOption": option, "loading": false}).to_string()
        ).write_as_axum_sse_event());

        // Subscribe to real-time updates via Zenoh
        let mut sub = state.event_bus
            .subscribe(&format!("charts/{}/updates", chart_id))
            .await;

        while let Some(update) = sub.next().await {
            yield Ok(PatchSignals::new(
                serde_json::json!({"chartOption": build_echarts_option(&update)}).to_string()
            ).write_as_axum_sse_event());
        }
    };

    Sse::new(stream)
}
```

This pattern streams initial chart configuration followed by incremental updates triggered by Zenoh events.

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
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                aggregate_type TEXT NOT NULL,
                aggregate_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload JSON NOT NULL,
                metadata JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_aggregate
                ON events(aggregate_type, aggregate_id, sequence);
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
}
```

## Related documentation

- `event-sourcing-core.md`: Master index and architecture overview
- `sse-connection-lifecycle.md`: SSE connection phases and debugging
- `event-replay-consistency.md`: Event replay and consistency boundaries
- `projection-patterns.md`: Projection caching strategies
- `performance-tuning.md`: Performance optimization strategies
- `zenoh-event-bus.md`: Zenoh integration for distributed event bus
