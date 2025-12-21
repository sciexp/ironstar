# Event sourcing + projection + SSE pipeline patterns

Design patterns for the event sourcing, projection, and SSE pipeline in ironstar, optimized for Rust and Datastar best practices.

## Architecture diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         Browser (Datastar)                                │
│  ┌────────────────────┐                    ┌──────────────────────┐      │
│  │ Long-lived SSE GET │ ←──────────────────│ Short-lived POST     │      │
│  │ @get('/feed')      │    (read path)     │ @post('/command')    │      │
│  │                    │                    │                      │      │
│  │ Reconnects with    │                    │ Immediate response   │      │
│  │ Last-Event-ID      │                    │ + loading indicator  │      │
│  └─────────┬──────────┘                    └──────────┬───────────┘      │
│            │                                          │                  │
└────────────┼──────────────────────────────────────────┼──────────────────┘
             │                                          │
             │ SSE: text/event-stream                   │ POST: application/json
             │ id: <sequence_number>                    │
             │ data: elements <html>                    │
             │                                          │
┌────────────┼──────────────────────────────────────────┼──────────────────┐
│            ▼                                          ▼                  │
│  ┌──────────────────────────────┐         ┌─────────────────────────┐   │
│  │     SSE Handler (axum)       │         │  Command Handler (axum) │   │
│  │  - Extract Last-Event-ID     │         │  - Validate command     │   │
│  │  - Subscribe to broadcast    │         │  - Return 202 Accepted  │   │
│  │  - Replay missed events      │         └────────────┬────────────┘   │
│  │  - Stream future updates     │                      │                │
│  └──────────┬───────────────────┘                      │                │
│             │ tokio::sync::broadcast                   │                │
│             │ ::Receiver                               │                │
│             │                                          ▼                │
│  ┌──────────┴───────────────────┐         ┌─────────────────────────┐   │
│  │   Projection (in-memory)     │         │  Application Layer      │   │
│  │  - Subscribe to events       │ ◄───────│  - Emit events          │   │
│  │  - Maintain read model       │         │  - Pure logic           │   │
│  │  - Serve queries             │         └────────────┬────────────┘   │
│  └──────────────────────────────┘                      │                │
│                                                        │                │
│                                             ┌──────────▼────────────┐   │
│                                             │  Event Store (SQLite) │   │
│                                             │  - Append event       │   │
│                                             │  - Generate sequence  │   │
│                                             │  - Persist to WAL     │   │
│                                             └──────────┬────────────┘   │
│                                                        │                │
│                                             ┌──────────▼────────────┐   │
│                                             │ tokio::sync::broadcast│   │
│                                             │ ::Sender              │   │
│                                             │ - Fan-out to N subs   │   │
│                                             └───────────────────────┘   │
│                                                                          │
│                             Axum Server Process                          │
└──────────────────────────────────────────────────────────────────────────┘

Legend:
  ──►  Data flow
  ◄──  Subscribes to
```

## Design decisions and rationale

### 1. Event replay strategy

**Decision: Use event sequence numbers as SSE `id` field, enable automatic replay via `Last-Event-ID` header.**

#### Pattern

Each event stored in SQLite gets a monotonically increasing sequence number.
When emitting SSE events, set the `id` field to the sequence number.
The browser automatically sends `Last-Event-ID` header on reconnection.
The SSE handler replays all events since that ID before streaming new ones.

#### Implementation

```rust
// Event store schema
CREATE TABLE events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSON NOT NULL,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_aggregate (aggregate_type, aggregate_id, sequence)
);

