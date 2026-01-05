# CQRS/Event Sourcing comparison: Ironstar vs Northstar tracer bullet

This document provides a detailed comparison of the CQRS/Event Sourcing approaches between ironstar (the Rust+Datastar template) and the northstar tracer bullet specification (the Rust analytics CQRS reference implementation).
Both implementations target Rust + Datastar + DuckDB analytics applications but differ in their architectural assumptions, trade-offs, and scope.

## Executive summary

**Ironstar**: General-purpose CQRS template using fmodel-rust's Decider pattern for event sourcing, emphasizing single-node deployment with Zenoh event bus (embedded mode) and pure in-memory projections rebuilt on startup.

**Northstar tracer bullet**: Analytics-focused CQRS specification with QuerySessionAggregate for DuckDB query execution, Zenoh from the start, and 5-crate workspace structure.

**Key alignment**: Both use pure sync aggregates, SQLite event store with optimistic locking, SSE with Last-Event-ID reconnection, and UUID-tracked errors.

**Key divergence**: Ironstar uses fmodel-rust's Decider pattern (pure function enforcement via type system) while northstar uses trait-based aggregates with mutable apply methods.
Ironstar uses 8-layer crate decomposition with HasXxx traits; northstar uses 5-crate workspace.
Ironstar emphasizes 4 critical invariants; northstar focuses on QuerySessionAggregate + DuckDB integration.

## 1. Aggregate design

### Signature and purity

| Aspect | Ironstar (fmodel-rust) | Northstar Tracer Bullet | Alignment |
|--------|------------------------|-------------------------|-----------|
| **Pure sync aggregates** | `Decider { decide: Fn(&C, &S) -> Result<Vec<E>, Error>, evolve: Fn(&S, &E) -> S }` | `handle(&self, command) -> Result<Vec<Event>, Error>` | ✅ Aligned (fmodel stricter) |
| **Async in aggregate** | Forbidden by type signature (no async) | Forbidden | ✅ Aligned |
| **I/O location** | EventRepository (infrastructure) | Application layer (CommandHandler spawns DuckDB queries) | ✅ Aligned |
| **State management** | Pure functions, no mutation | Mutable `&mut self.apply()` | ⚠️ fmodel enforces purity via types |
| **Purity enforcement** | Type system (Fn signature cannot contain async/I/O) | Convention + documentation | ✅ fmodel stronger |

**Ironstar pattern (fmodel-rust Decider):**
```rust
use fmodel_rust::decider::Decider;

pub fn todo_decider<'a>() -> Decider<'a, TodoCommand, Option<TodoState>, TodoEvent, TodoError> {
    Decider {
        decide: Box::new(|command, state| match command {
            TodoCommand::Create { id, text } => {
                if state.is_some() {
                    Ok(vec![TodoEvent::NotCreated {
                        id: id.clone(),
                        reason: "Todo already exists".into()
                    }])
                } else {
                    Ok(vec![TodoEvent::Created {
                        id: id.clone(),
                        text: text.clone(),
                        created_at: Utc::now()
                    }])
                }
            }
            // ... other commands
        }),

        evolve: Box::new(|state, event| match event {
            TodoEvent::Created { id, text, created_at } => Some(TodoState {
                id: id.clone(),
                text: text.clone(),
                created_at: *created_at,
                status: TodoStatus::Active,
                completed_at: None,
            }),
            TodoEvent::NotCreated { .. } => state.clone(),
            // ... other events
        }),

        initial_state: Box::new(|| None),
    }
}
```

**Northstar pattern:**
```rust
pub trait Aggregate: Default + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static {
    const TYPE: &'static str;
    type Command: Send;
    type Event: Clone + Serialize + for<'de> Deserialize<'de> + Send;
    type Error: StdError + Send;

    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
    fn apply(&mut self, event: &Self::Event);
    fn version(&self) -> u64;
    fn from_events(events: impl IntoIterator<Item = Self::Event>) -> Self;
}
```

**Assessment:**

Both enforce pure synchronous aggregates with no side effects, but fmodel-rust provides stronger guarantees.
Ironstar uses fmodel-rust's Decider pattern where pure functions are enforced by type signatures: `Fn(&C, &S) -> Result<Vec<E>, Error>` cannot contain async or I/O operations.
Northstar uses trait-based aggregates with mutable `apply(&mut self)` methods, relying on convention to prevent side effects.

