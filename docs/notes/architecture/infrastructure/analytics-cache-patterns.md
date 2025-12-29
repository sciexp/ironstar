# Analytics cache patterns

This document provides implementation patterns for the caching strategy defined in `analytics-cache-architecture.md`.

## Cache invalidation patterns

These patterns form a coherent progression from simple to sophisticated, culminating in Zenoh-based invalidation that integrates with the event bus.

### Pattern 1: Time-based TTL

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

### Pattern 2: Event-triggered invalidation

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

### Pattern 3: Aggregate-scoped proactive refresh

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

### Pattern 4: Zenoh-based cache invalidation

When using Zenoh as the event bus, cache invalidation leverages key expression filtering for precise, efficient invalidation with distribution-ready semantics.

See `zenoh-event-bus.md` "Key expression design" for Zenoh wildcard patterns (`*`, `**`, `$*`) and subscription semantics.

#### Cache dependency mapping

Each cache entry declares its dependencies as Zenoh key expression patterns.
When an event matching any dependency pattern is published, the cache entry is invalidated.

```rust
/// Describes what events a cached query depends on
#[derive(Clone, Debug)]
pub struct CacheDependency {
    pub cache_key: String,
    pub depends_on: Vec<String>,
}

impl CacheDependency {
    pub fn new(cache_key: impl Into<String>) -> Self {
        Self {
            cache_key: cache_key.into(),
            depends_on: Vec::new(),
        }
    }

    pub fn depends_on_aggregate(mut self, aggregate_type: &str) -> Self {
        self.depends_on.push(format!("events/{}/**", aggregate_type));
        self
    }

    pub fn depends_on_instance(mut self, aggregate_type: &str, id: &str) -> Self {
        self.depends_on.push(format!("events/{}/{}/*", aggregate_type, id));
        self
    }
}

// Example: daily stats cache depends on all Todo events
let daily_stats_dep = CacheDependency::new("daily_stats:2024-01-15")
    .depends_on_aggregate("Todo");
```

#### Cache invalidation service

```rust
use moka::future::Cache;
use std::sync::Arc;
use tokio::sync::RwLock;
use zenoh::Session;

pub struct ZenohCacheInvalidator {
    cache: Cache<String, Vec<u8>>,
    zenoh: Arc<Session>,
    dependencies: Arc<RwLock<Vec<CacheDependency>>>,
}

impl ZenohCacheInvalidator {
    pub async fn spawn(
        cache: Cache<String, Vec<u8>>,
        zenoh: Arc<Session>,
        dependencies: Vec<CacheDependency>,
    ) -> Result<Self, Error> {
        let invalidator = Self {
            cache,
            zenoh,
            dependencies: Arc::new(RwLock::new(dependencies)),
        };

        // Subscribe to all events and filter locally
        let subscriber = invalidator.zenoh
            .declare_subscriber("events/**")
            .await?;

        let cache = invalidator.cache.clone();
        let dependencies = invalidator.dependencies.clone();

        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                let key_expr = sample.key_expr().as_str();
                let deps = dependencies.read().await;

                for dep in deps.iter() {
                    if dep.depends_on.iter().any(|pattern| {
                        matches_key_expression(pattern, key_expr)
                    }) {
                        cache.invalidate(&dep.cache_key).await;
                    }
                }
            }
        });

        Ok(invalidator)
    }
}

fn matches_key_expression(pattern: &str, key: &str) -> bool {
    if pattern.ends_with("/**") {
        let prefix = &pattern[..pattern.len() - 3];
        key.starts_with(prefix)
    } else if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 2];
        key.starts_with(prefix) && !key[prefix.len()..].contains('/')
    } else {
        pattern == key
    }
}
```

### DuckLake catalog pattern

DuckLake catalogs are small SQLite metadata databases (~5MB) that map logical tables to parquet data files.
Once attached, query tables directly:

```sql
ATTACH 'ducklake:lakes/frozen/space.db' AS space;
SELECT * FROM space.main.astronauts ORDER BY total_space_days DESC LIMIT 10;
```

The DuckLake extension handles catalog→parquet mapping transparently via httpfs.

#### Embedded vs runtime catalogs

**Embedded catalogs** (recommended for known datasets):

Embed DuckLake catalogs in the binary via `rust_embed` to eliminate ATTACH latency:

```rust
#[derive(RustEmbed)]
#[folder = "assets/ducklake-catalogs/"]
struct DuckLakeCatalogs;

// Extract to temp file at startup, ATTACH locally (~0ms)
let catalog = DuckLakeCatalogs::get("space.db").unwrap();
let temp_path = std::env::temp_dir().join("ironstar-space.db");
std::fs::write(&temp_path, catalog.data)?;
conn.execute_batch(&format!("ATTACH 'ducklake:{}' AS space;", temp_path.display()))?;
```

Cache keys use build-time versioning (no runtime snapshot query needed):

```rust
const CATALOG_VERSION: &str = env!("CARGO_PKG_VERSION");
let cache_key = format!("embedded:space:{}:{}:{:x}",
    CATALOG_VERSION, table_name, query_hash);
```

Cache invalidation is automatic: new binary deploy = new cache keys.

**Runtime catalogs** (for user-provided datasets):

Attach from HuggingFace at runtime (~2s download penalty):

```sql
ATTACH 'ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db' AS space;
```

Cache keys require runtime snapshot query:

```rust
let snapshot_id: u64 = conn.query_row(
    "SELECT id FROM ducklake_current_snapshot('space')",
    [], |row| row.get(0)
)?;
let cache_key = format!("remote:sciexp/fixtures:{}:{}:{:x}",
    snapshot_id, table_name, query_hash);
```

#### Data fetching

Regardless of catalog source, parquet data is fetched on-demand via httpfs (~100-500ms per query).
This is where moka caching provides value — caching query results avoids repeated parquet fetches.

Canonical test dataset: `hf://datasets/sciexp/fixtures/lakes/frozen/space.db`

#### Performance considerations

| Aspect | broadcast | Zenoh |
|--------|-----------|-------|
| Latency (p50) | ~10μs | ~100μs |
| Filtering | Client-side | Server-side |
| Distribution-ready | No | Yes |

The latency difference is negligible compared to DuckDB query time (1-10ms).
Use Zenoh for architectural consistency with the event bus.

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
    duckdb_pool: Arc<async_duckdb::Pool>,
}

impl AnalyticsCache {
    pub fn new(duckdb_pool: Arc<async_duckdb::Pool>) -> Self {
        let cache = Cache::builder()
            .max_capacity(1_000)
            .time_to_live(Duration::from_secs(300)) // 5 min default TTL
            .time_to_idle(Duration::from_secs(60))  // evict if unused for 1 min
            .build();

        Self { cache, duckdb_pool }
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
        F: FnOnce(&duckdb::Connection) -> Result<T, duckdb::Error> + Send + 'static,
    {
        // Check cache
        if let Some(bytes) = self.cache.get(key).await {
            let archived = rkyv::check_archived_root::<T>(&bytes)
                .map_err(|_| Error::DeserializeFailed)?;
            return Ok(archived.deserialize(&mut rkyv::Infallible).unwrap());
        }

        // Compute via async-duckdb pool
        let result = self.duckdb_pool.conn(compute).await?;

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

## Related documentation

- Cache architecture decision: `analytics-cache-architecture.md`
- Event sourcing core concepts: `../cqrs/event-sourcing-core.md`
- Projection patterns: `../cqrs/projection-patterns.md`
- Zenoh event bus integration: `zenoh-event-bus.md`
- Architecture decisions: `../core/architecture-decisions.md` (section 6: DuckDB)