// SSE handler signature
async fn sse_feed(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>>
```

#### Replay mechanism

```rust
use axum::response::sse::{Event, Sse};
use datastar::prelude::*;  // PatchElements, ExecuteScript, etc.
use futures::stream::{self, Stream, StreamExt};
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

async fn sse_feed(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Extract Last-Event-ID from headers (SSE standard)
    let last_event_id = headers
        .get("Last-Event-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    // Subscribe to broadcast channel BEFORE replaying
    // This ensures no events are dropped between replay and live streaming
    let rx: broadcast::Receiver<StoredEvent> = app_state.event_bus.subscribe();

    // Replay missed events from SQLite
    let replayed_events = if let Some(since_seq) = last_event_id {
        app_state
            .event_store
            .query_since_sequence(since_seq + 1)
            .await
            .unwrap_or_default()
    } else {
        // Initial connection: send current projection state
        vec![app_state.projection.current_state_as_event().await]
    };

    // Convert replayed events to SSE format using datastar-rust builders
    // Note: SSE wire format uses lowercase strings ("outer", "inner", "append"),
    // while the Rust API uses PascalCase enum variants (ElementPatchMode::Outer).
    // The datastar-rust SDK handles conversion automatically.
    let replay_stream = stream::iter(replayed_events.into_iter().map(|evt| {
        Ok::<_, Infallible>(
            PatchElements::new(render_html(&evt))
                .id(evt.sequence.to_string())
                .into()  // Converts to axum::response::sse::Event
        )
    }));

    // Convert broadcast receiver to stream for future events
    let live_stream = BroadcastStream::new(rx)
        .filter_map(|result| async move {
            result.ok().map(|evt| {
                Ok::<_, Infallible>(
                    PatchElements::new(render_html(&evt))
                        .id(evt.sequence.to_string())
                        .into()
                )
            })
        });

    // Chain replay then live
    let combined = replay_stream.chain(live_stream);

    Sse::new(combined).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive-text"),
    )
}
```

#### Batch size considerations

**Initial load strategy**: Send complete projection state as single event (fat morph).
**Incremental updates**: One SSE event per domain event (fine-grained).

Trade-offs:
- **Fat morph** (send entire DOM subtree): Resilient to missed events, works with interrupted connections, aligns with Datastar philosophy ("In Morph We Trust").
- **Fine-grained** (append/remove): Smaller payload per event, but brittle if events are missed.

**Recommendation for ironstar**: Default to fat morph for initial state, fine-grained for incremental updates, but always design handlers to tolerate replay of the entire sequence.

### 1a. Reconnection resilience

**Decision: Subscribe before replay to prevent event loss during reconnection, with explicit handling of replay edge cases.**

#### The race condition

A naive SSE reconnection pattern queries historical events *then* subscribes to the broadcast channel.
This creates a race condition: events published between the query completion and subscription are lost.

```rust
// WRONG: Race condition between query and subscribe
async fn sse_feed_with_race_condition(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let last_event_id = extract_last_event_id(&headers);

    // Query completes at time T1
    let replayed = app_state.event_store
        .query_since_sequence(last_event_id.unwrap_or(0))
        .await
        .unwrap_or_default();

    // Events published between T1 and T2 are lost

    // Subscribe at time T2
    let rx = app_state.event_bus.subscribe();

    // Missing events are never delivered
    // ...
}
```

**Correct pattern**: Subscribe *before* querying to ensure all events published during replay are buffered in the broadcast channel receiver.

```rust
// CORRECT: Subscribe before replay
async fn sse_feed_resilient(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let last_event_id = extract_last_event_id(&headers);

    // 1. Subscribe FIRST - events published during replay are buffered
    let rx = app_state.event_bus.subscribe();

    // 2. Query historical events - any events published now are in rx buffer
    let replayed = if let Some(since_seq) = last_event_id {
        app_state.event_store
            .query_since_sequence(since_seq + 1)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // 3. Stream replayed events then live events
    // No gap: events published during query are in rx
    // ...
}
```

This pattern ensures gap-free delivery: events emitted while fetching historical data are buffered in the receiver and delivered after the replay stream completes.

#### Zenoh subscription with replay

When using Zenoh instead of `tokio::sync::broadcast`, the same subscribe-before-replay pattern applies.
Zenoh provides query/reply for historical data and pub/sub for live updates.

```rust
use zenoh::prelude::*;

async fn sse_feed_zenoh(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let last_event_id = extract_last_event_id(&headers);
    let session = app_state.zenoh_session.clone();

    // 1. Subscribe to future events FIRST
    let subscriber = session
        .declare_subscriber("events/**")
        .await
        .expect("failed to create subscriber");

    // 2. Query historical events via Zenoh storage
    let replayed = if let Some(since_seq) = last_event_id {
        let replies = session
            .get("events/**")
            .query()
            .await
            .expect("query failed");

        replies
            .into_iter()
            .filter_map(|reply| {
                let sample = reply.ok()?.into_result().ok()?;
                let event: StoredEvent = serde_json::from_slice(&sample.payload.to_bytes()).ok()?;
                if event.sequence > since_seq {
                    Some(event)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // 3. Convert to SSE stream (replay then live)
    let replay_stream = stream::iter(replayed.into_iter().map(|evt| {
        Ok::<_, Infallible>(event_to_sse(evt))
    }));

    let live_stream = subscriber
        .into_stream()
        .filter_map(|sample| async move {
            let event: StoredEvent = serde_json::from_slice(&sample.payload.to_bytes()).ok()?;
            Some(Ok::<_, Infallible>(event_to_sse(event)))
        });

    Sse::new(replay_stream.chain(live_stream))
}
```

Zenoh's query/reply provides a distributed alternative to SQLite for historical event retrieval.
This enables SSE handlers to run on different nodes than the event store, with Zenoh storage acting as a replicated event log.

#### Edge cases

**Last-Event-ID too old**: The client reconnected after the oldest available event was purged.
Fall back to sending complete current state rather than attempting partial replay.

```rust
async fn handle_stale_last_event_id(
    store: &impl EventStore,
    projection: &impl Projection,
    last_event_id: i64,
) -> Vec<StoredEvent> {
    let earliest = store.earliest_sequence().await.unwrap_or(0);

    if last_event_id < earliest {
        // Client is too far behind - send full state snapshot
        eprintln!(
            "Client Last-Event-ID {} is before earliest sequence {}; sending full state",
            last_event_id, earliest
        );
        vec![projection.current_state_as_event().await]
    } else {
        // Normal replay path
        store.query_since_sequence(last_event_id + 1)
            .await
            .unwrap_or_default()
    }
}
```

**Sequence gaps**: The event store has missing sequences due to deletion, compaction, or distributed synchronization lag.
Detect gaps and fall back to fat morph rather than risking inconsistent state.

```rust
fn detect_sequence_gaps(events: &[StoredEvent]) -> bool {
    events.windows(2).any(|w| w[1].sequence != w[0].sequence + 1)
}

async fn query_with_gap_detection(
    store: &impl EventStore,
    projection: &impl Projection,
    since_seq: i64,
) -> Vec<StoredEvent> {
    let events = store.query_since_sequence(since_seq + 1)
        .await
        .unwrap_or_default();

    if detect_sequence_gaps(&events) {
        eprintln!("Detected sequence gap; falling back to full state");
        vec![projection.current_state_as_event().await]
    } else {
        events
    }
}
```

**Error recovery**: SQLite query failures should degrade gracefully to full state rather than dropping the SSE connection.

```rust
async fn query_with_fallback(
    store: &impl EventStore,
    projection: &impl Projection,
    since_seq: i64,
) -> Vec<StoredEvent> {
    match store.query_since_sequence(since_seq + 1).await {
        Ok(events) => events,
        Err(e) => {
            eprintln!("Event store query failed: {}; sending full state", e);
            vec![projection.current_state_as_event().await]
        }
    }
}
```

#### EventStore trait extensions

Add methods to support reconnection edge case handling:

```rust
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

    /// Get the earliest available sequence number (for detecting stale Last-Event-ID)
    async fn earliest_sequence(&self) -> Result<i64, Error>;

    /// Get the latest sequence number (for monitoring lag)
    async fn latest_sequence(&self) -> Result<i64, Error>;
}
```

SQLite implementation:

```rust
#[async_trait]
impl EventStore for SqliteEventStore {
    // ... existing methods ...

    async fn earliest_sequence(&self) -> Result<i64, Error> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT MIN(sequence) FROM events"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.unwrap_or(0))
    }

    async fn latest_sequence(&self) -> Result<i64, Error> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT MAX(sequence) FROM events"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.unwrap_or(0))
    }
}
```

These methods enable:
- Detecting when `Last-Event-ID` is before the oldest available event (after purge/compaction)
- Monitoring projection lag (difference between latest sequence and last processed sequence)
- Validating event stream continuity

### 2. Projection caching strategy

**Decision: Pure in-memory projections with snapshot recovery on startup, no persistent projection state.**

#### Rationale

**Pure in-memory**: Recompute from events on startup.
- **Pros**: Simple, no cache invalidation, always consistent with event store.
- **Cons**: Slow startup if many events, holds memory.

**Persisted snapshots**: Store projection state periodically.
- **Pros**: Fast startup.
- **Cons**: Cache invalidation complexity, snapshot versioning, requires migration on projection schema changes.

**DuckDB materialized views**: Use DuckDB for analytical projections.
- **Pros**: Excellent for OLAP queries, automatic incremental updates.
- **Cons**: Overkill for simple UI projections, adds dependency complexity.

#### Implementation pattern

```rust
use async_trait::async_trait;
use axum::response::sse::Event;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Projection trait for read models
#[async_trait]
pub trait Projection: Send + Sync {
    type State: Clone + Send + Sync;

    /// Rebuild projection from event stream
    async fn rebuild(&self, events: Vec<StoredEvent>) -> Result<Self::State, Error>;

    /// Apply single event (for incremental updates)
    fn apply(&self, state: &mut Self::State, event: &StoredEvent) -> Result<(), Error>;

    /// Serialize current state to SSE event
    fn to_sse_event(&self, state: &Self::State, sequence: i64) -> Event;
}

/// In-memory projection manager
pub struct ProjectionManager<P: Projection> {
    projection: P,
    state: Arc<RwLock<P::State>>,
    event_bus_rx: broadcast::Receiver<StoredEvent>,
}
// Note: Error and StoredEvent types are defined in the appendix

impl<P: Projection> ProjectionManager<P> {
    /// Initialize projection by replaying all events
    pub async fn init(
        projection: P,
        event_store: &EventStore,
        event_bus: broadcast::Sender<StoredEvent>,
    ) -> Result<Self, Error> {
        let events = event_store.query_all().await?;
        let state = projection.rebuild(events).await?;

        let manager = Self {
            projection,
            state: Arc::new(RwLock::new(state)),
            event_bus_rx: event_bus.subscribe(),
        };

        // Spawn background task to update projection from event bus
        let state_clone = manager.state.clone();
        let projection_clone = manager.projection.clone();
        let mut rx = manager.event_bus_rx.resubscribe();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let mut state = state_clone.write().await;
                if let Err(e) = projection_clone.apply(&mut *state, &event) {
                    eprintln!("Projection update failed: {:?}", e);
                }
            }
        });

        Ok(manager)
    }

    /// Query current state (for non-SSE endpoints)
    pub async fn query(&self) -> P::State {
        self.state.read().await.clone()
    }

    /// Get current state as SSE event (for initial SSE connection)
    pub async fn current_state_as_event(&self, sequence: i64) -> Event {
        let state = self.state.read().await;
        self.projection.to_sse_event(&*state, sequence)
    }
}
```

#### When to use DuckDB

Use DuckDB materialized views for:
- **Analytics dashboards**: Aggregate queries over large event histories (e.g., "total sales per month").
- **Time-series analysis**: Window functions, moving averages.
- **Report generation**: Complex joins across multiple projections.

**Not** for:
- UI state (use in-memory projection).
- Session-specific data (use SQLite sessions table).
- Transactional commands (use event store).

### DuckDB and async runtime integration

DuckDB-rs is a synchronous, blocking library.
All query methods block the calling thread until results are available.
In async axum handlers running on tokio, blocking calls must be carefully wrapped to avoid blocking the async runtime's worker threads, which would degrade performance for all concurrent requests.

#### Integration strategies

**For quick queries** (expected to complete in milliseconds): Use `tokio::task::block_in_place()`.
This allows blocking operations within an async context without spawning a new OS thread.
The tokio runtime temporarily removes the worker thread from its pool while the blocking operation runs.

**For long-running analytics** (seconds or more): Use `tokio::task::spawn_blocking()`.
This spawns the blocking work on a dedicated thread pool, preventing it from tying up async worker threads.

#### Code examples

```rust
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;
use tokio::task;

