# Zenoh event bus architecture

Ironstar uses Zenoh as its event notification layer, providing server-side key expression filtering essential for CQRS + Datastar applications.
This mirrors the Go ecosystem pattern where NATS is the default choice for real-time event-driven applications.
Zenoh is the Rust-native equivalent: pure Rust implementation, no external server required, and key expression filtering that enables the "listen to hundreds of keys with server-side filtering" pattern.

## Architecture overview

| Layer | Component | Role |
|-------|-----------|------|
| Event notification | Zenoh | Key expression filtering, distribution-ready |
| Analytics cache | moka | TTL, eviction, synchronous fast path |
| Event store | SQLite | Append-only log (source of truth) |

**Key capabilities:**

1. Server-side key expression filtering (`events/Todo/**`, `events/User/*`)
2. Embedded mode with minimal configuration (4 lines)
3. Multiple subscribers with `select!` for multi-pattern watching
4. Distribution-ready architecture requiring no changes for multi-node deployment
5. Pure Rust implementation with no external server dependency

**Separation of concerns:**

Zenoh handles event notification and routing.
moka handles analytics caching with TTL and eviction policies.
Zenoh storage is designed for distributed data synchronization, not caching, so moka remains essential for cache-specific features like per-key TTL, time-to-idle eviction, and size-based eviction.

## Embedded configuration

Ironstar configures Zenoh for embedded operation without network activity.
While Zenoh has no built-in "embedded only" mode, disabling networking requires only four configuration settings:

```rust
use zenoh::Config;

let mut config = Config::default();
config.insert_json5("listen/endpoints", "[]").unwrap();
config.insert_json5("connect/endpoints", "[]").unwrap();
config.insert_json5("scouting/multicast/enabled", "false").unwrap();
config.insert_json5("scouting/gossip/enabled", "false").unwrap();

let session = zenoh::open(config).await.unwrap();
```

**Implementation notes:**

Even with empty endpoints, Zenoh instantiates its `TransportManager` and runtime machinery.
This overhead is acceptable for production infrastructure.
The configuration is identical whether running embedded or networked — the same `RuntimeBuilder::build()` path handles both cases.

## Key expression design

Zenoh provides server-side filtering via key expressions with wildcards:

| Wildcard | Matches | Example |
|----------|---------|---------|
| `*` | Single segment | `a/*/c` matches `a/b/c` |
| `**` | Zero or more segments | `a/**/c` matches `a/c`, `a/b/c`, `a/b/d/c` |
| `$*` | Within segment | `ab$*cd` matches `abxxcd` |

**Multi-pattern subscriptions:**

A single subscriber watches one key expression pattern.
To watch `events/Todo/**` AND `events/User/**`, create two separate subscribers and multiplex them with `tokio::select!`.
Alternatively, use a broader pattern like `events/**` if both patterns share a common prefix.

## Publishing patterns

After appending events to the SQLite event store, Ironstar publishes them to Zenoh for routing to active subscribers.

```rust
// After appending to SQLite event store
let key = format!("events/{}/{}", event.aggregate_type, event.aggregate_id);
session.put(&key, event.payload.as_bytes()).await?;
```

**Key expression structure:**

| Pattern | Matches | Use Case |
|---------|---------|----------|
| `events/{type}/{id}` | Single aggregate instance | Specific entity SSE feed |
| `events/{type}/**` | All instances of type | Type-wide projection updates |
| `events/**` | All events | Global audit log, analytics |
| `sessions/{session_id}/**` | All events for session | Session-scoped SSE feeds |

See `session-management.md` for session-scoped routing patterns.

## Subscription patterns

### Single pattern subscription

```rust
let subscriber = session.declare_subscriber("events/Todo/**")
    .with(flume::bounded(32))
    .await?;

while let Ok(sample) = subscriber.recv_async().await {
    let key_expr = sample.key_expr();
    let payload = sample.payload();
    // Process Todo events
}
```

### Multi-pattern subscription with select

```rust
let todo_sub = session.declare_subscriber("events/Todo/**").await?;
let user_sub = session.declare_subscriber("events/User/**").await?;

loop {
    tokio::select! {
        sample = todo_sub.recv_async() => {
            let sample = sample?;
            handle_todo_event(&sample);
        }
        sample = user_sub.recv_async() => {
            let sample = sample?;
            handle_user_event(&sample);
        }
    }
}
```

