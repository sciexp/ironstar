# Projection patterns

This document covers projection caching strategies, DuckDB integration for analytics, and the projection trait pattern for ironstar.
Projections are read models derived from event streams, optimized for specific query patterns.

## Hoffman's projection Laws

Projections in ironstar adhere to Kevin Hoffman's Laws governing read model behavior (see `~/.claude/commands/preferences/event-sourcing.md` for complete definitions):

| Law | Constraint | Ironstar Implementation |
|-----|-----------|------------------------|
| **Law 3**: All projection data from events | Projections cannot pull data from external sources or wall clocks | `rebuild()` and `apply()` methods receive only event data |
| **Law 4**: Work is a side effect | Projections must not perform I/O during event processing | Projection trait methods are pure transformations |
| **Law 5**: All projections stem from events | Every projection value derives from at least one event | No projection state exists without corresponding events |
| **Law 9**: Projectors cannot share | Each projector owns its state exclusively | Separate ProjectionManager instances with isolated RwLock |

These Laws ensure projections are disposable (can be rebuilt from events), testable (pure functions), and replay-safe (deterministic).
The moka cache + Zenoh invalidation pattern (see "Cache-aside with moka" below) maintains these guarantees while optimizing for read latency.

## Projection caching strategy

**Decision: Pure in-memory projections with snapshot recovery on startup, no persistent projection state.**

### Rationale

**Pure in-memory**: Recompute from events on startup.
- **Pros**: Simple, no cache invalidation, always consistent with event store.
- **Cons**: Slow startup if many events, holds memory.

**Persisted snapshots**: Store projection state periodically.
- **Pros**: Fast startup.
- **Cons**: Cache invalidation complexity, snapshot versioning, requires migration on projection schema changes.

**DuckDB materialized views**: Use DuckDB for analytical projections.
- **Pros**: Excellent for OLAP queries, automatic incremental updates.
- **Cons**: Overkill for simple UI projections, adds dependency complexity.

### Implementation pattern

> **Semantic foundation**: Projections implement a Galois connection with the event log.
> The `apply` method is the catamorphism algebra; `rebuild` is the unique fold from initiality.
> See [semantic-model.md § Catamorphism](../core/semantic-model.md#state-reconstruction-as-catamorphism) and [§ Galois connection](../core/semantic-model.md#projections-as-abstraction-concretion-pairs).

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
// Note: Error and StoredEvent types are defined in event-sourcing-core.md

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

### When to use DuckDB

Use DuckDB for querying external scientific datasets (HuggingFace Hub, S3, DuckLake via httpfs):
- **Analytics dashboards**: Query remote parquet datasets (e.g., climate data, financial data from HuggingFace Hub).
- **Time-series analysis**: DuckDB's window functions over columnar external data.
- **Report generation**: Complex queries across remote datasets without local data ingestion.

**Not** for:
- UI state (use in-memory projection).
- Session-specific data (use SQLite sessions table).
- Transactional commands (use event store).

### Analytics projections: materialization vs in-memory

The in-memory projection pattern described above works well for small state (todo lists, session state, UI widgets).
For analytics projections that produce large result sets (thousands of rows from DuckDB queries), consider alternative strategies:

**Materialized read tables**: Write DuckDB query results to a read-optimized table rather than holding in memory.
- Suitable for precomputed dashboards, reports, or aggregations that change infrequently
- Supports pagination and filtering at the database level
- Trade-off: materialization latency vs memory consumption

**Streaming results**: Stream rows incrementally via SSE rather than buffering entire result set.
- Suitable for real-time analytics feeds or large exports
- DuckDB cursor iteration with chunked SSE transmission
- Trade-off: client-side buffering complexity vs server memory pressure

**Cache-aside with moka**: Cache frequently-accessed query results in moka with TTL-based eviction.
- Suitable for analytics endpoints with high read-to-write ratios
- Zenoh key expression subscriptions trigger cache invalidation on relevant events
- See `../infrastructure/analytics-cache-architecture.md` for detailed cache invalidation patterns (Pattern 4)
- Trade-off: stale data risk (bounded by TTL) vs query latency

Choose based on query frequency, result set size, and staleness tolerance.
In-memory projections remain optimal for small, session-scoped state.

## DuckDB async runtime integration

DuckDB-rs is a synchronous, blocking library, but async-duckdb provides a clean async interface via connection pooling and dedicated background threads.
This eliminates the need for manual `spawn_blocking` or `block_in_place` wrappers in async axum handlers.

### async-duckdb Pool integration

async-duckdb provides a Pool type that manages multiple read-only connections, each running on its own dedicated background thread.
The `.conn()` method bridges the sync DuckDB API to the async world using a closure-based pattern.

### Code examples

```rust
use axum::{extract::State, response::IntoResponse, Json};
use async_duckdb::Pool;
use std::sync::Arc;

// Analytics handler with async-duckdb
async fn analytics_handler(
    State(pool): State<Arc<Pool>>,
) -> Result<impl IntoResponse, AppError> {
    // Note: AppError is defined in event-sourcing-core.md

    // Non-blocking query execution via pool
    let result = pool.conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT aggregate_type, COUNT(*) as count
             FROM events
             GROUP BY aggregate_type"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AggregateCount {
                aggregate_type: row.get(0)?,
                count: row.get(1)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
    }).await?;

    Ok(Json(result))
}

// Long-running analytics query - same pattern
async fn heavy_report_handler(
    State(pool): State<Arc<Pool>>,
) -> Result<impl IntoResponse, AppError> {
    // No spawn_blocking needed - pool handles threading automatically
    let result = pool.conn(|conn| {
        // Complex multi-step analytics
        generate_monthly_report(conn)
    }).await?;

    Ok(Json(result))
}
```

### Pool initialization

```rust
use async_duckdb::{Pool, PoolBuilder};

// Create read-only pool for concurrent analytics
let pool = PoolBuilder::new()
    .path("analytics.duckdb")
    .num_conns(4)  // Number of concurrent read connections
    .open()
    .await?;

// Use in axum application state
let app_state = AppState {
    duckdb_pool: Arc::new(pool),
    // ... other state
};
```

### Key design points

- Pool is read-only by default (per DuckDB single-writer concurrency model)
- Each connection runs on a dedicated background thread
- The `.conn()` closure bridges sync DuckDB API to async world
- Runtime-agnostic (works with tokio, async-std, etc.)
- No manual `spawn_blocking` or `block_in_place` required

For ironstar analytics projections, async-duckdb Pool provides clean async integration without manual threading concerns.

## Projection trait code pattern

The Projection trait enables polymorphic read model implementations with consistent lifecycle management.

```rust
use async_trait::async_trait;
use axum::response::sse::Event;

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
```

See the "Implementation pattern" section above for the complete ProjectionManager implementation.

## Related documentation

- `event-sourcing-core.md`: Master index and architecture overview
- `sse-connection-lifecycle.md`: SSE connection phases and debugging
- `event-replay-consistency.md`: Event replay and consistency boundaries
- `performance-tuning.md`: Performance optimization strategies
- `command-write-patterns.md`: Command handlers and write path
- `../infrastructure/analytics-cache-architecture.md`: DuckDB cache invalidation via Zenoh
