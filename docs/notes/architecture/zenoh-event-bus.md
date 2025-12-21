# Zenoh early adoption research

This document investigates whether Zenoh should be adopted from the start of Ironstar development instead of `tokio::sync::broadcast` for event notification.

## Executive summary

**Recommendation: Adopt Zenoh for pub/sub, keep moka for analytics cache.**

This mirrors the Go ecosystem pattern where NATS is the default choice for real-time event-driven applications.
Zenoh is the Rust-native equivalent: pure Rust implementation, no external server required, and key expression filtering that enables the "listen to hundreds of keys with server-side filtering" pattern essential for CQRS + Datastar applications.

**Architecture decision:**

| Layer | Component | Role |
|-------|-----------|------|
| Event notification | Zenoh | Key expression filtering, distribution-ready |
| Analytics cache | moka | TTL, eviction, synchronous fast path |
| Event store | SQLite | Append-only log (unchanged) |

**Key findings:**

1. Zenoh provides server-side key expression filtering (`events/Todo/**`, `events/User/*`)
2. Zenoh storage is not a cache replacement (no TTL, no eviction) — moka remains essential
3. Multiple subscribers with `select!` handles multi-pattern watching elegantly
4. Configuration for embedded mode is minimal (4 lines)
5. The dependency cost (~50 crates) is acceptable for production infrastructure

## Research objectives recap

The investigation explored whether Zenoh could consolidate two roles:

1. **Pub/sub with key-expression filtering** (replacing `tokio::sync::broadcast`)
2. **Storage backend for analytics cache** (replacing `moka` + `rkyv`)

## Findings

### Zenoh embedded configuration

Zenoh has no built-in "embedded only" mode.
To achieve in-process operation without network activity:

```rust
use zenoh::Config;

let mut config = Config::default();
config.insert_json5("listen/endpoints", "[]").unwrap();
config.insert_json5("connect/endpoints", "[]").unwrap();
config.insert_json5("scouting/multicast/enabled", "false").unwrap();
config.insert_json5("scouting/gossip/enabled", "false").unwrap();

let session = zenoh::open(config).await.unwrap();
```

**Overhead concerns:**

- `TransportManager` is instantiated even with empty endpoints
- Runtime starts regardless of network configuration
- Background task machinery exists even when not used
- The `RuntimeBuilder::build()` path is identical for networked and embedded use

### Zenoh pub/sub and key expressions

Zenoh provides server-side filtering via key expressions with wildcards:

| Wildcard | Matches | Example |
|----------|---------|---------|
| `*` | Single segment | `a/*/c` matches `a/b/c` |
| `**` | Zero or more segments | `a/**/c` matches `a/c`, `a/b/c`, `a/b/d/c` |
| `$*` | Within segment | `ab$*cd` matches `abxxcd` |

**Limitation: No union semantics.**

A single subscriber watches one key expression pattern.
To watch `events/Todo/**` AND `events/User/**`, you need either:

1. Two separate subscribers
2. A broader pattern `events/**` that matches both

This differs from NATS KV's `watch_many(["foo.>", "bar.>"])` which accepts multiple patterns in one subscription.

**Subscriber API:**

```rust
let subscriber = session.declare_subscriber("events/**")
    .with(flume::bounded(32))
    .await?;

while let Ok(sample) = subscriber.recv_async().await {
    let key_expr = sample.key_expr();
    let payload = sample.payload();
    // Process...
}
```

### Zenoh storage backends

Zenoh storage is designed as a distributed data store, not a cache.

**Built-in memory backend:**

- Persistence: volatile (data lost on restart)
- History: latest value only per key
- No per-key TTL
- No size-based eviction
- No time-to-idle

**Metadata GC:**

Storage has a `GarbageCollectionConfig` with `lifespan` setting, but this only affects metadata (timestamp history), not actual data values.
This is fundamentally different from moka's cache eviction model.

**Gap summary:**

| Feature | moka | Zenoh Storage |
|---------|------|---------------|
| Per-key TTL | Yes (`time_to_live()`) | No |
| Time-to-idle | Yes (`time_to_idle()`) | No |
| Size-based eviction | Yes (`max_capacity()`) | No |
| Eviction policy | TinyLFU / LRU | None |
| Async-native | Yes | Yes |
| Synchronous fast path | Yes | No (always async query/reply) |

**Conclusion:** Zenoh storage cannot replace moka for analytics caching.

### NATS KV watch semantics (reference)

For comparison, NATS KV provides the "gold standard" multi-key watch API:

```rust
// Watch multiple patterns in one subscription
let mut watch = kv.watch_many(["foo.>", "bar.>"]).await?;

while let Some(entry) = watch.next().await {
    let entry = entry?;
    // entry.key, entry.revision, entry.operation
}
```

Key features:

