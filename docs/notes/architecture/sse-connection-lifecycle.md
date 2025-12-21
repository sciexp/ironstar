# SSE connection lifecycle

This document covers the complete SSE connection lifecycle in ironstar, from connection establishment through active streaming to disconnection and cleanup.
Understanding these patterns is critical for implementing reliable event streaming and debugging connection issues.

## Connection state machine

```
┌──────────┐
│ Browser  │
└────┬─────┘
     │
     │ navigator.onLine && @get('/feed')
     ▼
┌────────────────┐
│  Connecting    │  ──────────────┐
│  - TCP handshake               │ Connection failure
│  - HTTP GET /feed              │ (network error, 5xx)
│  - Send Last-Event-ID header   │
└────┬───────────┘               │
     │                           │
     │ 200 OK + text/event-stream │
     ▼                           │
┌────────────────┐               │
│  Subscribed    │               │
│  - Receiving events           │
│  - Keep-alive heartbeats      │
│  - Update Last-Event-ID       │
└────┬───────────┘               │
     │                           │
     │ TCP close / timeout       │
     ▼                           │
┌────────────────┐               │
│ Disconnected   │ ◄─────────────┘
│  - Auto-retry with             │
│    exponential backoff         │
│  - Send Last-Event-ID          │
└────────────────┘               │
     │                           │
     └───────────────────────────┘
     Retry after delay
```

The browser automatically handles reconnection with exponential backoff (typically 3s → 6s → 12s, up to implementation-defined maximum).
No client-side JavaScript required for basic reconnection behavior.

## Phase 1: Connection establishment

The SSE handler receives the HTTP request and extracts connection metadata.

```rust
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::sse::{Event, Sse},
};
use futures::stream::Stream;
use std::{convert::Infallible, net::SocketAddr};

async fn sse_feed(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Extract Last-Event-ID for replay (SSE standard header)
    let last_event_id = headers
        .get("Last-Event-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    // Extract User-Agent for debugging
    let user_agent = headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!(
        client_addr = %addr,
        user_agent = %user_agent,
        last_event_id = ?last_event_id,
        "SSE connection established"
    );

    // Continue to Phase 2...
}
```

**ConnectInfo extractor**:
- Requires `Router::into_make_service_with_connect_info::<SocketAddr>()` in main.rs
- Provides client IP for logging and rate limiting
- Essential for debugging connection issues

## Phase 2: Subscription establishment

Critical ordering: subscribe to broadcast channel BEFORE replaying events to prevent dropped events during the gap.

```rust
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

// Inside sse_feed handler...

// CRITICAL: Subscribe BEFORE replay to prevent event loss
let rx: broadcast::Receiver<StoredEvent> = app_state.event_bus.subscribe();

// Replay events missed since last connection
let replayed_events = if let Some(since_seq) = last_event_id {
    app_state
        .event_store
        .query_since_sequence(since_seq + 1)
        .await
        .unwrap_or_default()
} else {
    // Initial connection: send full projection state
    vec![app_state.projection.current_state_as_event().await]
};

// Convert to SSE stream
let replay_stream = stream::iter(replayed_events.into_iter().map(|evt| {
    Ok::<_, Infallible>(
        PatchElements::new(render_html(&evt))
            .id(evt.sequence.to_string())
            .into()
    )
}));
```

**Why subscribe before replay?**

If you replay first, then subscribe, events emitted during replay are lost:

```
Timeline (WRONG - events lost):
t0: Start replay query (query fetches events 1-100)
t1: Event 101 published → LOST (not subscribed yet)
t2: Replay completes, subscribe to broadcast
t3: Event 102 published → received

Timeline (CORRECT - no event loss):
t0: Subscribe to broadcast channel
t1: Start replay query (query fetches events 1-100)
t2: Event 101 published → received via broadcast
t3: Replay completes
t4: Event 102 published → received via broadcast
```

## Phase 3: Active streaming

Stream replayed events followed by live events from the broadcast channel.

```rust
use datastar::prelude::*;
use futures::stream::{self, StreamExt};
use std::time::Duration;

// Chain replay then live events
let live_stream = BroadcastStream::new(rx)
    .filter_map(|result| async move {
        match result {
            Ok(evt) => Some(Ok(PatchElements::new(render_html(&evt))
                .id(evt.sequence.to_string())
                .into())),
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                // Slow consumer: force reconnect to replay missed events
                tracing::warn!(
                    skipped_events = skipped,
                    "SSE consumer lagged, forcing reconnect"
                );
                Some(Ok(ExecuteScript::new("window.location.reload()").into()))
            }
            Err(broadcast::error::RecvError::Closed) => None,
        }
    });

let combined = replay_stream.chain(live_stream);

// Return SSE response with keep-alive
Sse::new(combined).keep_alive(
    axum::response::sse::KeepAlive::new()
        .interval(Duration::from_secs(15))
        .text("keep-alive-text"),
)
```

**Keep-alive heartbeats**:
- Prevent proxy timeouts (most proxies timeout idle connections after 30-60s)
- Allow early detection of broken connections
- Browser updates internal timeout on each heartbeat
- Default 15s interval is conservative (can go up to 30s for lower overhead)

## Phase 4: Disconnection and cleanup