fmodel-rust's approach directly implements Hoffman's Law 7 ("Work is a side effect") by making it type-impossible to perform I/O in decision logic.

**Key advantage of fmodel-rust:**

The `Decider` type signature guarantees purity at compile time.
You cannot accidentally introduce async I/O into decision logic because the Fn signature rejects it.
This is stronger than northstar's convention-based approach where `handle(&self)` could theoretically perform I/O if the developer violates the pattern.

### Version tracking

| Aspect | Ironstar (fmodel-rust) | Northstar Tracer Bullet | Alignment |
|--------|------------------------|-------------------------|-----------|
| **Version field** | Delegated to EventRepository::version_provider | `version(&self) -> u64` method in aggregate | ✅ Delegated to infrastructure |
| **Optimistic locking** | `previous_id` UNIQUE constraint in SQLite | Via version field passed to event_store | ✅ Compatible (different mechanisms) |

**Ironstar's version tracking (fmodel-rust EventRepository):**
```rust
impl<C, E> EventRepository<C, E, Uuid, EventStoreError> for SqliteEventRepository {
    async fn version_provider(&self, event: &E) -> Result<Option<Uuid>, EventStoreError> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT event_id FROM events
             WHERE decider_id = ? AND decider = ?
             ORDER BY offset DESC LIMIT 1"
        )
        .bind(event.identifier())
        .bind(event.decider_type())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|id| id.parse().map_err(EventStoreError::from)).transpose()
    }
}

// Optimistic locking via previous_id UNIQUE constraint
// INSERT INTO events (event_id, previous_id, ...) VALUES (?, ?, ...)
// Second concurrent writer violates UNIQUE(previous_id) → retry
```

**Northstar's version tracking:**
```rust
impl QuerySessionAggregate {
    pub version(&self) -> u64 {
        self.version
    }
}

// In CommandHandler
let current_version = aggregate.version();
self.event_store.append_events(
    QuerySessionAggregate::TYPE,
    &session_id.to_string(),
    current_version,  // Optimistic lock
    new_events.clone(),
).await?;
```

**Assessment:**

Both implementations provide optimistic locking but use different mechanisms.
Ironstar uses fmodel-rust's EventRepository trait where version tracking is delegated to the infrastructure layer via `version_provider()`.
The SQLite implementation uses a `previous_id` column with UNIQUE constraint: the first event has `previous_id = NULL`, subsequent events reference the previous event's UUID.
When two writers attempt to append concurrently, the second violates the UNIQUE constraint and must retry.

Northstar tracks version explicitly in the aggregate and passes it to the event store for validation.

**Key advantage of fmodel-rust's approach:**

Version tracking is an infrastructure concern, not a domain concern.
The Decider (pure domain logic) doesn't need to know about versioning.
The EventRepository handles optimistic locking at the database level via the `previous_id` mechanism.
This separation preserves the Decider's purity while still preventing concurrent modification conflicts.

See evaluation document lines 200-207 for detailed explanation of the `previous_id` optimistic locking mechanism.

## 2. Event store schema

### Table structure

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Primary key** | `sequence INTEGER PRIMARY KEY AUTOINCREMENT` | `id UUID PRIMARY KEY` | ❌ Divergence |
| **Sequence numbering** | Global monotonic sequence | Per-aggregate sequence + version | ❌ Divergence |
| **Optimistic locking** | Via sequence check | Via version check | ⚠️ Compatible |
| **Event versioning** | Via upcasters (event schema evolution) | `event_version` column + metadata | ✅ Aligned |

**Ironstar schema:**
```sql
CREATE TABLE events (
    global_sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    aggregate_sequence INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    event_version TEXT NOT NULL DEFAULT '1.0.0',
    payload TEXT NOT NULL,
    metadata TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    UNIQUE(aggregate_type, aggregate_id, aggregate_sequence)
) STRICT;
```

**Northstar schema:**
```sql
CREATE TABLE events (
    id UUID PRIMARY KEY,                      -- Event ID
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,                -- Per-aggregate sequence
    event_type TEXT NOT NULL,
    event_version TEXT NOT NULL,              -- Schema version
    payload JSON NOT NULL,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(aggregate_type, aggregate_id, sequence)  -- Optimistic locking
);
```

**Assessment:**

**Critical divergence**: Ironstar uses global sequence for SSE Last-Event-ID and monotonic ordering.
Northstar uses per-aggregate sequence with UNIQUE constraint for optimistic locking.