### SSE handler with specific subscription

```rust
async fn sse_todo_feed(
    State(app_state): State<AppState>,
    Path(todo_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Subscribe only to events for this specific Todo
    let key_expr = format!("events/Todo/{}", todo_id);
    let subscriber = app_state.zenoh.declare_subscriber(&key_expr).await.unwrap();

    let stream = async_stream::stream! {
        while let Ok(sample) = subscriber.recv_async().await {
            yield Ok(sample_to_sse_event(&sample));
        }
    };

    Sse::new(stream)
}
```

## Zenoh storage considerations

Zenoh includes a storage subsystem designed for distributed data synchronization, not caching.
Ironstar does not use Zenoh storage for analytics caching.

**Why not use Zenoh storage for caching:**

| Feature | moka (used) | Zenoh Storage (not used) |
|---------|-------------|--------------------------|
| Per-key TTL | Yes (`time_to_live()`) | No |
| Time-to-idle | Yes (`time_to_idle()`) | No |
| Size-based eviction | Yes (`max_capacity()`) | No |
| Eviction policy | TinyLFU / LRU | None |
| Synchronous fast path | Yes | No (always async query/reply) |

Zenoh storage keeps the latest value per key with no automatic eviction.
Its `GarbageCollectionConfig` only affects metadata (timestamp history), not actual data values.
This model is fundamentally different from cache eviction policies.

**Separation of concerns:**

- Zenoh: Event notification and routing (pub/sub)
- moka: Analytics caching (TTL, eviction, fast reads)
- SQLite: Durable event storage (source of truth)

See `analytics-cache-architecture.md` for cache invalidation patterns using Zenoh subscriptions.

## Session-scoped routing

Ironstar uses Zenoh key expressions to route events scoped to specific user sessions.
This enables per-session SSE feeds that receive only events relevant to the authenticated user.

**Key expression pattern:**

```
sessions/{session_id}/events/{aggregate_type}/{aggregate_id}
```

**Publishing with session context:**

```rust
// Publish to both global and session-scoped key expressions
let global_key = format!("events/{}/{}", event.aggregate_type, event.aggregate_id);
let session_key = format!("sessions/{}/events/{}/{}",
    session_id, event.aggregate_type, event.aggregate_id);

session.put(&global_key, payload.clone()).await?;
session.put(&session_key, payload).await?;
```

**Session-scoped SSE subscription:**

```rust
async fn sse_session_feed(
    State(app_state): State<AppState>,
    Extension(session_id): Extension<SessionId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let key_expr = format!("sessions/{}/**", session_id);
    let subscriber = app_state.zenoh.declare_subscriber(&key_expr).await.unwrap();

    let stream = async_stream::stream! {
        while let Ok(sample) = subscriber.recv_async().await {
            yield Ok(sample_to_sse_event(&sample));
        }
    };

    Sse::new(stream)
}
```

See `session-management.md` for complete session lifecycle and security patterns.

## Testing and debugging

### Unit testing key expression matching

```rust
#[tokio::test]
async fn test_zenoh_key_expression_matching() {
    let config = zenoh_embedded_config();
    let session = zenoh::open(config).await.unwrap();

    let subscriber = session.declare_subscriber("events/Todo/**").await.unwrap();

    session.put("events/Todo/123", b"test").await.unwrap();

    let sample = tokio::time::timeout(
        Duration::from_millis(100),
        subscriber.recv_async()
    ).await.expect("timeout").expect("recv failed");

    assert_eq!(sample.key_expr().as_str(), "events/Todo/123");
}
```

### Integration testing SSE feeds

```rust
#[tokio::test]
async fn test_sse_feed_receives_published_events() {
    let app_state = test_app_state().await;
    let session_id = SessionId::new();

    // Start SSE subscription
    let key_expr = format!("sessions/{}/events/**", session_id);
    let subscriber = app_state.zenoh.declare_subscriber(&key_expr).await.unwrap();

    // Publish event
    let event = StoredEvent { /* ... */ };
    publish_event(&app_state, &session_id, event).await.unwrap();

    // Verify receipt
    let sample = tokio::time::timeout(
        Duration::from_millis(100),
        subscriber.recv_async()
    ).await.expect("timeout").expect("recv failed");

    let received: StoredEvent = serde_json::from_slice(sample.payload().to_bytes().as_ref()).unwrap();
    assert_eq!(received.aggregate_id, "123");
}
```