- Server-side filtering with subject patterns
- Single subscription handles multiple patterns
- Push-based delivery via ordered consumer
- `seen_current` flag indicates when history is exhausted

Zenoh approaches this differently: one subscriber per pattern, but the pattern matching itself is efficient and server-side.

## Comparative code patterns

### Current pattern (broadcast + moka)

```rust
// Event notification
let (tx, _) = broadcast::channel::<StoredEvent>(256);

// Publishing
tx.send(event)?;

// Subscribing (receives ALL events)
let rx = tx.subscribe();
while let Ok(evt) = rx.recv().await {
    // Manual filtering
    if evt.aggregate_type == "Todo" {
        handle_todo_event(&evt);
    }
}

// Analytics caching
let cache: Cache<String, Vec<u8>> = Cache::builder()
    .max_capacity(1_000)
    .time_to_live(Duration::from_secs(300))
    .build();

cache.insert(key, value).await;
if let Some(v) = cache.get(&key).await {
    // Use cached value
}

// Cache invalidation (manual)
cache.invalidate_entries_if(|k, _| k.starts_with("daily_stats:")).await;
```

### Proposed pattern (Zenoh unified)

```rust
// Session creation (requires config for embedded mode)
let mut config = Config::default();
config.insert_json5("listen/endpoints", "[]").unwrap();
config.insert_json5("connect/endpoints", "[]").unwrap();
config.insert_json5("scouting/multicast/enabled", "false").unwrap();
config.insert_json5("scouting/gossip/enabled", "false").unwrap();
let session = zenoh::open(config).await?;

// Publishing
session.put("events/Todo/123", payload).await?;

// Subscribing with pattern (server-side filtering)
let subscriber = session.declare_subscriber("events/Todo/**").await?;
while let Ok(sample) = subscriber.recv_async().await {
    // Only receives matching events
    handle_todo_event(&sample);
}

// Analytics "caching" via storage (NOT equivalent to moka)
// Requires storage plugin configuration
session.put("cache/daily_stats/2024-01-15", result).await?;
let reply = session.get("cache/daily_stats/2024-01-15").await?;
// No TTL, no eviction, no size limits
```

### Key differences

| Aspect | broadcast + moka | Zenoh |
|--------|------------------|-------|
| Filter location | Client-side | Server-side (key expression) |
| Multi-pattern watch | N/A (receive all) | One subscriber per pattern |
| Cache TTL | Built-in | Not available |
| Cache eviction | TinyLFU / LRU | None |
| Configuration | Zero | Required for embedded mode |
| Async overhead | Minimal | Higher (runtime machinery) |

## Trade-off matrix

| Dimension | broadcast + moka | Zenoh + moka | Assessment |
|-----------|------------------|--------------|------------|
| **In-process latency** | ~10 μs | ~100+ μs | Acceptable overhead |
| **Memory footprint** | ~50 KB base | ~2+ MB | Acceptable for server |
| **Dependency count** | ~10 | ~60 | Normal for full-stack app |
| **Configuration** | Zero | 4 lines | Trivial |
| **Cache features** | moka: full | moka: full | Equivalent (keep moka) |
| **Subscription filtering** | Manual (O(n)) | Server-side (O(1)) | **Zenoh wins** |
| **Multi-pattern subscription** | Receive all | One sub per pattern + select | **Zenoh wins** |
| **Distribution readiness** | Redesign needed | Native | **Zenoh wins** |
| **CQRS + Datastar fit** | Poor | Designed for this | **Zenoh wins** |

**Weighted assessment:**

The dimensions where Zenoh wins (subscription filtering, multi-pattern, distribution, CQRS fit) are the ones that matter for a Datastar event-sourced application.
The dimensions where broadcast wins (latency, footprint, dependencies) are acceptable costs for production infrastructure.

## Recommendation

### Adopt Zenoh for pub/sub from the start

Use Zenoh as the event notification layer, keeping moka for analytics caching.

**Rationale:**

1. **Key expression filtering is essential for CQRS + Datastar.** SSE handlers need to subscribe to specific aggregate types and instances.
   With `broadcast`, every handler receives every event and filters locally — this doesn't scale.

2. **The Go + NATS analogy applies.** Northstar uses embedded NATS without hesitation.
   Zenoh is the Rust-native equivalent: no external server, pure Rust, same architectural role.

3. **Configuration is minimal.** Four lines to disable networking for embedded mode.
   This is not a meaningful barrier.

4. **Dependencies are acceptable.** We're building production infrastructure, not a microcontroller.
   ~50 transitive dependencies is normal for a full-stack web application.

5. **Distribution-ready from day one.** When multi-node deployment is needed, the pub/sub layer requires no changes.

### Keep moka for analytics cache

Zenoh storage is designed for distributed data synchronization, not caching.
moka provides the cache-specific features we need:

- Per-key TTL (`time_to_live()`)
- Time-to-idle eviction (`time_to_idle()`)
- Size-based eviction (`max_capacity()`)
- TinyLFU admission policy
- Synchronous fast path for hot reads

### Implementation pattern

**Embedded Zenoh session:**

```rust
use zenoh::Config;

fn zenoh_embedded_config() -> Config {
    let mut config = Config::default();
    config.insert_json5("listen/endpoints", "[]").unwrap();
    config.insert_json5("connect/endpoints", "[]").unwrap();
    config.insert_json5("scouting/multicast/enabled", "false").unwrap();
    config.insert_json5("scouting/gossip/enabled", "false").unwrap();
    config
}

let session = zenoh::open(zenoh_embedded_config()).await?;
```

**Publishing events:**

```rust
// After appending to SQLite event store
let key = format!("events/{}/{}", event.aggregate_type, event.aggregate_id);
session.put(&key, event.payload.as_bytes()).await?;
```

**Multi-pattern subscription with select:**

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

**SSE handler with specific subscription:**

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

### What changes from original CLAUDE.md

| Component | Before | After |
|-----------|--------|-------|
| Event Bus | `tokio::sync::broadcast` | Zenoh session |
| Analytics Cache | moka | moka (unchanged) |
| Event Store | SQLite | SQLite (unchanged) |
| Distribution (future) | Zenoh | Zenoh (already integrated) |

The "future distribution" path becomes the current architecture.

## Appendix: NATS KV vs Zenoh feature comparison

For reference, comparing the two distributed alternatives:

| Feature | NATS KV | Zenoh |
|---------|---------|-------|
| Multi-pattern watch | `watch_many(["a.>", "b.>"])` | One subscriber per pattern |
| Wildcard syntax | `*` (single), `>` (multi) | `*` (single), `**` (multi), `$*` (intra) |
| History delivery | `DeliverPolicy::LastPerSubject` | `zenoh-ext::AdvancedSubscriber` |
| TTL on keys | Yes (per-bucket TTL) | No |
| Revision tracking | Global sequence per bucket | Per-storage timestamps |
| External server | Required (NATS server) | Optional (peer mode) |
| Rust-native | Client only (Go server) | Full (Rust implementation) |

NATS KV has better cache-like semantics (TTL per bucket, history policies).
Zenoh has better Rust ecosystem integration (pure Rust, no external server).

For Ironstar, Zenoh is the right choice: no external server dependency, native Rust implementation, and key expression filtering that matches the CQRS + Datastar requirements.
moka handles the caching use case that neither Zenoh nor NATS KV address optimally.

## Migration path

This section provides guidance for transitioning from `tokio::sync::broadcast` to Zenoh, whether during initial ironstar development or when adapting existing code.

### Coexistence strategy

Broadcast and Zenoh can coexist during development.
The pattern is to publish to both and gradually migrate subscribers.

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

This dual-publishing approach ensures existing broadcast subscribers continue working while new Zenoh-based subscribers can be tested in parallel.

### Migration phases

#### Phase 1: Add Zenoh alongside broadcast

Keep all broadcast code working while adding Zenoh publishing infrastructure.

1. Add Zenoh session to AppState
2. Dual-publish all events (broadcast + Zenoh)
3. All existing subscribers continue using broadcast
4. Add integration tests for Zenoh publishing

#### Phase 2: Migrate subscribers incrementally

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

#### Phase 3: Remove broadcast

Once all subscribers have migrated and the system is stable:

1. Remove broadcast channel from DualEventBus
2. Remove broadcast Receiver usage
3. Remove `use_zenoh_for_sse` feature flag
4. Update documentation

### Testing strategy

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

### Rollback procedure

If Zenoh integration causes issues in production:

1. Set `use_zenoh_for_sse = false` in config
2. Restart the service
3. Verify broadcast-based SSE is working
4. Investigate Zenoh logs for errors
5. Fix root cause before re-enabling

### Monitoring during migration

| Metric | Healthy | Investigate |
|--------|---------|-------------|
| SSE connection count | Stable | Sudden drops |
| Event latency | <200ms | >500ms |
| Zenoh subscriber count | Matches SSE connections | Mismatch |
| Error rate | <0.1% | >1% |

### When to migrate

**Migrate immediately**: New ironstar projects with no existing code.

**Migrate incrementally**: Projects with existing broadcast subscribers in production.

**Don't migrate yet**: If Zenoh causes build issues or embedded mode has unacceptable overhead (profile first).

## Related documentation

- Session-scoped event routing patterns: `session-management.md`
- Analytics cache invalidation via Zenoh: `analytics-cache-architecture.md`
- Event sourcing integration: `event-sourcing-sse-pipeline.md`