**Trade-offs:**

| Approach | Pros | Cons |
|----------|------|------|
| **Ironstar global sequence** | Simple SSE reconnection (`Last-Event-ID: 42`), total event ordering | Cannot enforce per-aggregate optimistic locking at DB level |
| **Northstar per-aggregate sequence** | DB-level optimistic locking via UNIQUE constraint, supports distributed event stores | SSE reconnection requires aggregate-aware logic |

**Resolution:**

**Hybrid schema combining both approaches:**
```sql
CREATE TABLE events (
    global_sequence INTEGER PRIMARY KEY AUTOINCREMENT,  -- For SSE Last-Event-ID
    id UUID NOT NULL UNIQUE,                             -- Event identity
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    aggregate_sequence INTEGER NOT NULL,                 -- Per-aggregate version
    event_type TEXT NOT NULL,
    event_version TEXT NOT NULL DEFAULT '1.0.0',
    payload JSON NOT NULL,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(aggregate_type, aggregate_id, aggregate_sequence)  -- Optimistic locking
);
```

This schema provides:
- Global sequence for SSE Last-Event-ID (ironstar requirement)
- Per-aggregate sequence for optimistic locking (northstar requirement)
- UUID event identity (northstar requirement)
- Event versioning for schema evolution (both requirements)

## 3. Command handling

### Load → decide → persist → publish flow

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Load pattern** | Load events, reconstitute aggregate | Load events, reconstitute aggregate | ✅ Aligned |
| **Decide (pure)** | `handle_command(state, cmd)` | `aggregate.handle(cmd)` | ✅ Aligned |
| **Persist** | Append to SQLite, broadcast to tokio channel | Append to SQLite with optimistic locking | ✅ Aligned |
| **Publish** | tokio::broadcast::send() | ZenohPublisher::publish_batch() | ❌ Divergence |
| **Async query execution** | async-duckdb Client/Pool with async API | async-duckdb Client/Pool with async API | ✅ Aligned |

**Ironstar pattern (fmodel-rust EventSourcedAggregate):**
```rust
use fmodel_rust::aggregate::EventSourcedAggregate;

pub async fn handle_todo_command(
    State(app_state): State<AppState>,
    Json(command): Json<TodoCommand>,
) -> impl IntoResponse {
    let aggregate = EventSourcedAggregate::new(
        app_state.event_repository.clone(),
        todo_decider(),
    );

    // EventSourcedAggregate handles load → decide → persist internally
    match aggregate.handle(&command).await {
        Ok(events) => {
            // Publish to Zenoh for SSE broadcast
            for (event, _version) in &events {
                app_state.zenoh.put(
                    format!("events/Todo/{}", event.identifier()),
                    serde_json::to_vec(event).unwrap(),
                ).await.ok();
            }
            (StatusCode::ACCEPTED, Json(events)).into_response()
        }
        Err(err) => {
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}
```

**Northstar pattern:**
```rust
pub async fn handle(
    &self,
    command: AnalyticsCommand,
) -> Result<Vec<AnalyticsEvent>, CommandError> {
    let session_id = command.session_id();

    // 1. Load
    let events = self.event_store
        .load_events(QuerySessionAggregate::TYPE, &session_id.to_string())
        .await?;

    // 2. Reconstitute + Decide
    let aggregate = QuerySessionAggregate::from_events(events);
    let current_version = aggregate.version();
    let new_events = aggregate.handle(command.clone())?;

    // 3. Persist with optimistic locking
    self.event_store.append_events(
        QuerySessionAggregate::TYPE,
        &session_id.to_string(),
        current_version,
        new_events.clone(),
    ).await?;

    // 4. Publish to Zenoh
    let topic = format!("viz/{}/events", session_id);
    self.publisher.publish_batch(&topic, &new_events).await?;

    // 5. Spawn background DuckDB query execution
    if let AnalyticsCommand::ExecuteQuery { query_id, sql, .. } = command {
        self.spawn_query_execution(session_id.to_string(), query_id, sql).await;
    }

    Ok(new_events)
}
```

**Assessment:**

Both follow the classic CQRS command handling pattern: load → decide → persist → publish.

Key differences:
1. **Ironstar (fmodel-rust)**: The `EventSourcedAggregate::handle()` method encapsulates the load → decide → persist flow, exposing only the command input and event output. The Axum handler only needs to publish events to Zenoh.
2. **Northstar**: The command handler explicitly orchestrates load → decide → persist → publish, giving more control but requiring more boilerplate.

