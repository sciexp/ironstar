---
title: CQRS bootstrapping guide
---

# CQRS bootstrapping guide

This guide provides step-by-step instructions for implementing the minimal CQRS pipeline in ironstar, following the tracer bullet methodology.
The goal is to build the thinnest possible vertical slice that demonstrates command handling, event persistence, event distribution, and SSE-driven UI updates.

## What we're building

The minimal CQRS pipeline consists of:

1. A single aggregate type with one command and one event
2. Event persistence to SQLite with optimistic locking
3. Event distribution via Zenoh embedded mode
4. SSE streaming to browser via datastar
5. Reactive UI update using datastar signals

This establishes the end-to-end data flow: user action → command → event → persistence → broadcast → SSE → UI update.

## Prerequisites

Before starting implementation, ensure:

- Rust toolchain installed via `rust-toolchain.toml`
- Nix flake development environment active
- SQLite available (via sqlx)
- Understanding of the architectural principles in `docs/notes/architecture/core/design-principles.md`
- Familiarity with the technology choices in `docs/notes/architecture/core/architecture-decisions.md`

## Phase 1: Domain foundation

Build the pure functional core with no side effects.

### Step 1.1: Define value objects

Create minimal value objects for aggregate identity and domain concepts.

```rust
// src/domain/todo/values.rs

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

/// TodoId is a UUID-based aggregate identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Serialize, Deserialize)]
pub struct TodoId(uuid::Uuid);

impl TodoId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

/// TodoText is a non-empty string with maximum length
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoText(String);

impl TodoText {
    pub fn new(text: String) -> Result<Self, &'static str> {
        if text.is_empty() {
            Err("Todo text cannot be empty")
        } else if text.len() > 500 {
            Err("Todo text exceeds maximum length")
        } else {
            Ok(Self(text))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**Validation checkpoint**: Value objects enforce invariants and fail construction on invalid input.

### Step 1.2: Define events

Events are immutable facts that have occurred.
Use sum types to represent all possible events for the aggregate.

```rust
// src/domain/todo/events.rs

use serde::{Deserialize, Serialize};
use super::values::{TodoId, TodoText};

/// All events that can occur to a Todo aggregate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TodoEvent {
    Created { id: TodoId, text: TodoText },
    Completed { id: TodoId },
}

impl TodoEvent {
    pub fn aggregate_id(&self) -> &TodoId {
        match self {
            TodoEvent::Created { id, .. } => id,
            TodoEvent::Completed { id } => id,
        }
    }
}
```

**Validation checkpoint**: Events serialize to JSON with type discriminators for event store persistence.

### Step 1.3: Define commands

Commands represent user intent and may be rejected.

```rust
// src/domain/todo/commands.rs

use super::values::{TodoId, TodoText};

/// All commands that can be issued to a Todo aggregate
#[derive(Debug, Clone)]
pub enum TodoCommand {
    Create { id: TodoId, text: TodoText },
    Complete { id: TodoId },
}
```

**Validation checkpoint**: Commands contain validated value objects from step 1.1.

### Step 1.4: Define aggregate state

Aggregate state is derived from events.

```rust
// src/domain/todo/aggregate.rs

use super::values::TodoId;
use super::events::TodoEvent;

/// Todo aggregate state
#[derive(Debug, Clone, PartialEq)]
pub enum TodoState {
    NotCreated,
    Active { id: TodoId, text: String },
    Completed { id: TodoId, text: String },
}

impl Default for TodoState {
    fn default() -> Self {
        Self::NotCreated
    }
}

impl TodoState {
    /// Apply an event to transition state
    pub fn apply(self, event: &TodoEvent) -> Self {
        match (self, event) {
            (TodoState::NotCreated, TodoEvent::Created { id, text }) => {
                TodoState::Active {
                    id: id.clone(),
                    text: text.as_str().to_string(),
                }
            }
            (TodoState::Active { id, text }, TodoEvent::Completed { .. }) => {
                TodoState::Completed {
                    id,
                    text,
                }
            }
            (state, _) => state, // Invalid transitions are no-ops
        }
    }
}
```

**Validation checkpoint**: State transitions are pure functions with no side effects.

### Step 1.5: Implement command handler

Command handlers are pure synchronous functions that produce events or errors.

```rust
// src/domain/todo/aggregate.rs (continued)

use super::commands::TodoCommand;

#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    #[error("Todo already exists")]
    AlreadyExists,
    #[error("Todo not found")]
    NotFound,
    #[error("Todo already completed")]
    AlreadyCompleted,
}

