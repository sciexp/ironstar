# Event replay and consistency patterns

This document covers event replay strategies, reconnection resilience patterns, and consistency boundaries in ironstar's CQRS architecture.
Understanding these patterns ensures reliable event delivery and predictable consistency semantics.

## Event replay strategy

**Decision: Use event sequence numbers as SSE `id` field, enable automatic replay via `Last-Event-ID` header.**

### Pattern overview

Each event stored in SQLite gets a monotonically increasing sequence number.
When emitting SSE events, set the `id` field to the sequence number.
The browser automatically sends `Last-Event-ID` header on reconnection.
The SSE handler replays all events since that ID before streaming new ones.

### Event store schema

```rust
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
```

The `sequence` column provides:
- Monotonically increasing global order
- Efficient replay via `WHERE sequence > ?`
- SSE `id` field for automatic browser replay
- Projection lag metrics

### Replay mechanism implementation

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

### Batch size considerations

**Initial load strategy**: Send complete projection state as single event (fat morph).
**Incremental updates**: One SSE event per domain event (fine-grained).

Trade-offs:
- **Fat morph** (send entire DOM subtree): Resilient to missed events, works with interrupted connections, aligns with Datastar philosophy ("In Morph We Trust").
- **Fine-grained** (append/remove): Smaller payload per event, but brittle if events are missed.

**Recommendation for ironstar**: Default to fat morph for initial state, fine-grained for incremental updates, but always design handlers to tolerate replay of the entire sequence.

## Reconnection resilience

**Decision: Subscribe before replay to prevent event loss during reconnection, with explicit handling of replay edge cases.**

### The race condition

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

### Zenoh subscription with replay

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

### Edge case handling

#### Last-Event-ID too old

The client reconnected after the oldest available event was purged.
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

#### Sequence gaps

The event store has missing sequences due to deletion, compaction, or distributed synchronization lag.
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

#### Error recovery

SQLite query failures should degrade gracefully to full state rather than dropping the SSE connection.

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

### EventStore trait extensions

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

## Consistency boundaries

### Guarantees

**Per-aggregate consistency**: Events for a single aggregate are totally ordered by sequence number.

**Cross-aggregate consistency**: No guarantees.
If command affects multiple aggregates, events are appended sequentially but readers may observe intermediate states.

**SSE vs POST ordering**: The SSE update may arrive before or after the POST response due to network timing and browser concurrency.

### Handling SSE arriving before POST

This is the common case in CQRS: the SSE connection receives the event before the POST handler finishes.

**Frontend pattern (problematic)**:

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

Backend rendering includes cleanup script:

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

## Related documentation

- `event-sourcing-core.md`: Master index and architecture overview
- `sse-connection-lifecycle.md`: SSE connection phases and debugging
- `projection-patterns.md`: Projection caching strategies
- `performance-tuning.md`: Performance optimization for high-throughput scenarios
- `command-write-patterns.md`: Command handlers and write path
- `zenoh-event-bus.md`: Zenoh integration for distributed event bus