Northstar adds:
1. Explicit optimistic locking with version check (ironstar handles this in EventRepository)
2. Background task spawning for long-running DuckDB queries (analytics-specific pattern)
3. Zenoh publication (both use Zenoh)

**Key advantage of fmodel-rust's EventSourcedAggregate:**

The application layer becomes simpler: receive command → call aggregate.handle() → publish events.
The load-decide-persist complexity is handled by the EventSourcedAggregate abstraction.
This reduces boilerplate while maintaining testability (the Decider itself is pure and easily tested).

**Divergence requiring resolution:**

**Event bus**: Both ironstar and northstar use Zenoh from the start.
Ironstar provides DualEventBus pattern for optional coexistence with existing tokio::broadcast codebases.

**Resolution**: No divergence — both use Zenoh. The DualEventBus pattern is available as an opt-in compatibility layer when integrating with legacy systems.

### DuckDB async execution

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Async strategy** | async-duckdb with thread-per-connection | async-duckdb with thread-per-connection | ✅ Aligned |
| **Connection pooling** | async-duckdb Pool (read-only) | async-duckdb Pool (read-only) | ✅ Aligned |
| **API pattern** | Closure-based `pool.conn(\|conn\| ...)` async API | Closure-based `pool.conn(\|conn\| ...)` async API | ✅ Aligned |

**Ironstar pattern:**
```rust
use async_duckdb::{Pool, PoolBuilder};

// Read-only pool for concurrent analytics
let pool = PoolBuilder::new()
    .path("analytics.duckdb")
    .num_conns(4)
    .open()
    .await?;

// Non-blocking query execution
let result = pool.conn(|conn| {
    conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
}).await?;
```

**Northstar pattern:**
```rust
use async_duckdb::{Pool, PoolBuilder};

// Read-only pool for concurrent analytics
let pool = PoolBuilder::new()
    .path("analytics.duckdb")
    .num_conns(4)
    .open()
    .await?;

// Non-blocking query execution
let result = pool.conn(|conn| {
    conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
}).await?;
```

**Assessment:**

Both implementations use async-duckdb for non-blocking DuckDB operations.
The crate provides an async wrapper around synchronous duckdb-rs using a background-thread-per-connection pattern.
The Pool type is specifically designed for read-only connections, ideal for analytics workloads.

**Resolution status: ✅ Aligned**

Ironstar has adopted async-duckdb with read-only Pool for analytics queries, matching the northstar pattern.

## 4. Projection patterns

### Rebuild strategy

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Startup rebuild** | Yes, from all events | Yes, from all events | ✅ Aligned |
| **Snapshots** | Future enhancement | Not mentioned (assumed no) | ✅ Aligned |
| **Incremental updates** | Subscribe to event bus | Subscribe to Zenoh topics | ✅ Aligned |
| **Projection trait** | Custom Projection trait | Not specified (implicit pattern) | ⚠️ Compatible |

**Ironstar ProjectionManager:**
```rust
pub struct ProjectionManager<P: Projection> {
    projection: P,
    state: Arc<RwLock<P::State>>,
    event_bus_rx: broadcast::Receiver<StoredEvent>,
}

impl<P: Projection> ProjectionManager<P> {
    pub async fn init(
        projection: P,
        event_store: &EventStore,
        event_bus: broadcast::Sender<StoredEvent>,
    ) -> Result<Self, Error> {
        // Rebuild from all events
        let events = event_store.query_all().await?;
        let state = projection.rebuild(events).await?;

        // Spawn background task for incremental updates
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                projection.apply(&mut *state, &event);
            }
        });

        Ok(manager)
    }
}
```

**Northstar QueryService (read model pattern):**
```rust
pub async fn get_session_state(
    &self,
    session_id: &str,
) -> Result<SessionStateViewModel, QueryError> {
    // Reconstitute from events (no persistent projection)
    let events = self.event_store
        .load_events(QuerySessionAggregate::TYPE, session_id)
        .await?;

    let aggregate = QuerySessionAggregate::from_events(events);

    // Project to view model
    Ok(SessionStateViewModel {
        session_id: session_id.to_string(),
        dataset_url: aggregate.dataset_url.map(|u| u.to_string()),
        active_query: aggregate.current_query.map(|q| ActiveQueryView { ... }),
        // ...
    })
}
```