// Quick query pattern - block_in_place
async fn analytics_handler(
    State(analytics): State<Arc<AnalyticsService>>,
) -> Result<impl IntoResponse, AppError> {
    // Note: AppError is defined in the appendix
    let analytics = analytics.clone();

    // block_in_place: allows blocking without spawning new thread
    // Use for queries expected to complete quickly (< 100ms)
    let result = task::block_in_place(|| {
        analytics.query_aggregate_counts()
    })?;

    Ok(Json(result))
}

// Long-running query pattern - spawn_blocking
async fn heavy_report_handler(
    State(analytics): State<Arc<AnalyticsService>>,
) -> Result<impl IntoResponse, AppError> {
    let analytics = analytics.clone();

    // spawn_blocking: runs on dedicated blocking thread pool
    // Use for long-running queries (seconds or more)
    let result = task::spawn_blocking(move || {
        analytics.generate_monthly_report()
    })
    .await??;  // First ? for JoinError, second ? for business logic error

    Ok(Json(result))
}
```

#### Connection management

DuckDB's `Connection` type is `Send` but not `Sync`.
`Statement` is neither `Send` nor `Sync`.
This means:

- A `Connection` can be moved between threads but not shared.
- `Statement` must stay on the thread where it was created.

**Connection pooling pattern**:

```rust
use duckdb::Connection;
use std::sync::{Arc, Mutex};