impl TodoState {
    /// Handle a command and produce events
    pub fn handle(
        &self,
        command: TodoCommand,
    ) -> Result<Vec<TodoEvent>, TodoError> {
        match (self, command) {
            (TodoState::NotCreated, TodoCommand::Create { id, text }) => {
                Ok(vec![TodoEvent::Created { id, text }])
            }
            (TodoState::Active { .. }, TodoCommand::Create { .. }) => {
                Err(TodoError::AlreadyExists)
            }
            (TodoState::Active { id, .. }, TodoCommand::Complete { .. }) => {
                Ok(vec![TodoEvent::Completed { id: id.clone() }])
            }
            (TodoState::Completed { .. }, TodoCommand::Complete { .. }) => {
                Err(TodoError::AlreadyCompleted)
            }
            (TodoState::NotCreated, TodoCommand::Complete { .. }) => {
                Err(TodoError::NotFound)
            }
            _ => Ok(vec![]),
        }
    }
}
```

**Validation checkpoint**: Write unit tests for command handling using the given/when/then pattern from cqrs-es TestFramework.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_todo() {
        // Given: empty state
        let state = TodoState::default();

        // When: create command
        let id = TodoId::new();
        let text = TodoText::new("Buy milk".to_string()).unwrap();
        let command = TodoCommand::Create { id: id.clone(), text: text.clone() };

        // Then: created event emitted
        let events = state.handle(command).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], TodoEvent::Created { .. }));

        // And: state transitions to Active
        let new_state = state.apply(&events[0]);
        assert!(matches!(new_state, TodoState::Active { .. }));
    }
}
```

**Phase 1 complete**: Pure domain logic implemented with no async, no IO, no side effects.

## Phase 2: Event store and persistence

Implement durable event storage with SQLite.

### Step 2.1: Define event store schema

Create the SQLite events table following sqlite-es patterns.

```sql
-- migrations/001_create_events_table.sql

CREATE TABLE IF NOT EXISTS events (
    -- Global sequence for Last-Event-ID in SSE
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Aggregate identity
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,

    -- Event sequence within aggregate for optimistic locking
    sequence INTEGER NOT NULL,

    -- Event data
    event_type TEXT NOT NULL,
    payload JSON NOT NULL,

    -- Metadata
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Unique constraint for optimistic locking
    UNIQUE(aggregate_type, aggregate_id, sequence)
);

CREATE INDEX idx_events_aggregate ON events(aggregate_type, aggregate_id);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_created ON events(created_at);
```

**Validation checkpoint**: Run migration with `sqlx migrate run`.

### Step 2.2: Define EventStore trait

Abstract event persistence behind a trait for testability.

```rust
// src/infrastructure/event_store.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: i64,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub sequence: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Optimistic locking failure: expected sequence {expected}, got {actual}")]
    OptimisticLockingFailure { expected: i64, actual: i64 },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to the store with optimistic locking
    async fn append_events(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        expected_sequence: i64,
        events: Vec<serde_json::Value>,
    ) -> Result<Vec<i64>, EventStoreError>;

    /// Load all events for an aggregate
    async fn load_events(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;

    /// Load events starting from a global sequence (for SSE replay)
    async fn load_events_from_sequence(
        &self,
        from_sequence: i64,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;
}
```

**Validation checkpoint**: Trait compiles and matches sqlx async patterns.

### Step 2.3: Implement SQLite event store

Concrete implementation with optimistic locking.