### Monitoring metrics

| Metric | Healthy | Investigate |
|--------|---------|-------------|
| SSE connection count | Stable | Sudden drops |
| Event latency | <200ms | >500ms |
| Zenoh subscriber count | Matches SSE connections | Mismatch |
| Error rate | <0.1% | >1% |
| Memory usage | <50MB per 1000 subscribers | >100MB |

### Debugging connection issues

Enable Zenoh logging to diagnose subscription or publishing problems:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new("zenoh=debug"))
    .with(tracing_subscriber::fmt::layer())
    .init();
```

Common issues:

| Symptom | Cause | Fix |
|---------|-------|-----|
| Subscriber never receives samples | Key expression mismatch | Verify pattern matches published key |
| Events arrive late | Channel buffer overflow | Increase `.with(flume::bounded(N))` size |
| Memory leak | Subscribers not closed | Ensure subscriber drop on SSE disconnect |

## Migration guidance for template users

Template users adapting ironstar for existing codebases may need to transition from `tokio::sync::broadcast` to Zenoh.
This section provides coexistence and migration strategies.

### Coexistence pattern

For gradual migration, publish to both broadcast and Zenoh while subscribers transition incrementally.

```rust
/// Dual-publish to both broadcast and Zenoh during migration
pub struct DualEventBus {
    broadcast: broadcast::Sender<StoredEvent>,
    zenoh: Arc<Session>,
}

impl DualEventBus {
    pub async fn publish(&self, event: StoredEvent) -> Result<(), Error> {
        // Publish to both (order doesn't matter since SQLite is source of truth)
        let _ = self.broadcast.send(event.clone());

        let key = format!("events/{}/{}", event.aggregate_type, event.aggregate_id);
        self.zenoh.put(&key, serde_json::to_vec(&event)?).await?;

        Ok(())
    }
}
```

Dual-publishing ensures existing broadcast subscribers continue working while Zenoh-based subscribers are tested in parallel.

### Migration sequence

**Phase 1: Add Zenoh alongside broadcast**

Keep broadcast code working while adding Zenoh publishing.

1. Add Zenoh session to AppState
2. Dual-publish all events (broadcast + Zenoh)
3. All existing subscribers continue using broadcast
4. Add integration tests for Zenoh publishing

**Phase 2: Migrate subscribers incrementally**

Migrate one SSE handler at a time, starting with least critical components.

```rust
// Feature flag for gradual migration
#[derive(Clone)]
pub struct AppState {
    pub event_bus: DualEventBus,
    pub use_zenoh_for_sse: bool,
}

async fn sse_feed(
    State(app): State<AppState>,
    headers: HeaderMap,
) -> Sse<impl Stream<...>> {
    if app.use_zenoh_for_sse {
        sse_feed_zenoh(app, headers).await
    } else {
        sse_feed_broadcast(app, headers).await
    }
}
```

Recommended migration order:

| Order | Component | Risk | Rationale |
|-------|-----------|------|-----------|
| 1 | Analytics dashboard SSE | Low | Non-critical, tolerate stale data |
| 2 | Projection updaters | Medium | Can rebuild from event store |
| 3 | Primary UI SSE feeds | High | User-facing, test thoroughly |

**Phase 3: Remove broadcast**

Once all subscribers have migrated:

1. Remove broadcast channel from DualEventBus
2. Remove broadcast Receiver usage
3. Remove `use_zenoh_for_sse` feature flag
4. Update documentation

### Rollback procedure

For template users experiencing issues during migration:

1. Set `use_zenoh_for_sse = false` in config
2. Restart the service
3. Verify broadcast-based SSE is working
4. Investigate Zenoh logs for errors
5. Fix root cause before re-enabling

### When to use migration vs fresh start

**Fresh Zenoh integration**: New ironstar template instantiations start with Zenoh by default.

**Gradual migration**: Existing projects with broadcast subscribers in production use the coexistence pattern above.

**Profile first**: If embedded mode overhead is a concern, benchmark before migrating (typical overhead: ~2MB memory, ~100μs latency).

## Related documentation

- Session-scoped event routing patterns: `session-management.md`
- Analytics cache invalidation via Zenoh: `analytics-cache-architecture.md`
- Event sourcing integration: `../cqrs/event-sourcing-core.md`
