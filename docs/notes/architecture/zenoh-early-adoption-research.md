# Zenoh early adoption research

This document investigates whether Zenoh should be adopted from the start of Ironstar development instead of the current `tokio::sync::broadcast` + `moka` stack.

## Executive summary

**Recommendation: Keep the current stack (`broadcast` + `moka`) for initial development.**

Zenoh is a powerful distributed pub/sub system, but its strengths don't align with Ironstar's single-node deployment target.
The overhead of Zenoh's runtime, the lack of cache-appropriate features in its storage layer, and the added configuration complexity outweigh the benefits of early adoption.

**Key findings:**

1. Zenoh lacks a true "embedded only" mode; it requires explicit configuration to disable networking
2. Zenoh storage has no per-key TTL or size-based eviction (unsuitable as moka replacement)
3. Key expression filtering is powerful but requires one subscriber per pattern (no union semantics)
4. The current stack is simpler, has fewer dependencies, and meets all single-node requirements

**When to reconsider Zenoh:**

- Multi-node deployment becomes a requirement
- Event bus filtering at subscription time becomes a performance bottleneck
- The application needs distributed storage with automatic synchronization

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

| Dimension | broadcast + moka | Zenoh | Winner |
|-----------|------------------|-------|--------|
| **In-process latency** | ~10 μs (broadcast clone) | ~100+ μs (runtime overhead) | broadcast |
| **Memory footprint** | ~50 KB base | ~2+ MB (runtime structures) | broadcast |
| **Dependency count** | tokio (0 extra), moka (~10) | zenoh (~50+) | broadcast |
| **Configuration complexity** | Zero config | Requires embedded config | broadcast |
| **Cache features** | Full (TTL, TTI, eviction) | None applicable | moka |
| **Subscription filtering** | None (manual) | Key expressions | Zenoh |
| **Multi-pattern subscription** | N/A | One sub per pattern | Tie |
| **Distribution readiness** | None (redesign needed) | Native | Zenoh |
| **API ergonomics** | Simple, Rust-native | More complex, config-driven | broadcast |
| **Compile time** | Fast | Slower (large crate) | broadcast |

**Weighted assessment for single-node deployment:**

- Configuration complexity matters in early development
- Cache features are essential for analytics workload
- Distribution readiness is future concern, not immediate
- Development velocity favors simpler stack

## Recommendation

### Keep current stack for now

The `tokio::sync::broadcast` + `moka` combination is the right choice for Ironstar's single-node deployment target.

**Rationale:**

1. **Zenoh storage cannot replace moka.** The lack of per-key TTL, time-to-idle, and size-based eviction makes it unsuitable for analytics caching.
   Moka would need to be retained regardless.

2. **Zenoh pub/sub advantage is marginal for single-node.** Key expression filtering helps when network bandwidth matters.
   In-process, the cost of receiving all events and filtering client-side is minimal.

3. **Configuration overhead is real.** Zenoh requires explicit configuration to disable networking.
   The current stack requires zero configuration.

4. **Dependency weight matters.** Zenoh adds ~50+ transitive dependencies.
   For a template project, leaner dependencies are preferable.

5. **Development velocity.** The current stack is simpler to reason about, debug, and maintain during early development.

### When to reconsider Zenoh

Zenoh becomes valuable when:

1. **Multi-node deployment is required.** Zenoh's distributed pub/sub eliminates the need for external message brokers.

2. **Event filtering becomes a bottleneck.** If hundreds of projections are receiving and filtering all events, server-side filtering saves CPU cycles.

3. **Distributed state synchronization is needed.** Zenoh's storage backends provide automatic replication across nodes.

### Migration path

When the time comes, the migration from broadcast to Zenoh is straightforward:

1. Replace `broadcast::channel()` with `zenoh::open()`
2. Replace `tx.send()` with `session.put()`
3. Replace manual filtering with key expression subscriptions
4. Keep moka for analytics caching (Zenoh doesn't replace it)

The event store (SQLite) and analytics engine (DuckDB) remain unchanged.
SSE handlers adapt from broadcast to Zenoh subscribers.
The hypertext templating layer is unaffected.

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

Neither is ideal for the current single-node Ironstar target, where the embedded `broadcast + moka` stack is optimal.