**Assessment:**

Ironstar uses explicit ProjectionManager with background updates.
Northstar uses on-demand reconstitution in QueryService (no persistent projections).

Both are valid — ironstar optimizes for read-heavy workloads with cached projections, while northstar optimizes for simplicity with on-demand reconstitution.

**Divergence requiring resolution:**

None.
These are complementary patterns.
Ironstar can use northstar's on-demand pattern for lightweight read models, while maintaining ProjectionManager for complex UI state.

### Caching

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Analytics cache** | moka with TTL and eviction | Not specified | N/A |
| **Cache invalidation** | Zenoh subscriptions (Pattern 4) | Not specified | N/A |
| **DuckDB projections** | Via httpfs for remote datasets | DuckDB async query execution | ✅ Aligned |

**Assessment:**

Ironstar provides explicit analytics caching strategy with moka + Zenoh invalidation.
Northstar focuses on DuckDB query execution without caching layer.

**Resolution:**

Northstar can adopt ironstar's analytics caching patterns when query results need TTL-based caching.

## 5. Event bus architecture

### Design philosophy

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Initial implementation** | Zenoh (embedded mode, distribution-ready) | Zenoh (distribution-ready) | ✅ Aligned |
| **Optional fallback** | DualEventBus for tokio::broadcast coexistence | N/A (Zenoh from start) | ⚠️ Compatible |
| **Key expression filtering** | Zenoh from start | Zenoh from start | ✅ Aligned |
| **Session-scoped routing** | `events/session/{id}/**` | `viz/{session}/events` | ⚠️ Compatible |

**Ironstar architecture:**

```
Production: Zenoh embedded mode from the start
    │
    └─ Optional: DualEventBus pattern for coexistence with legacy tokio::broadcast systems
```

**Northstar architecture:**

```
Production: Zenoh embedded from the start
```

**Assessment:**

**Alignment**: Both ironstar and northstar use Zenoh from the start for event distribution.
Ironstar provides an optional DualEventBus pattern for compatibility with existing tokio::broadcast codebases during integration scenarios.

**Trade-offs:**

| Approach | Pros | Cons |
|----------|------|------|
| **Zenoh embedded (both)** | Distribution-ready, consistent architecture, key expression filtering, zero migration needed | Slightly more complex than simple tokio channels |
| **DualEventBus fallback (ironstar only)** | Allows gradual migration from tokio::broadcast in legacy systems | Additional complexity, only needed for integration scenarios |

**Resolution:**

**For general-purpose template (ironstar)**: Use Zenoh from the start, with DualEventBus available as an opt-in compatibility layer.
Rationale: Zenoh embedded mode is simple enough for single-node deployments while remaining distribution-ready.

**For analytics-focused apps (northstar)**: Use Zenoh from the start.
Rationale: Analytics workloads often scale horizontally; Zenoh provides consistent architecture from day one.

### Embedded vs networked Zenoh

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Embedded config** | Explicit 4-line config disabling networking | Implied embedded (not specified) | ✅ Aligned |
| **Network endpoints** | `[]` (empty) | Not specified | ⚠️ Compatible |
| **Scouting disabled** | Explicit | Not specified | ⚠️ Compatible |

**Ironstar embedded config:**
```rust
let mut config = Config::default();
config.insert_json5("listen/endpoints", "[]").unwrap();
config.insert_json5("connect/endpoints", "[]").unwrap();
config.insert_json5("scouting/multicast/enabled", "false").unwrap();
config.insert_json5("scouting/gossip/enabled", "false").unwrap();
```

**Assessment:**

Ironstar provides explicit embedded configuration.
Northstar should adopt this pattern for clarity.

## 6. SSE integration

### Last-Event-ID handling

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Last-Event-ID source** | Global sequence number | Not specified (implied event ID) | ❌ Divergence |
| **Reconnection pattern** | Subscribe before replay (invariant) | Not specified | ⚠️ Compatible |
| **Gap detection** | Replay events with `sequence > last_id` | Not specified | ⚠️ Compatible |

**Ironstar Subscribe-Before-Replay invariant:**
```rust
// CORRECT: Subscribe first, then replay
let mut rx = state.event_bus.subscribe();
let historical = state.event_store.load_since(last_id).await?;

// Stream historical + future events
```

**Assessment:**