// Simple approach: Mutex around single connection
pub struct DuckDBService {
    conn: Arc<Mutex<Connection>>,
}

impl DuckDBService {
    pub fn query_aggregate_counts(&self) -> Result<Vec<AggregateCount>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT aggregate_type, COUNT(*) FROM events GROUP BY aggregate_type")?;
        let rows = stmt.query_map([], |row| {
            Ok(AggregateCount {
                aggregate_type: row.get(0)?,
                count: row.get(1)?,
            })
        })?;
        rows.collect()
    }
}

// Alternative: One connection per blocking task (no contention)
pub struct DuckDBService {
    database_path: String,
}

impl DuckDBService {
    pub fn query_aggregate_counts(&self) -> Result<Vec<AggregateCount>, Error> {
        // Each query gets its own connection
        let conn = Connection::open(&self.database_path)?;
        let mut stmt = conn.prepare("SELECT aggregate_type, COUNT(*) FROM events GROUP BY aggregate_type")?;
        let rows = stmt.query_map([], |row| {
            Ok(AggregateCount {
                aggregate_type: row.get(0)?,
                count: row.get(1)?,
            })
        })?;
        rows.collect()
    }
}
```

For ironstar analytics projections, the one-connection-per-task pattern is simpler and avoids lock contention.
DuckDB handles concurrent access at the file level, so multiple connections to the same database file work correctly.

### 3. Broadcast channel patterns

**Decision: `tokio::sync::broadcast` with lagged receiver handling and fan-out semantics.**

#### Implementation

```rust
use std::sync::Arc;
use tokio::sync::broadcast;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    event_store: Arc<EventStore>,
    event_bus: broadcast::Sender<StoredEvent>,
    projections: Arc<Projections>,
}