```rust
// src/infrastructure/sqlite_event_store.rs

use async_trait::async_trait;
use sqlx::SqlitePool;
use super::event_store::{EventStore, EventStoreError, StoredEvent};

pub struct SqliteEventStore {
    pool: SqlitePool,
}

impl SqliteEventStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn append_events(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        expected_sequence: i64,
        events: Vec<serde_json::Value>,
    ) -> Result<Vec<i64>, EventStoreError> {
        let mut tx = self.pool.begin().await?;

        // Check current sequence
        let current_sequence: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(sequence) FROM events WHERE aggregate_type = ? AND aggregate_id = ?"
        )
        .bind(aggregate_type)
        .bind(aggregate_id)
        .fetch_optional(&mut *tx)
        .await?;

        let current = current_sequence.unwrap_or(-1);

        if current != expected_sequence {
            return Err(EventStoreError::OptimisticLockingFailure {
                expected: expected_sequence,
                actual: current,
            });
        }

        // Insert events
        let mut event_ids = Vec::new();
        for (i, event) in events.iter().enumerate() {
            let sequence = expected_sequence + 1 + i as i64;
            let event_type = event.get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    EventStoreError::Serialization(
                        serde_json::Error::custom("Missing event type")
                    )
                })?;

            let result = sqlx::query(
                "INSERT INTO events (aggregate_type, aggregate_id, sequence, event_type, payload)
                 VALUES (?, ?, ?, ?, ?)
                 RETURNING id"
            )
            .bind(aggregate_type)
            .bind(aggregate_id)
            .bind(sequence)
            .bind(event_type)
            .bind(event)
            .fetch_one(&mut *tx)
            .await?;

            let id: i64 = result.get("id");
            event_ids.push(id);
        }

        tx.commit().await?;
        Ok(event_ids)
    }

    async fn load_events(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let events = sqlx::query_as!(
            StoredEvent,
            "SELECT id, aggregate_type, aggregate_id, sequence, event_type,
                    payload as \"payload: serde_json::Value\",
                    metadata as \"metadata: Option<serde_json::Value>\",
                    created_at
             FROM events
             WHERE aggregate_type = ? AND aggregate_id = ?
             ORDER BY sequence ASC",
            aggregate_type,
            aggregate_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    async fn load_events_from_sequence(
        &self,
        from_sequence: i64,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let events = sqlx::query_as!(
            StoredEvent,
            "SELECT id, aggregate_type, aggregate_id, sequence, event_type,
                    payload as \"payload: serde_json::Value\",
                    metadata as \"metadata: Option<serde_json::Value>\",
                    created_at
             FROM events
             WHERE id > ?
             ORDER BY id ASC",
            from_sequence
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }
}
```

**Validation checkpoint**: Write integration tests for event store with in-memory SQLite.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_optimistic_locking() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();

        let store = SqliteEventStore::new(pool);

        let event = serde_json::json!({
            "type": "created",
            "id": "test-id",
            "text": "Test todo"
        });

        // First append succeeds
        store.append_events("Todo", "1", -1, vec![event.clone()])
            .await
            .unwrap();

        // Second append with wrong expected sequence fails
        let result = store.append_events("Todo", "1", -1, vec![event]).await;
        assert!(matches!(result, Err(EventStoreError::OptimisticLockingFailure { .. })));
    }
}
```

**Phase 2 complete**: Events persist to SQLite with optimistic locking preventing concurrent modification.

## Phase 3: Event bus and distribution

Implement event distribution via Zenoh embedded mode.

### Step 3.1: Configure Zenoh embedded mode

Create minimal Zenoh configuration for single-node operation.

```rust
// src/infrastructure/event_bus.rs

use zenoh::prelude::r#async::*;

pub async fn create_embedded_zenoh() -> Result<Session, Box<dyn std::error::Error>> {
    let mut config = Config::default();

    // Embedded mode: no network listeners
    config.listen.set_endpoints(vec![]).unwrap();

    // Disable scouting to prevent network discovery
    config.scouting.multicast.set_enabled(Some(false)).unwrap();

    let session = zenoh::open(config).res().await?;
    Ok(session)
}
```

**Validation checkpoint**: Zenoh session creation succeeds with no network ports opened.

### Step 3.2: Define event bus abstraction

Abstract pub/sub behind a trait.

```rust
// src/infrastructure/event_bus.rs (continued)

use async_trait::async_trait;