Ironstar's global sequence enables simple SSE reconnection: `load_since(sequence > N)`.
Northstar's per-aggregate sequence requires aggregate-aware reconnection logic.

**Resolution:**

Adopt hybrid schema (see section 2) with global_sequence for SSE and aggregate_sequence for optimistic locking.

### SSE event format

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Event ID** | Sequence number | Not specified | ⚠️ Compatible |
| **Event data** | PatchElements (fat morph) | Not specified | ⚠️ Compatible |
| **Keep-alive** | 15 seconds (default) | Not specified | ⚠️ Compatible |

**Assessment:**

Both use Datastar's PatchElements for SSE payloads.
Ironstar provides explicit defaults; northstar should adopt them.

## 7. Error handling

### UUID tracking

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Error UUID** | Not specified in core docs | `id: Uuid` field in AnalyticsError | ❌ Divergence |
| **Backtrace** | Not specified | `backtrace: Backtrace` field | ❌ Divergence |
| **Error correlation** | Not specified | UUID for distributed tracing | ❌ Divergence |

**Northstar error pattern:**
```rust
pub struct AnalyticsError {
    id: Uuid,           // For log correlation
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl AnalyticsError {
    pub fn query_not_found(query_id: QueryId) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: ErrorKind::QueryNotFound { query_id },
            backtrace: Backtrace::capture(),
        }
    }

    pub fn error_id(&self) -> Uuid {
        self.id
    }
}
```

**Assessment:**

Northstar provides explicit error tracking with UUIDs and backtraces.
This is an essential pattern for production CQRS systems where errors may occur asynchronously (in background tasks, projections, etc.).

**Divergence requiring resolution:**

**Critical**: Ironstar must adopt UUID-tracked errors.

**Resolution:**

Add to ironstar's error types:
```rust
use std::backtrace::Backtrace;
use uuid::Uuid;

pub struct DomainError {
    id: Uuid,
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl DomainError {
    pub fn error_id(&self) -> Uuid {
        self.id
    }
}
```

### Error propagation

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **thiserror** | Used for error derives | Used for error derives | ✅ Aligned |
| **Domain vs infrastructure** | Separate error types | AnalyticsError wraps domain + infra | ⚠️ Compatible |
| **Error conversion** | `#[from]` attributes | `#[from]` attributes | ✅ Aligned |

**Assessment:**

Both use thiserror for ergonomic error handling.
Compatible approaches.

## Summary of alignments and divergences

### ✅ Strong alignments

1. **Pure sync aggregates**: Both forbid async/I/O in aggregates (fmodel-rust enforces via type system)
2. **SQLite event store**: Both use SQLite for durable event storage
3. **Event versioning**: Both support schema evolution via event versioning
4. **SSE reconnection**: Both use event sequence for gap detection
5. **DuckDB integration**: Both use DuckDB for analytics queries
6. **Smart constructors**: Both use value objects with validation (DatasetRef, SqlQuery)
7. **thiserror error handling**: Both use consistent error patterns
8. **Optimistic locking**: Both prevent concurrent modifications (ironstar via `previous_id` UNIQUE, northstar via version field)

### ⚠️ Compatible differences

1. **Aggregate abstraction**: Ironstar uses fmodel-rust Decider (pure functions), northstar uses trait with `&mut self.apply()` — fmodel enforces stronger purity guarantees
2. **Version tracking**: Ironstar delegates to EventRepository::version_provider, northstar tracks in aggregate state — both provide optimistic locking
3. **Command handling**: Ironstar uses EventSourcedAggregate (encapsulates load/decide/persist), northstar uses explicit orchestration — different ergonomics
4. **Projection caching**: Ironstar uses ProjectionManager, northstar uses on-demand reconstitution — complementary patterns
5. **Event bus**: Both use Zenoh from the start — ironstar provides optional DualEventBus for tokio::broadcast coexistence
6. **Key expressions**: Different patterns (`events/session/{id}/**` vs `viz/{session}/events`) but both use Zenoh wildcards

### ❌ Critical divergences requiring resolution

1. **Event store schema**: Ironstar uses fmodel-rust schema with `previous_id` for optimistic locking, northstar uses per-aggregate sequence
   - **Resolution**: Ironstar's fmodel-rust schema (from evaluation doc lines 136-198) provides both global `offset` (for SSE) and `previous_id` UNIQUE constraint (for optimistic locking). Northstar can adopt this pattern.