impl AppState {
    pub fn new(event_store: EventStore, bus_capacity: usize) -> Self {
        let (event_bus, _) = broadcast::channel(bus_capacity);

        Self {
            event_store: Arc::new(event_store),
            event_bus,
            projections: Arc::new(Projections::default()),
        }
    }
}

/// Stored event with sequence number
#[derive(Clone, Debug)]
pub struct StoredEvent {
    pub sequence: i64,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

/// Handling slow consumers (lagged receivers)
async fn sse_stream_with_lag_handling(
    rx: broadcast::Receiver<StoredEvent>,
    event_store: Arc<EventStore>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    // Note: Imports from replay_stream example also apply here
    use axum::response::sse::Event;
    use datastar::prelude::*;  // ExecuteScript
    use futures::stream::{Stream, StreamExt};
    use std::convert::Infallible;
    use tokio_stream::wrappers::BroadcastStream;
    BroadcastStream::new(rx).filter_map(move |result| {
        let event_store = event_store.clone();
        async move {
            match result {
                Ok(event) => Some(Ok(convert_to_sse(event))),
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    // Slow consumer: replay skipped events from event store
                    // This prevents data loss but adds latency
                    eprintln!("SSE consumer lagged, skipped {} events", skipped);

                    // In practice, you'd fetch the skipped events here
                    // For now, send a signal to reconnect using datastar-rust builder
                    Some(Ok(ExecuteScript::new("window.location.reload()").into()))
                }
                Err(broadcast::error::RecvError::Closed) => None,
            }
        }
    })
}
```

#### Channel sizing

**Bus capacity** determines how many events can be buffered before slow consumers are marked as lagged.

```rust
// Conservative: Small buffer, fail fast on slow consumers
broadcast::channel::<StoredEvent>(16)

// Permissive: Large buffer, tolerate slow consumers (uses more memory)
broadcast::channel::<StoredEvent>(1024)

// Ironstar default: 256 events (~1MB assuming 4KB events)
broadcast::channel::<StoredEvent>(256)
```

#### Multiple projection types

```rust
use tokio::sync::broadcast;

/// Projections manager supporting multiple projection types
pub struct Projections {
    todo_list: ProjectionManager<TodoListProjection>,
    user_profile: ProjectionManager<UserProfileProjection>,
    analytics: ProjectionManager<AnalyticsProjection>,
}

impl Projections {
    pub async fn init(
        event_store: &EventStore,
        event_bus: broadcast::Sender<StoredEvent>,
    ) -> Result<Self, Error> {
        Ok(Self {
            todo_list: ProjectionManager::init(
                TodoListProjection,
                event_store,
                event_bus.clone(),
            ).await?,
            user_profile: ProjectionManager::init(
                UserProfileProjection,
                event_store,
                event_bus.clone(),
            ).await?,
            analytics: ProjectionManager::init(
                AnalyticsProjection,
                event_store,
                event_bus.clone(),
            ).await?,
        })
    }
}
```

### 4. Write path (command handling)

**Decision: Command → Event → Append → Broadcast, with immediate 202 Accepted response.**

#### Pattern

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
    // Note: ValidationError type is defined in the appendix
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

#### Loading indicator integration

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
1. User sees immediate feedback (loading indicator).
2. POST returns quickly (no blocking).
3. SSE delivers the update and removes the loading indicator.
4. No optimistic updates (backend is source of truth).

### 5. Consistency boundaries

#### Guarantees

**Per-aggregate consistency**: Events for a single aggregate are totally ordered by sequence number.

**Cross-aggregate consistency**: No guarantees. If command affects multiple aggregates, events are appended sequentially but readers may observe intermediate states.

**SSE vs POST ordering**: The SSE update may arrive before or after the POST response due to network timing and browser concurrency.

#### Handling SSE arriving before POST

This is the common case in CQRS: the SSE connection receives the event before the POST handler finishes.

**Frontend pattern**:

```html
<button data-on:click="
    el.classList.add('loading');
    @post('/command').then(() => {
        // POST completed, but SSE may have already updated DOM
        // Remove loading only if SSE hasn't already done it
        setTimeout(() => el.classList.remove('loading'), 100);
    })
