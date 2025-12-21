# Projection patterns

This document covers projection caching strategies, DuckDB integration for analytics, and the projection trait pattern for ironstar.
Projections are read models derived from event streams, optimized for specific query patterns.

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

Use DuckDB materialized views for:
- **Analytics dashboards**: Aggregate queries over large event histories (e.g., "total sales per month").
- **Time-series analysis**: Window functions, moving averages.
- **Report generation**: Complex joins across multiple projections.

**Not** for:
- UI state (use in-memory projection).
- Session-specific data (use SQLite sessions table).
- Transactional commands (use event store).

## DuckDB async runtime integration

DuckDB-rs is a synchronous, blocking library.
All query methods block the calling thread until results are available.
In async axum handlers running on tokio, blocking calls must be carefully wrapped to avoid blocking the async runtime's worker threads, which would degrade performance for all concurrent requests.

### Integration strategies

**For quick queries** (expected to complete in milliseconds): Use `tokio::task::block_in_place()`.
This allows blocking operations within an async context without spawning a new OS thread.
The tokio runtime temporarily removes the worker thread from its pool while the blocking operation runs.

**For long-running analytics** (seconds or more): Use `tokio::task::spawn_blocking()`.
This spawns the blocking work on a dedicated thread pool, preventing it from tying up async worker threads.

### Code examples

```rust
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;
use tokio::task;

// Quick query pattern - block_in_place
async fn analytics_handler(
    State(analytics): State<Arc<AnalyticsService>>,
) -> Result<impl IntoResponse, AppError> {
    // Note: AppError is defined in event-sourcing-core.md
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

### Connection management

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