2. **UUID-tracked errors**: Northstar has explicit UUID + backtrace, ironstar does not yet (but compatible with fmodel-rust)
   - **Resolution**: Ironstar must adopt UUID-tracked error pattern (not a fmodel-rust concern, orthogonal)

3. **async-duckdb vs spawn_blocking**: Northstar uses async-duckdb for non-blocking DuckDB operations with connection pooling
   - **Resolution status: ✅ Resolved** — Ironstar has adopted async-duckdb with read-only Pool for analytics queries

## Recommendations for ironstar

### Must adopt from northstar

1. **✅ fmodel-rust Decider pattern** for event sourcing (adopted) — stronger purity guarantees via type system
2. **✅ fmodel-rust EventRepository** with SQLite implementation (adopted) — `previous_id` UNIQUE constraint for optimistic locking
3. **UUID-tracked errors** with backtrace for distributed error correlation
4. **✅ async-duckdb for DuckDB analytics integration** with read-only connection pooling (adopted)

### Should consider from northstar

1. **Background task spawning pattern** for long-running DuckDB queries
2. **QuerySessionAggregate pattern** as reference implementation for analytics aggregates

### Ironstar strengths to preserve and extend

1. **fmodel-rust Decider pattern** — type-enforced purity, algebraic composition via `combine()`, built-in TestSpecification DSL
2. **fmodel-rust EventRepository abstraction** — separates domain (Decider) from infrastructure (event persistence), enables `previous_id` optimistic locking
3. **4 critical invariants** (Subscribe-Before-Replay, Pure Aggregate, Monotonic Sequence, Events-as-Truth) — excellent teaching framework, now stronger with fmodel-rust
4. **8-layer crate architecture** with HasXxx traits — better modularity for complex applications
5. **Zenoh-first with optional DualEventBus** — distribution-ready from day one with compatibility layer for legacy systems
6. **ProjectionManager pattern** — better for read-heavy workloads with complex UI state
7. **Analytics caching architecture** with moka + Zenoh invalidation — production-ready caching strategy

### New capabilities from fmodel-rust adoption

1. **Saga/process managers** — built-in `Saga<Event, Command>` for event-driven choreography across aggregates
2. **Composition primitives** — `combine()` for multi-aggregate workflows, `merge()` for shared event types
3. **Given/When/Then testing** — `DeciderTestSpecification` for property-based aggregate testing
4. **Separation of concerns** — Decider (pure domain), EventSourcedAggregate (application), EventRepository (infrastructure)

## Conclusion

Ironstar and northstar are highly aligned on fundamental CQRS/ES patterns: pure sync aggregates, SQLite event store, event versioning, and SSE reconnection.

Ironstar's adoption of fmodel-rust strengthens its architectural foundation:
1. **Purity enforcement**: The Decider type signature (`Fn(&C, &S) -> Result<Vec<E>, Error>`) makes it impossible to accidentally introduce async/I/O in domain logic — stronger than northstar's convention-based approach
2. **Version tracking**: The EventRepository's `version_provider()` method and `previous_id` UNIQUE constraint delegate optimistic locking to infrastructure, keeping the Decider pure
3. **Composition**: fmodel-rust's `combine()` and `Saga` enable multi-aggregate workflows without custom event bus wiring
4. **Testing**: The built-in `DeciderTestSpecification` provides given/when/then testing that ironstar's architecture documents already specify

The main remaining divergence is **UUID-tracked errors** — ironstar must adopt northstar's pattern of embedding UUID + backtrace in error types for production observability.
This is orthogonal to fmodel-rust and applies to both approaches.

**Key insight**: Ironstar's fmodel-rust adoption positions it as a more theoretically grounded template than northstar.
The Decider pattern is the minimal algebraic interface for functional event sourcing (from Jérémie Chassaing's work), directly implementing the category-theoretic foundations in ironstar's preference documents.
Northstar remains an excellent reference implementation for analytics-specific patterns (QuerySessionAggregate, DuckDB query execution, background task spawning).

Both templates serve complementary purposes:
- **Ironstar**: General-purpose template with stronger functional purity via fmodel-rust, 8-layer crate architecture, and distribution-ready Zenoh from day one
- **Northstar**: Analytics-focused reference with QuerySessionAggregate pattern and DuckDB integration best practices

The northstar tracer bullet serves as an excellent reference implementation for analytics-specific patterns that ironstar can adopt when implementing analytics features.