">
    Execute
</button>
```

**Better pattern (rely on SSE exclusively)**:

```html
<button data-on:click="
    el.classList.add('loading');
    @post('/command')
">
    Execute
    <span data-show="el.classList.contains('loading')">Saving...</span>
</button>
```

The SSE update morphs the DOM and includes a script to remove loading class.
This way, timing doesn't matter: SSE is the single source of truth for when the operation completes.

## Code patterns

### Aggregate trait

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

### Aggregate testing patterns

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

### Event store trait

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
// Note: Error, DomainEvent, and StoredEvent types are defined in the appendix

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

### Projection trait

See section 2 above for full implementation.

### Handler patterns

See sections 1 and 4 above for SSE and command handler implementations.

## Ironstar defaults and recommendations

### Default architecture

The module structure below is a simplified view focused on event sourcing components.
See `architecture-decisions.md` for the complete structure including the explicit `application/` layer for command and query handlers.

```
src/
├── domain/
│   ├── events.rs          # DomainEvent enum (sum type)
│   ├── commands.rs        # Command types (product types)
│   └── aggregates/        # Aggregate root logic (validation)
├── infrastructure/
│   ├── event_store.rs     # SqliteEventStore impl
│   ├── event_bus.rs       # Broadcast channel setup
│   └── projections/       # Projection implementations
│       ├── mod.rs
│       ├── todo_list.rs
│       └── analytics.rs
└── presentation/
    ├── handlers/
    │   ├── sse.rs         # SSE feed handler
    │   └── commands.rs    # POST command handlers
    └── templates/         # hypertext components
```

### Configuration defaults

```toml
# config.toml
[event_sourcing]
# Broadcast channel capacity (number of events buffered)
broadcast_capacity = 256

# SQLite WAL mode (default: WAL)
sqlite_journal_mode = "WAL"

# SQLite synchronous mode (default: FULL for durability)
sqlite_synchronous = "FULL"

[sse]
# Keep-alive interval (prevent proxy timeouts)
keep_alive_seconds = 15

# Enable compression (Brotli via tower-http)
enable_compression = true

[projections]
# Projection rebuild on startup (default: true for simplicity)
rebuild_on_startup = true

# Future: snapshot interval (not implemented yet)
# snapshot_every_n_events = 1000
```

### Production considerations

#### SQLite tuning

```rust
use sqlx::SqlitePool;

// Optimize SQLite for event sourcing workload
sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
sqlx::query("PRAGMA synchronous=FULL").execute(&pool).await?;
sqlx::query("PRAGMA cache_size=-64000").execute(&pool).await?; // 64MB cache
sqlx::query("PRAGMA temp_store=MEMORY").execute(&pool).await?;
```

#### Compression

Enable Brotli compression for SSE responses:

```rust
use axum::{Router, routing::get};
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .route("/feed", get(sse_feed))
    .layer(CompressionLayer::new());
```

Datastar documentation claims 200:1 compression ratios for HTML over SSE with Brotli.

#### Monitoring

```rust
use prometheus::{IntCounter, IntGauge, Registry};

