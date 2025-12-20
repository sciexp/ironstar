# Analytics cache architecture

Research and design for caching expensive DuckDB analytics queries in the ironstar template.

## Problem statement

DuckDB queries for dashboards are expensive (seconds, not milliseconds) and dashboards are hit frequently by multiple users.
The current architecture uses:
- SQLite + sqlx for the event store (append-only log)
- DuckDB for OLAP analytics queries over event history
- tokio::sync::broadcast for in-process pub/sub for SSE notifications

This document evaluates caching strategies to avoid redundant expensive DuckDB queries while maintaining consistency with the event-sourced architecture.

## Evaluation criteria

| Criterion | Weight | Description |
|-----------|--------|-------------|
| Async compatibility | High | Must integrate cleanly with tokio async runtime |
| Binary blob storage | High | Analytics results are serialized, not structured KV |
| Invalidation precision | Medium | Per-aggregate-type or time-based invalidation |
| Persistence | Low | Cache is rebuildable from DuckDB; nice-to-have |
| Memory footprint | Medium | Modest hardware target |
| Complexity | Medium | Prefer simpler solutions |

## Candidate evaluation

### 1. moka (in-memory cache)

[moka](https://github.com/moka-rs/moka) is a high-performance concurrent caching library for Rust inspired by Java's Caffeine.

**Strengths:**
- Async-native with `moka::future::Cache`
- TTL (time-to-live) and size-based eviction built in
- Lock-free concurrent access (no Arc<Mutex<...>> needed)
- LFU-based eviction maintains near-optimal hit ratio
- Eviction listener support for cleanup callbacks
- Actively maintained, MSRV 1.70+

**Weaknesses:**
- No persistence (cache lost on restart)
- Memory-only (limited by available RAM)

**API pattern:**

```rust
use moka::future::Cache;
use std::time::Duration;

// Create cache with TTL and size limit
let cache: Cache<String, Vec<u8>> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(300)) // 5 minute TTL
    .build();

// Insert serialized analytics result
cache.insert(cache_key, serialized_result).await;

// Retrieve (returns Option)
if let Some(result) = cache.get(&cache_key).await {
    return deserialize(result);
}
```

### 2. redb (embedded ACID KV)

[redb](https://github.com/cberner/redb) is an embedded key-value database in pure Rust with ACID semantics.

**Analysis from design.md:**
- Copy-on-write B-tree structure (never corrupts on crash)
- Memory-mapped reads provide excellent read performance for hot data
- Configurable durability: `Durability::None` (non-durable), `Durability::Eventual` (1PC+C with checksum), `Durability::Immediate` (2PC)
- Supports arbitrary byte slices as values (suitable for binary blobs)
- Single-writer, multiple-reader MVCC isolation
- XXH3_128 checksums detect partial writes

**Key design considerations from redb's design.md:**
- "Non-durable commits" provide atomicity without fsync (good for cache)
- Savepoints enable point-in-time snapshots (not needed for cache)
- Memory-mapped I/O means hot data stays in page cache

**Weaknesses:**
- Synchronous API (needs `spawn_blocking` in async handlers)
- No built-in TTL (must implement expiration logic)
- Persistence overhead (unnecessary for pure cache use case)

**API pattern:**

```rust
use redb::{Database, TableDefinition, Durability};

const ANALYTICS_CACHE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("analytics_cache");

// Open database
let db = Database::create("analytics_cache.redb")?;

// Write with non-durable (cache-appropriate) durability
let txn = db.begin_write()?;
txn.set_durability(Durability::None);
{
    let mut table = txn.open_table(ANALYTICS_CACHE)?;
    table.insert(cache_key, serialized_result.as_slice())?;
}
txn.commit()?;

// Read (memory-mapped, fast for hot data)
let txn = db.begin_read()?;
let table = txn.open_table(ANALYTICS_CACHE)?;
let result = table.get(cache_key)?;
```

### 3. SQLite table (cache alongside event store)

Use a dedicated SQLite table in the existing database for cache storage.

**Strengths:**
- Already in the stack (no new dependency)
- sqlx provides async interface
- Can use WAL mode shared with event store

**Weaknesses:**
- Mixes cache (ephemeral) with source-of-truth (durable)
- SQLite single-writer constraint affects both cache and events
- Conceptually confusing (cache should be separate from primary data)

**Not recommended:** Violates separation of concerns and risks write contention.

### 4. sled (embedded database)

[sled](https://github.com/spacejam/sled) is a lock-free embedded database in Rust.

**Strengths:**
- Lock-free concurrent access
- Append-only log structure with crash safety
- Watch/subscribe functionality for change notifications

**Weaknesses:**
- Unstable 1.0-alpha status (edition = "2024")
- Known space amplification issues ("uses too much space sometimes")
- Author working on major rewrite (komora/marble storage engine)
- Less predictable than redb for write-heavy workloads

**Not recommended:** Stability concerns for a template project.
Benchmark data shows redb outperforms sled on individual writes (227ms vs 642ms) and large writes (8805ms vs 37736ms).

## Serialization format evaluation

Analytics query results must be serialized for cache storage.
The choice affects both storage efficiency and read performance.

### Comparison matrix

| Format | Serialize | Deserialize | Size | Zero-copy | DuckDB native |
|--------|-----------|-------------|------|-----------|---------------|
| bincode | Fast | Fast | Compact | No | No |
| rkyv | Fast | **Ultra-fast** (21ns) | Compact | **Yes** | No |
| Arrow IPC | Fast | Fast | Larger | Yes | **Yes** |
| JSON | Slow | Slow | Large | No | No |

### Analysis

**bincode:**
- General-purpose, well-understood
- Good performance for both serialize and deserialize
- ~300ns deserialize (vs rkyv's 21ns)
- Already used in ironstar for session storage (redb)

**rkyv:**
- Zero-copy deserialization is ideal for read-heavy caches
- 10-15x faster deserialization than bincode
- Requires `#[derive(Archive, Serialize, Deserialize)]` on types
- Schema evolution is more constrained than bincode

**Arrow IPC:**
- Native DuckDB format eliminates encode/decode for columnar data
- "Arrow's ability to eliminate encoding and decoding overheads typically yields faster and more efficient data interchange"
- Best for scenarios where query results are columnar and may be re-processed
- Larger than bincode/rkyv for small results

### Recommendation

Use **rkyv** for analytics cache serialization:
- Zero-copy deserialization aligns with cache read-heavy access pattern
- Benchmarks show 10-15x deserialization speedup over bincode
- Analytics results are typically read many times per write

For structured columnar results that need further processing (e.g., feeding into visualization), consider Arrow IPC for interoperability.

## Cache invalidation patterns

### Pattern 1: Per-aggregate-type invalidation

When events arrive, invalidate cache entries based on the aggregate type in the event.

```rust
use std::collections::HashSet;
use tokio::sync::broadcast;

/// Cache key structure
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct AnalyticsCacheKey {
    /// Query identifier (e.g., "daily_event_counts")
    pub query_id: String,
    /// Aggregate types this query depends on
    pub depends_on: HashSet<String>,
}

/// Invalidation logic
pub fn should_invalidate(
    cache_key: &AnalyticsCacheKey,
    event: &StoredEvent
) -> bool {
    cache_key.depends_on.is_empty()
        || cache_key.depends_on.contains(&event.aggregate_type)
}

/// Background invalidation task
pub async fn run_cache_invalidator(
    cache: Cache<String, Vec<u8>>,
    mut event_rx: broadcast::Receiver<StoredEvent>,
    cache_registry: Arc<RwLock<HashMap<String, AnalyticsCacheKey>>>,
) {
    while let Ok(event) = event_rx.recv().await {
        let registry = cache_registry.read().await;
        for (key, meta) in registry.iter() {
            if should_invalidate(meta, &event) {
                cache.invalidate(key).await;
            }
        }
    }
}
```

### Pattern 2: Time-window based invalidation

For analytics queries with time boundaries (e.g., "daily stats"), cache until the window closes.

```rust
use chrono::{DateTime, Utc, Duration};

#[derive(Clone)]
pub struct TimeWindowCache {
    cache: Cache<String, (DateTime<Utc>, Vec<u8>)>,
}

impl TimeWindowCache {
    /// Get cached result if still within the valid time window
    pub async fn get_if_valid(
        &self,
        key: &str,
        window_end: DateTime<Utc>,
    ) -> Option<Vec<u8>> {
        self.cache.get(key).await.and_then(|(cached_until, data)| {
            if Utc::now() < cached_until && cached_until <= window_end {
                Some(data)
            } else {
                None
            }
        })
    }

    /// Cache result until the specified time window closes
    pub async fn insert_with_window(
        &self,
        key: String,
        data: Vec<u8>,
        valid_until: DateTime<Utc>,
    ) {
        self.cache.insert(key, (valid_until, data)).await;
    }
}

// Example: Daily stats cache expires at midnight UTC
fn daily_stats_cache_key(date: chrono::NaiveDate) -> (String, DateTime<Utc>) {
    let key = format!("daily_stats:{}", date);
    let expires = date.succ_opt()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(|| Utc::now() + Duration::days(1));
    (key, expires)
}
```

### Pattern 3: Event-driven cache refresh

Subscribe to broadcast channel and proactively refresh cache entries on relevant events.

```rust
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub struct ProactiveCache {
    cache: Cache<String, Vec<u8>>,
    analytics: Arc<AnalyticsService>,
}

impl ProactiveCache {
    /// Spawn background task that refreshes cache on events
    pub fn spawn_refresh_task(
        self: Arc<Self>,
        mut event_rx: broadcast::Receiver<StoredEvent>,
    ) {
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                // Determine which queries need refresh based on event
                let queries_to_refresh = self.queries_affected_by(&event);

                for query_id in queries_to_refresh {
                    // Refresh in background (don't block event processing)
                    let self_clone = self.clone();
                    tokio::spawn(async move {
                        if let Ok(result) = self_clone.analytics
                            .execute_query(&query_id).await
                        {
                            self_clone.cache.insert(query_id, result).await;
                        }
                    });
                }
            }
        });
    }

    fn queries_affected_by(&self, event: &StoredEvent) -> Vec<String> {
        // Return query IDs that depend on this event's aggregate type
        // This mapping would be configured at startup
        vec![]
    }
}
```

## SSE/Datastar integration

### How cache updates trigger SSE notifications

When a cached analytics result changes, connected dashboards should receive the update via SSE.

```rust
use axum::response::sse::{Event, Sse};
use datastar::prelude::*;
use futures::stream::{self, Stream, StreamExt};

/// SSE handler for analytics dashboard
pub async fn analytics_sse(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Subscribe to analytics updates
    let mut rx = app_state.analytics_bus.subscribe();

    // Send initial state
    let initial = app_state.analytics_cache.get_current_dashboard().await;
    let initial_event = render_dashboard(&initial).to_patch_elements();

    let stream = stream::once(async move { Ok(initial_event.into()) })
        .chain(
            tokio_stream::wrappers::BroadcastStream::new(rx)
                .filter_map(|result| async move {
                    result.ok().map(|update| {
                        Ok(render_dashboard(&update).to_patch_elements().into())
                    })
                })
        );

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
    )
}
```

### Multiple dashboard viewers sharing cached data

The moka cache is inherently concurrent-safe.
All SSE connections read from the same cache instance without coordination.

```rust
/// AppState shared across all handlers
#[derive(Clone)]
pub struct AppState {
    // ... existing fields ...

    /// Analytics cache (shared across all connections)
    pub analytics_cache: Cache<String, Vec<u8>>,

    /// Bus for analytics updates (distinct from event bus)
    pub analytics_bus: broadcast::Sender<AnalyticsUpdate>,
}

/// On cache refresh, notify all connected dashboards
async fn refresh_and_notify(
    cache: &Cache<String, Vec<u8>>,
    bus: &broadcast::Sender<AnalyticsUpdate>,
    query_id: &str,
    result: Vec<u8>,
) {
    // Update cache
    cache.insert(query_id.to_string(), result.clone()).await;

    // Notify all subscribers (fire and forget)
    let _ = bus.send(AnalyticsUpdate {
        query_id: query_id.to_string(),
        data: result,
    });
}
```

## Architecture decision

### Recommendation: moka with rkyv serialization

For ironstar's single-node deployment target with rebuildable cache:

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Cache layer | moka | Async-native, TTL built-in, no persistence overhead |
| Serialization | rkyv | Zero-copy deserialization for read-heavy pattern |
| Invalidation | Per-aggregate-type + time-window | Precise invalidation, minimal over-invalidation |
| SSE integration | Separate analytics_bus | Isolates analytics updates from domain events |

### Why not redb for cache?

redb is excellent for session storage (where persistence is valuable) but introduces unnecessary complexity for analytics cache:
- Synchronous API requires spawn_blocking wrappers
- Persistence is unnecessary (cache is rebuildable from DuckDB)
- Manual TTL implementation adds complexity
- Memory-mapped I/O is overkill when data fits in memory

### Hybrid approach for future consideration

If cache persistence becomes valuable (e.g., faster startup with warm cache), consider:

```rust
/// Layered cache: moka (hot) -> redb (warm) -> DuckDB (cold)
pub struct LayeredAnalyticsCache {
    hot: Cache<String, Vec<u8>>,          // moka (sub-ms access)
    warm: Option<Database>,                // redb (10ms access)
    cold: Arc<DuckDBService>,              // DuckDB (seconds access)
}

impl LayeredAnalyticsCache {
    pub async fn get(&self, key: &str) -> Result<Vec<u8>, Error> {
        // L1: moka (in-memory)
        if let Some(data) = self.hot.get(key).await {
            return Ok(data);
        }

        // L2: redb (persistent, optional)
        if let Some(ref db) = self.warm {
            if let Some(data) = self.get_from_redb(db, key)? {
                self.hot.insert(key.to_string(), data.clone()).await;
                return Ok(data);
            }
        }

        // L3: DuckDB (compute)
        let data = self.cold.execute_and_serialize(key).await?;
        self.hot.insert(key.to_string(), data.clone()).await;
        if let Some(ref db) = self.warm {
            self.write_to_redb(db, key, &data)?;
        }

        Ok(data)
    }
}
```

This layered approach adds the redb warm layer only when persistence justifies the added complexity.

## Implementation plan

### Phase 1: moka cache with TTL (recommended for v1)

```rust
// Cargo.toml
[dependencies]
moka = { version = "0.12", features = ["future"] }
rkyv = { version = "0.8", features = ["validation"] }
```

```rust
// src/infrastructure/analytics_cache.rs

use moka::future::Cache;
use rkyv::{Archive, Deserialize, Serialize};
use std::time::Duration;

/// Cached analytics result
#[derive(Archive, Deserialize, Serialize, Clone)]
pub struct CachedAnalytics {
    pub query_id: String,
    pub computed_at: i64,
    pub data: Vec<u8>,
}

/// Analytics cache service
pub struct AnalyticsCache {
    cache: Cache<String, Vec<u8>>,
    analytics: Arc<DuckDBService>,
}

impl AnalyticsCache {
    pub fn new(analytics: Arc<DuckDBService>) -> Self {
        let cache = Cache::builder()
            .max_capacity(1_000)
            .time_to_live(Duration::from_secs(300)) // 5 min default TTL
            .time_to_idle(Duration::from_secs(60))  // evict if unused for 1 min
            .build();

        Self { cache, analytics }
    }

    /// Get cached result or compute and cache
    pub async fn get_or_compute<T, F>(
        &self,
        key: &str,
        compute: F,
    ) -> Result<T, Error>
    where
        T: Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>,
        T::Archived: rkyv::Deserialize<T, rkyv::Infallible>,
        F: FnOnce() -> Result<T, Error>,
    {
        // Check cache
        if let Some(bytes) = self.cache.get(key).await {
            let archived = rkyv::check_archived_root::<T>(&bytes)
                .map_err(|_| Error::DeserializeFailed)?;
            return Ok(archived.deserialize(&mut rkyv::Infallible).unwrap());
        }

        // Compute (on blocking thread pool)
        let result = tokio::task::spawn_blocking(compute).await??;

        // Serialize and cache
        let bytes = rkyv::to_bytes::<_, 256>(&result)
            .map_err(|_| Error::SerializeFailed)?
            .to_vec();
        self.cache.insert(key.to_string(), bytes).await;

        Ok(result)
    }

    /// Invalidate entries matching predicate
    pub async fn invalidate_where<F>(&self, predicate: F)
    where
        F: Fn(&str) -> bool,
    {
        self.cache.invalidate_entries_if(move |key, _| predicate(key)).await;
    }
}
```

### Phase 2: Event-driven invalidation

Wire cache invalidation to the event broadcast channel.

### Phase 3: Proactive refresh (optional)

For frequently-accessed analytics, proactively refresh on events rather than invalidating.

## Tradeoff summary

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| moka only | Simple, async-native, TTL built-in | No persistence, memory-bound | **Use for v1** |
| redb only | Persistent, memory-mapped reads | Sync API, manual TTL, overkill | Use for session store |
| moka + redb | Fast + persistent warm layer | Added complexity | Consider for v2 if needed |
| SQLite table | Already in stack | Mixes cache with source-of-truth | Not recommended |
| sled | Lock-free | Unstable 1.0-alpha, space issues | Not recommended |

## References

- [moka documentation](https://docs.rs/moka/latest/moka/)
- [moka GitHub](https://github.com/moka-rs/moka)
- [rkyv benchmark](https://david.kolo.ski/blog/rkyv-is-faster-than/)
- [rust serialization benchmark](https://github.com/djkoloski/rust_serialization_benchmark)
- [redb design.md](/Users/crs58/projects/rust-workspace/redb/docs/design.md)
- [redb 1.0 release](https://www.redb.org/post/2023/06/16/1-0-stable-release/)
- [Arrow IPC in DuckDB](https://duckdb.org/2025/05/23/arrow-ipc-support-in-duckdb)
- [sled GitHub](https://github.com/spacejam/sled)

## Related documentation

- Event sourcing patterns: `event-sourcing-sse-pipeline.md`
- Architecture decisions: `architecture-decisions.md` (section 6: DuckDB)
- Session storage with redb: `architecture-decisions.md` (section 5: redb)