#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to a key expression
    async fn publish(
        &self,
        key_expr: &str,
        payload: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Subscribe to events matching a key expression
    async fn subscribe(
        &self,
        key_expr: &str,
    ) -> Result<Box<dyn Stream<Item = Vec<u8>> + Send + Unpin>, Box<dyn std::error::Error>>;
}
```

**Validation checkpoint**: Trait supports key expression patterns like `events/Todo/**`.

### Step 3.3: Implement Zenoh event bus

Concrete implementation using Zenoh session.

```rust
// src/infrastructure/zenoh_event_bus.rs

use async_trait::async_trait;
use zenoh::prelude::r#async::*;
use futures::stream::Stream;
use super::event_bus::EventBus;

pub struct ZenohEventBus {
    session: Session,
}

impl ZenohEventBus {
    pub fn new(session: Session) -> Self {
        Self { session }
    }
}

#[async_trait]
impl EventBus for ZenohEventBus {
    async fn publish(
        &self,
        key_expr: &str,
        payload: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.session
            .put(key_expr, payload)
            .res()
            .await?;
        Ok(())
    }

    async fn subscribe(
        &self,
        key_expr: &str,
    ) -> Result<Box<dyn Stream<Item = Vec<u8>> + Send + Unpin>, Box<dyn std::error::Error>> {
        let subscriber = self.session
            .declare_subscriber(key_expr)
            .res()
            .await?;

        let stream = subscriber.map(|sample| sample.value.payload.contiguous().to_vec());
        Ok(Box::new(stream))
    }
}
```

**Validation checkpoint**: Publish/subscribe works in same process without network.

### Step 3.4: Define key expression patterns

Document key expression conventions for CQRS routing.

```rust
// src/infrastructure/event_bus.rs (continued)

/// Key expression patterns for event routing
pub mod key_patterns {
    /// All events for a specific aggregate type
    /// Example: events/Todo/**
    pub fn aggregate_type(type_name: &str) -> String {
        format!("events/{}/**", type_name)
    }

    /// Events for a specific aggregate instance
    /// Example: events/Todo/123
    pub fn aggregate_instance(type_name: &str, id: &str) -> String {
        format!("events/{}/{}", type_name, id)
    }

    /// Events for a specific session
    /// Example: events/session/abc-def-123/**
    pub fn session(session_id: &str) -> String {
        format!("events/session/{}/**", session_id)
    }
}
```

**Validation checkpoint**: Key expressions match Zenoh syntax and support hierarchical filtering.

**Phase 3 complete**: Events can be published and subscribed via Zenoh with key expression filtering.

## Phase 4: Web layer integration

Connect the CQRS pipeline to the web layer with axum and datastar.

### Step 4.1: Define command endpoint

Create an axum handler that accepts commands via POST.

```rust
// src/web/handlers/todo.rs

use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use crate::domain::todo::{
    commands::TodoCommand,
    values::{TodoId, TodoText},
};
use crate::application::todo_service::TodoService;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TodoCommandRequest {
    Create { text: String },
    Complete { id: String },
}

pub async fn handle_command(
    State(service): State<TodoService>,
    Json(request): Json<TodoCommandRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let command = match request {
        TodoCommandRequest::Create { text } => {
            let id = TodoId::new();
            let text = TodoText::new(text)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            TodoCommand::Create { id, text }
        }
        TodoCommandRequest::Complete { id } => {
            let id = id.parse::<uuid::Uuid>()
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            TodoCommand::Complete { id: TodoId::from(id) }
        }
    };

    service.handle_command(command)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
```

**Validation checkpoint**: POST requests trigger command handling.

### Step 4.2: Implement TodoService

Application service that coordinates event store and event bus.

```rust
// src/application/todo_service.rs

use crate::domain::todo::{
    aggregate::TodoState,
    commands::TodoCommand,
    events::TodoEvent,
};
use crate::infrastructure::{
    event_store::EventStore,
    event_bus::EventBus,
};

pub struct TodoService {
    event_store: Arc<dyn EventStore>,
    event_bus: Arc<dyn EventBus>,
}

impl TodoService {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self { event_store, event_bus }
    }

    pub async fn handle_command(
        &self,
        command: TodoCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Extract aggregate ID from command
        let aggregate_id = match &command {
            TodoCommand::Create { id, .. } => id.to_string(),
            TodoCommand::Complete { id } => id.to_string(),
        };

        // Load events and reconstruct state
        let stored_events = self.event_store
            .load_events("Todo", &aggregate_id)
            .await?;

        let mut state = TodoState::default();
        let mut last_sequence = -1i64;

        for stored in &stored_events {
            let event: TodoEvent = serde_json::from_value(stored.payload.clone())?;
            state = state.apply(&event);
            last_sequence = stored.sequence;
        }

        // Handle command
        let new_events = state.handle(command)?;

        // Serialize events
        let payloads: Vec<_> = new_events.iter()
            .map(|e| serde_json::to_value(e))
            .collect::<Result<_, _>>()?;

        // Persist events
        let event_ids = self.event_store
            .append_events("Todo", &aggregate_id, last_sequence, payloads.clone())
            .await?;

        // Publish events to bus
        for (i, payload) in payloads.iter().enumerate() {
            let key = format!("events/Todo/{}", aggregate_id);
            let bytes = serde_json::to_vec(payload)?;
            self.event_bus.publish(&key, bytes).await?;
        }

        Ok(())
    }
}
```

**Validation checkpoint**: Commands persist events and publish to Zenoh.

### Step 4.3: Implement SSE endpoint

Create an axum handler that streams events via SSE using datastar-rust.

```rust
// src/web/handlers/sse.rs

use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;
use crate::infrastructure::event_bus::EventBus;

pub async fn todo_events_stream(
    State(event_bus): State<Arc<dyn EventBus>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = event_bus
        .subscribe("events/Todo/**")
        .await
        .unwrap();

    let sse_stream = stream.map(|payload| {
        let json_str = String::from_utf8_lossy(&payload);
        Ok(Event::default().data(json_str))
    });

    Sse::new(sse_stream)
}
```

**Validation checkpoint**: SSE endpoint streams events when todos are created/completed.

### Step 4.4: Create HTML page with datastar

Build a minimal HTML page that sends commands and displays events.

```rust
// src/web/pages/todo.rs

use hypertext::{html_elements, maud_move, Raw, Renderable};

pub fn todo_page() -> impl Renderable {
    maud_move!(
        (Raw("<!DOCTYPE html>"))
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Ironstar Todo" }
                script type="module" src="/static/dist/bundle.js" {}
            }
            body {
                div
                    data-store=r#"{ todos: [] }"#
                    data-on-load=r#"$$get('/api/sse/todos')"# {

                    h1 { "Todos" }

                    form data-on-submit=r#"$$post('/api/commands/todo', { type: 'create', text: $newTodoText })"# {
                        input
                            type="text"
                            data-model="newTodoText"
                            placeholder="New todo...";
                        button type="submit" { "Add" }
                    }

                    ul data-text="$todos.length" {}
                }
            }
        }
    )
}
```

**Validation checkpoint**: Page loads, form submits create commands, SSE connection established.

### Step 4.5: Wire up routes

Connect all handlers in axum router.

```rust
// src/main.rs

use axum::{
    routing::{get, post},
    Router,
};

#[tokio::main]
async fn main() {
    // Initialize infrastructure
    let pool = SqlitePool::connect("sqlite:ironstar.db").await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let event_store = Arc::new(SqliteEventStore::new(pool));
    let zenoh_session = create_embedded_zenoh().await.unwrap();
    let event_bus = Arc::new(ZenohEventBus::new(zenoh_session));

    let todo_service = TodoService::new(
        Arc::clone(&event_store),
        Arc::clone(&event_bus),
    );

    // Build router
    let app = Router::new()
        .route("/", get(|| async { todo_page().render() }))
        .route("/api/commands/todo", post(handle_command))
        .route("/api/sse/todos", get(todo_events_stream))
        .with_state(todo_service)
        .with_state(event_bus);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

**Validation checkpoint**: Server starts, handles commands, streams events.

**Phase 4 complete**: Full CQRS pipeline from browser to database and back via SSE.

## Phase 5: Verification

Validate the tracer bullet works end-to-end.

### Step 5.1: Manual verification

1. Start the server: `cargo run`
2. Open browser to `http://localhost:3000`
3. Open browser DevTools Network tab, filter SSE
4. Type a todo in the input field and submit
5. Verify:
   - POST request to `/api/commands/todo` returns 200
   - SSE connection to `/api/sse/todos` receives event
   - Todo appears in UI (once signal projection implemented)

### Step 5.2: Database verification

Check that events are persisted correctly:

```sql
sqlite3 ironstar.db "SELECT * FROM events ORDER BY id DESC LIMIT 5;"
```

Expected output:
- `aggregate_type` is "Todo"
- `event_type` is "created" or "completed"
- `payload` is valid JSON
- `sequence` increments correctly per aggregate

### Step 5.3: Integration test

Write an integration test that exercises the full pipeline:

```rust
#[tokio::test]
async fn test_todo_creation_pipeline() {
    // Setup
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let event_store = Arc::new(SqliteEventStore::new(pool));
    let zenoh_session = create_embedded_zenoh().await.unwrap();
    let event_bus = Arc::new(ZenohEventBus::new(zenoh_session));

    let service = TodoService::new(
        Arc::clone(&event_store),
        Arc::clone(&event_bus),
    );

    // Subscribe before issuing command
    let mut subscription = event_bus
        .subscribe("events/Todo/**")
        .await
        .unwrap();

    // Issue command
    let id = TodoId::new();
    let text = TodoText::new("Test todo".to_string()).unwrap();
    let command = TodoCommand::Create { id: id.clone(), text };

    service.handle_command(command).await.unwrap();

    // Verify event published
    let event_payload = subscription.next().await.unwrap();
    let event: TodoEvent = serde_json::from_slice(&event_payload).unwrap();

    assert!(matches!(event, TodoEvent::Created { .. }));

    // Verify event persisted
    let stored = event_store.load_events("Todo", &id.to_string())
        .await
        .unwrap();

    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].event_type, "created");
}
```

**Validation checkpoint**: Integration test passes, proving command → event → persistence → broadcast → subscription works.

### Step 5.4: Error handling verification

Test error paths to ensure proper handling:

1. Submit empty todo text → expect 400 Bad Request
2. Complete non-existent todo → expect command rejection
3. Create duplicate todo → expect optimistic locking failure
4. Disconnect SSE, issue command, reconnect → verify Last-Event-ID replay

**Phase 5 complete**: The tracer bullet is operational and verified.

## Common pitfalls

### Pitfall 1: Mixing async in pure domain logic

**Problem**: Adding `async` to `TodoState::handle()` method.

**Why it's wrong**: Domain logic should be pure and synchronous. Async is for effects (IO, network, etc).

**Fix**: Keep command handlers synchronous. Move async operations to application services.

### Pitfall 2: Forgetting optimistic locking check

**Problem**: Appending events without checking expected sequence.

**Why it's wrong**: Concurrent modifications can produce invalid state.

**Fix**: Always load current sequence, validate against expected, use database transaction.

### Pitfall 3: Not using key expressions for filtering

**Problem**: Broadcasting all events to all subscribers with `events/*`.

**Why it's wrong**: Clients receive irrelevant events, wasting bandwidth.

**Fix**: Use hierarchical key expressions like `events/Todo/{id}` and subscribe with wildcards like `events/Todo/**`.

### Pitfall 4: Serialization errors not handled

**Problem**: Assuming `serde_json::to_value()` always succeeds.

**Why it's wrong**: Serialization can fail (e.g., non-UTF8 strings).

**Fix**: Propagate serialization errors through Result types, never unwrap.

### Pitfall 5: Missing Last-Event-ID support

**Problem**: SSE reconnection replays all events from the beginning.

**Why it's wrong**: Clients see duplicate events after brief disconnections.

**Fix**: Use the global `id` column from events table for Last-Event-ID. See `docs/notes/architecture/cqrs/sse-connection-lifecycle.md` for complete implementation.

### Pitfall 6: Not validating commands at boundary

**Problem**: Accepting raw strings in command handler without validation.

**Why it's wrong**: Invalid data propagates into domain layer.

**Fix**: Parse and validate inputs at the web layer boundary using value objects before constructing commands.

### Pitfall 7: Blocking Zenoh operations

**Problem**: Using synchronous Zenoh API in async context.

**Why it's wrong**: Blocks tokio executor threads.

**Fix**: Use `zenoh::prelude::r#async::*` and `.res().await` pattern consistently.

## Next steps

Once the tracer bullet works, expand the implementation:

1. **Add projections**: Implement read models that subscribe to events and build materialized views
   - See `docs/notes/architecture/cqrs/projection-patterns.md`

2. **Add sessions**: Implement per-session Zenoh key expressions for user-specific updates
   - See `docs/notes/architecture/infrastructure/session-management.md`

3. **Add authentication**: Implement OAuth integration with GitHub
   - See `docs/notes/architecture/decisions/oauth-authentication.md`

4. **Add analytics**: Integrate DuckDB for OLAP queries
   - See `docs/notes/architecture/infrastructure/analytics-cache-architecture.md`

5. **Add event upcasting**: Implement schema evolution for events
   - Study `~/projects/rust-workspace/event_sourcing.rs` Upcaster pattern

6. **Add aggregate testing framework**: Build the TestFramework DSL
   - Study `~/projects/rust-workspace/cqrs-es` TestFramework implementation

7. **Scale to multi-crate**: Extract domain, application, infrastructure, web into separate crates
   - See `docs/notes/architecture/core/crate-architecture.md`

## Related documentation

- Design principles: `docs/notes/architecture/core/design-principles.md`
- Architecture decisions: `docs/notes/architecture/core/architecture-decisions.md`
- Event sourcing core: `docs/notes/architecture/cqrs/event-sourcing-core.md`
- Zenoh event bus: `docs/notes/architecture/infrastructure/zenoh-event-bus.md`
- SSE connection lifecycle: `docs/notes/architecture/cqrs/sse-connection-lifecycle.md`
- Performance tuning: `docs/notes/architecture/cqrs/performance-tuning.md`