SSE cleanup happens automatically via Rust's RAII when the stream is dropped.

### Graceful disconnection (client navigates away)

```rust
// When the browser closes the connection:
// 1. axum detects TCP FIN or RST
// 2. Stream is dropped
// 3. BroadcastStream::drop() unsubscribes from broadcast channel
// 4. No explicit cleanup code needed (RAII handles it)
```

### Ungraceful disconnection (network failure)

```rust
// Network partition or client crashes:
// 1. Keep-alive heartbeats fail to send
// 2. axum's write timeout triggers (if configured)
// 3. Stream is dropped
// 4. Cleanup via RAII

// Configure write timeout via tower-http:
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

let app = Router::new()
    .route("/feed", get(sse_feed))
    .layer(TimeoutLayer::new(Duration::from_secs(60)));
```

Without write timeout, dead connections remain open until TCP keepalive triggers (OS default: 2 hours on Linux).
Recommended: Set write timeout to 2-3x keep-alive interval (e.g., 45s timeout with 15s keep-alive).

## Resource guarantees

| Resource | Cleanup Trigger | Guaranteed By |
|----------|----------------|---------------|
| Broadcast receiver | Stream drop | `BroadcastStream::drop()` |
| SSE stream task | TCP close or timeout | Tokio task cancellation |
| Memory (buffered events) | Stream drop | Rust RAII |
| TCP socket | axum response complete | OS kernel |
| Database connection | Not held (query completes before streaming) | N/A |

**Memory leak risk**: If you spawn a background task that holds a broadcast receiver, dropping the stream won't cancel it.
Always tie receiver lifetime to stream lifetime (use `select!` or structured concurrency).

## Monitoring active connections

Track active SSE connections for capacity planning and debugging.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ConnectionTracker {
    active: Arc<AtomicUsize>,
}

impl ConnectionTracker {
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn track(&self) -> ConnectionGuard {
        self.active.fetch_add(1, Ordering::SeqCst);
        ConnectionGuard {
            active: self.active.clone(),
        }
    }

    pub fn count(&self) -> usize {
        self.active.load(Ordering::SeqCst)
    }
}

/// RAII guard that decrements counter on drop
pub struct ConnectionGuard {
    active: Arc<AtomicUsize>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

// Usage in SSE handler:
async fn sse_feed(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let _guard = app_state.connection_tracker.track();
    // Guard is held for lifetime of SSE stream
    // Automatic decrement on stream drop

    // ... rest of handler
}
```

Expose active connection count via metrics endpoint:

```rust
use axum::{response::Json, extract::State};
use serde::Serialize;

#[derive(Serialize)]
struct Metrics {
    active_sse_connections: usize,
}

async fn metrics_handler(
    State(app_state): State<AppState>,
) -> Json<Metrics> {
    Json(Metrics {
        active_sse_connections: app_state.connection_tracker.count(),
    })
}
```

## Reconnection best practices

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

## Edge cases and debugging

### Edge case: Last-Event-ID too old

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

### Edge case: Sequence gaps

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

### Edge case: Error recovery

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

### Debugging connection issues

| Symptom | Likely Cause | Debug Strategy |
|---------|--------------|----------------|
| Connection never established | CORS, network, firewall | Check browser DevTools Network tab, verify CORS headers, check server logs for connection attempts |
| Connects but no events | Wrong endpoint, handler panic, empty replay | Check `Last-Event-ID` header value, verify events exist in event store with `query_since_sequence()`, add tracing to handler |
| Events arrive then stop | Broadcast lag, handler panic, keep-alive failure | Check for `RecvError::Lagged` in logs, verify keep-alive interval < proxy timeout, check CPU usage (slow render?) |
| Frequent reconnects | Keep-alive too long, proxy timeout, server timeout | Reduce keep-alive interval to 15s, check proxy logs, verify tower-http timeout is 2-3x keep-alive |
| Memory leak (connections grow) | Background tasks holding receivers, missing drop | Audit code for `tokio::spawn` with broadcast receivers not tied to stream lifetime, use ConnectionTracker |
| Events duplicated on reconnect | Replay logic includes events after `Last-Event-ID` | Verify `query_since_sequence(since_seq + 1)` uses `+ 1`, not `since_seq` |

**Essential debugging headers**:

```rust
// Log all relevant headers on connection
tracing::debug!(
    last_event_id = ?headers.get("Last-Event-ID"),
    user_agent = ?headers.get("User-Agent"),
    accept = ?headers.get("Accept"),
    cache_control = ?headers.get("Cache-Control"),
    "SSE connection headers"
);
```

**Browser DevTools checks**:
1. Network tab → filter "EventStream" → verify Status 200 and Type "eventsource"
2. Right-click connection → Copy as cURL to test without browser
3. Inspect EventStream → verify `id:` field matches sequence numbers
4. Check Console for JavaScript errors (datastar initialization failures)

## Related documentation

- `event-sourcing-core.md`: Master index and architecture overview
- `event-replay-consistency.md`: Event replay strategy and consistency boundaries
- `projection-patterns.md`: Projection caching strategies
- `performance-tuning.md`: Performance optimization for high-throughput scenarios
- `command-write-patterns.md`: Command handlers and write path
- `zenoh-event-bus.md`: Zenoh integration for distributed event bus