pub struct Metrics {
    events_appended: IntCounter,
    sse_connections: IntGauge,
    projection_lag: IntGauge,
}
// Note: Prometheus metrics require the prometheus crate in Cargo.toml
```

Track:
- Events appended per second
- Active SSE connections
- Projection lag (last processed sequence vs last appended sequence)
- Broadcast channel lag events

## Trade-off analysis

### Event replay: sequence numbers vs timestamps

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Sequence numbers** | Monotonic, no clock skew, efficient indexing | Couples event identity to storage | **Use for ironstar** |
| **Timestamps** | Natural ordering, works across distributed systems | Clock skew, not unique, slower queries | Use with distributed event store (Zenoh future) |

### Projection caching: in-memory vs persisted snapshots

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **In-memory (rebuild)** | Simple, no cache invalidation, always consistent | Slow startup with many events | **Use for ironstar v1** |
| **Persisted snapshots** | Fast startup | Cache invalidation, snapshot versioning, migrations | Add later if startup becomes slow |
| **DuckDB views** | Optimized for analytics, incremental updates | Overkill for UI state, extra dependency | Use only for analytics projections |

### SSE replay: fat morph vs incremental

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Fat morph** | Resilient to missed events, simple | Larger payload per event | **Default for ironstar** |
| **Incremental (append/remove)** | Smaller payload | Brittle if events missed, complex | Use only when payload size is proven bottleneck |

### Broadcast channel: small vs large capacity

| Capacity | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Small (16)** | Low memory, fail fast on slow consumers | Lagged receivers trigger reconnects | Use for real-time apps with strict latency requirements |
| **Large (1024)** | Tolerates slow consumers | Higher memory usage, delayed error detection | Use for batch/analytics workloads |
| **Medium (256)** | Balanced | - | **Use for ironstar (default)** |

## Future enhancements

### Distributed event sourcing with Zenoh

When moving to distributed deployment, replace `tokio::sync::broadcast` with Zenoh pub/sub:

```rust
use zenoh::prelude::*;

// Future Zenoh integration
let session = zenoh::open(config).await?;
let publisher = session.declare_publisher("events/**").await?;

// Publish event
publisher.put(serde_json::to_vec(&event)?).await?;

// Subscribe
let subscriber = session.declare_subscriber("events/**").await?;
while let Ok(sample) = subscriber.recv_async().await {
    let event: StoredEvent = serde_json::from_slice(&sample.payload)?;
    // Update projection
}
```

Zenoh provides:
- Distributed pub/sub across processes and machines
- Storage backends (persist events in Zenoh storage, not just SQLite)
- Query/reply pattern (fetch historical events from any node)
- Zero-copy shared memory when local

### Event schema evolution with upcasters

As the domain evolves, event schemas change.
Upcasters transform old event formats to current schemas during event loading, avoiding costly data migrations.

```rust
use serde_json::Value;

/// Transforms events from old schema versions to current schema
pub trait EventUpcaster: Send + Sync {
    /// Check if this upcaster handles the given event type and version
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool;

    /// Transform the event payload to the current schema
    fn upcast(&self, payload: Value) -> Value;
}

/// Registry of upcasters applied during event loading
pub struct UpcasterChain {
    upcasters: Vec<Box<dyn EventUpcaster>>,
}

impl UpcasterChain {
    pub fn new() -> Self {
        Self { upcasters: Vec::new() }
    }

    pub fn register(mut self, upcaster: Box<dyn EventUpcaster>) -> Self {
        self.upcasters.push(upcaster);
        self
    }

    /// Apply all matching upcasters to transform event to current schema
    pub fn upcast(&self, event_type: &str, event_version: &str, mut payload: Value) -> Value {
        for upcaster in &self.upcasters {
            if upcaster.can_upcast(event_type, event_version) {
                payload = upcaster.upcast(payload);
            }
        }
        payload
    }
}

/// Load events with automatic schema upcasting
pub fn load_events_with_upcasting<A: Aggregate>(
    raw_events: Vec<StoredEvent>,
    upcaster_chain: &UpcasterChain,
) -> Vec<A::Event> {
    raw_events
        .into_iter()
        .filter_map(|stored| {
            let event_version = stored.metadata
                .as_ref()
                .and_then(|m| m.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("1");

            let payload = upcaster_chain.upcast(
                &stored.event_type,
                event_version,
                stored.payload,
            );

            serde_json::from_value(payload).ok()
        })
        .collect()
}

// Example upcaster: TodoCreated v1 -> v2 (added priority field)
struct TodoCreatedV1ToV2;

impl EventUpcaster for TodoCreatedV1ToV2 {
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool {
        event_type == "TodoCreated" && event_version == "1"
    }

    fn upcast(&self, mut payload: Value) -> Value {
        // Add default priority if missing
        if let Value::Object(ref mut map) = payload {
            if !map.contains_key("priority") {
                map.insert("priority".to_string(), Value::String("normal".to_string()));
            }
        }
        payload
    }
}
```

Upcasters are applied lazily during event loading, not as batch migrations.
This keeps the event store immutable (events are facts that cannot change) while allowing the domain model to evolve.

### Snapshot optimization

Add periodic snapshots when event count grows:

```rust
use sqlx::SqlitePool;

pub struct SnapshotStore {
    pool: sqlx::SqlitePool,
}

impl SnapshotStore {
    async fn save_snapshot(&self, projection_name: &str, sequence: i64, state: &[u8]) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO snapshots (projection_name, sequence, state, created_at)
            VALUES (?, ?, ?, datetime('now'))
            "#,
        )
        .bind(projection_name)
        .bind(sequence)
        .bind(state)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_snapshot(&self, projection_name: &str) -> Result<Option<(i64, Vec<u8>)>, Error> {
        let row = sqlx::query_as::<_, (i64, Vec<u8>)>(
            r#"
            SELECT sequence, state FROM snapshots
            WHERE projection_name = ?
            ORDER BY sequence DESC
            LIMIT 1
            "#,
        )
        .bind(projection_name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }
}
```

Trigger snapshot every N events or on a schedule.

## Appendix: Common type definitions

The code examples throughout this document reference types like `AppError`, `StoredEvent`, `DomainEvent`, and `ValidationError`.
These are example types that would typically live in `src/domain/` and `src/infrastructure/`.
Adapt them to your specific domain requirements.

```rust
//! Common types used in event sourcing examples
//! These would typically live in src/domain/ and src/infrastructure/

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application error type for handler responses
#[derive(Error, Debug)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

/// Validation errors for command handling
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("field '{field}' is required")]
    Required { field: &'static str },

    #[error("field '{field}' must be at most {max} characters")]
    TooLong { field: &'static str, max: usize },

    #[error("invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
}

/// Event stored in SQLite event store
/// Note: Derives Clone for use in tokio::sync::broadcast channels
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredEvent {
    pub sequence: i64,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Domain events (example for a Todo aggregate)
/// Sum type representing all possible events in the domain
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    TodoCreated { id: String, text: String },
    TodoCompleted { id: String },
    TodoDeleted { id: String },
    TodoTextUpdated { id: String, text: String },
}

impl DomainEvent {
    /// Extract event type name for storage in event_type column
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::TodoCreated { .. } => "TodoCreated",
            Self::TodoCompleted { .. } => "TodoCompleted",
            Self::TodoDeleted { .. } => "TodoDeleted",
            Self::TodoTextUpdated { .. } => "TodoTextUpdated",
        }
    }

    /// Extract aggregate ID for storage in aggregate_id column
    pub fn aggregate_id(&self) -> &str {
        match self {
            Self::TodoCreated { id, .. }
            | Self::TodoCompleted { id }
            | Self::TodoDeleted { id }
            | Self::TodoTextUpdated { id, .. } => id,
        }
    }
}

/// Commands (requests to change state)
/// Product types containing validated user input
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    CreateTodo { text: String },
    CompleteTodo { id: String },
    DeleteTodo { id: String },
    UpdateTodoText { id: String, text: String },
}
```

**Dependencies referenced in these types:**

- `serde` (v1.0): Serialization/deserialization
- `thiserror` (v1.0): Error derive macros
- `chrono` (v0.4): Date/time types
- `sqlx` (v0.9): Database errors

**Implementation notes:**

- `StoredEvent` derives `Clone` because it's sent through `tokio::sync::broadcast` channels, which require cloneable types.
- `DomainEvent` uses `#[serde(tag = "type")]` for tagged union JSON serialization, making event payloads human-readable.
- `ValidationError` uses `thiserror::Error` to automatically implement `std::error::Error` with proper Display formatting.
- `AppError` uses `#[from]` attribute to enable automatic conversion from `ValidationError` and `sqlx::Error` via the `?` operator.

## References

### Datastar and SSE

- Datastar SDK ADR: `/Users/crs58/projects/lakescope-workspace/datastar/sdk/ADR.md`
- Tao of Datastar: `/Users/crs58/projects/lakescope-workspace/datastar-doc/guide_the_tao_of_datastar.md`
- Northstar Go template: `/Users/crs58/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
- Lince Rust example: `/Users/crs58/projects/rust-workspace/datastar-rust-lince/`
- SSE spec: https://html.spec.whatwg.org/multipage/server-sent-events.html

### CQRS and event sourcing frameworks

- cqrs-es TestFramework: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/framework.rs`
- cqrs-es TestExecutor: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/executor.rs`
- cqrs-es TestValidator: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/validator.rs`
- esrs pure Aggregate trait: `/Users/crs58/projects/rust-workspace/event_sourcing.rs/src/aggregate.rs`
- sqlite-es event repository: `/Users/crs58/projects/rust-workspace/sqlite-es/src/event_repository.rs`
- CQRS pattern: https://martinfowler.com/bliki/CQRS.html

### Analytics caching

- Analytics cache architecture: `analytics-cache-architecture.md`
