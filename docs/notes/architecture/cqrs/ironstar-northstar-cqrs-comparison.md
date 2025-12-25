# CQRS/Event Sourcing comparison: Ironstar vs Northstar tracer bullet

This document provides a detailed comparison of the CQRS/Event Sourcing approaches between ironstar (the Rust+Datastar template) and the northstar tracer bullet specification (the Rust analytics CQRS reference implementation).
Both implementations target Rust + Datastar + DuckDB analytics applications but differ in their architectural assumptions, trade-offs, and scope.

## Executive summary

**Ironstar**: General-purpose CQRS template emphasizing single-node deployment, tokio::broadcast event bus with Zenoh migration path, and pure in-memory projections rebuilt on startup.

**Northstar tracer bullet**: Analytics-focused CQRS specification with QuerySessionAggregate for DuckDB query execution, Zenoh from the start, and 5-crate workspace structure.

**Key alignment**: Both use pure sync aggregates, SQLite event store with optimistic locking, SSE with Last-Event-ID reconnection, and UUID-tracked errors.

**Key divergence**: Ironstar starts with tokio::broadcast and migrates to Zenoh; northstar uses Zenoh immediately.
Ironstar uses 8-layer crate decomposition with HasXxx traits; northstar uses 5-crate workspace.
Ironstar emphasizes 4 critical invariants; northstar focuses on QuerySessionAggregate + DuckDB integration.

## 1. Aggregate design

### Signature and purity

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Pure sync aggregates** | `handle_command(state, cmd) -> Result<Vec<Event>, Error>` | `handle(&self, command) -> Result<Vec<Event>, Error>` | ✅ Aligned |
| **Async in aggregate** | Forbidden (Pure Aggregate Invariant) | Forbidden | ✅ Aligned |
| **I/O location** | Application layer before aggregate | Application layer (CommandHandler spawns DuckDB queries) | ✅ Aligned |
| **State management** | Immutable state passed to pure functions | Mutable `&self` with immutable internal state | ⚠️ Compatible (different styles) |

**Ironstar pattern:**
```rust
pub trait Aggregate: Default + Send + Sync {
    const NAME: &'static str;
    type State: Default + Clone + Send + Sync;
    type Command;
    type Event: Clone;
    type Error: Error;

    fn handle_command(state: &Self::State, cmd: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
    fn apply_event(state: Self::State, event: Self::Event) -> Self::State;
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

Both enforce pure synchronous aggregates with no side effects.
Ironstar uses immutable functional style (`state: &Self::State`), while northstar uses mutable methods (`&mut self`).
Both approaches are valid — ironstar's functional style aligns with esrs reference patterns, while northstar's mutable style is more conventional OOP.

**Divergence requiring resolution:**

None.
The functional vs OOP style is an implementation detail that doesn't affect correctness.
Ironstar can adopt northstar's mutable style for aggregate state management if preferred.

### Version tracking

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Version field** | Not explicit in Aggregate trait | `version(&self) -> u64` method | ❌ Divergence |
| **Optimistic locking** | Via EventStore with sequence numbers | Via version field in aggregate | ⚠️ Compatible |

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

Northstar explicitly tracks aggregate version for optimistic locking at the aggregate level.
Ironstar relies on SQLite AUTOINCREMENT sequence numbers for monotonic ordering.

**Resolution:**

Ironstar should add `version(&self) -> u64` to its Aggregate trait to match northstar.
The version field enables optimistic locking checks before appending events, preventing concurrent modification conflicts.

## 2. Event store schema

### Table structure

| Aspect | Ironstar | Northstar Tracer Bullet | Alignment |
|--------|----------|-------------------------|-----------|
| **Primary key** | `sequence INTEGER PRIMARY KEY AUTOINCREMENT` | `id UUID PRIMARY KEY` | ❌ Divergence |
| **Sequence numbering** | Global monotonic sequence | Per-aggregate sequence + version | ❌ Divergence |
| **Optimistic locking** | Via sequence check | Via version check | ⚠️ Compatible |
| **Event versioning** | Via upcasters (event schema evolution) | `event_version` column + metadata | ✅ Aligned |

**Ironstar schema (implicit from docs):**
```sql
CREATE TABLE events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,  -- Global monotonic
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSON NOT NULL,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
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
| **Async query execution** | Not specified for analytics | Spawn tokio task for DuckDB query | ⚠️ Compatible |

**Ironstar pattern:**
```rust
async fn handle_command<A: Aggregate>(
    store: &SqliteEventStore,
    bus: &broadcast::Sender<StoredEvent>,
    aggregate_id: &str,
    cmd: A::Command,
) -> Result<Vec<A::Event>, CommandError<A::Error>> {
    // 1. Load
    let events = store.load_events(A::NAME, aggregate_id).await?;
    let state = A::from_events(events);

    // 2. Decide (pure)
    let new_events = A::handle_command(&state, cmd)?;

    // 3. Persist
    for event in &new_events {
        store.append(A::NAME, aggregate_id, event).await?;
    }

    // 4. Publish
    for event in &new_events {
        let _ = bus.send(event.clone());
    }

    Ok(new_events)
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

Both follow the classic CQRS command handling pattern.
Northstar adds:
1. Explicit optimistic locking with version check
2. Background task spawning for long-running DuckDB queries
3. Zenoh publication instead of tokio::broadcast

**Divergence requiring resolution:**

**Event bus**: Ironstar uses tokio::broadcast with explicit migration path to Zenoh.
Northstar uses Zenoh from the start.

**Resolution**: Ironstar should adopt DualEventBus pattern during migration phase (see `distributed-event-bus-migration.md`), then fully migrate to Zenoh for production.

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
| **Initial implementation** | tokio::broadcast (single-node) | Zenoh (distribution-ready) | ❌ Divergence |
| **Migration path** | DualEventBus → full Zenoh | N/A (Zenoh from start) | ⚠️ Compatible |
| **Key expression filtering** | Zenoh after migration | Zenoh from start | ✅ Aligned |
| **Session-scoped routing** | `events/session/{id}/**` | `viz/{session}/events` | ⚠️ Compatible |

**Ironstar migration strategy:**

```
Phase 1: tokio::broadcast (single-node, simple)
    ↓
Phase 2: DualEventBus (both tokio::broadcast + Zenoh, migration in progress)
    ↓
Phase 3: Full Zenoh (distributed, key expression filtering)
```

**Northstar strategy:**

```
Production: Zenoh embedded from the start
```

**Assessment:**

**Philosophical divergence**: Ironstar prioritizes simplicity for single-node deployments, adding Zenoh when distribution is needed.
Northstar prioritizes distribution-readiness from the start.

**Trade-offs:**

| Approach | Pros | Cons |
|----------|------|------|
| **Ironstar (tokio → Zenoh)** | Simpler initial setup, fewer dependencies, faster local dev loop | Migration complexity when scaling |
| **Northstar (Zenoh from start)** | No migration needed, consistent architecture, ready for distribution | Slightly more complex initial setup |

**Resolution:**

**For general-purpose template (ironstar)**: Keep tokio::broadcast → Zenoh migration path.
Rationale: Most users deploy single-node apps; they benefit from simpler initial setup.

**For analytics-focused apps (northstar)**: Use Zenoh from the start.
Rationale: Analytics workloads often scale horizontally; pre-optimize for distribution.

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

1. **Pure sync aggregates**: Both forbid async/I/O in aggregates
2. **SQLite event store**: Both use SQLite for durable event storage
3. **Event versioning**: Both support schema evolution via event versioning
4. **SSE reconnection**: Both use event sequence for gap detection
5. **DuckDB integration**: Both use DuckDB for analytics queries
6. **Smart constructors**: Both use value objects with validation (DatasetRef, SqlQuery)
7. **thiserror error handling**: Both use consistent error patterns

### ⚠️ Compatible differences

1. **Aggregate style**: Ironstar functional (`&State`), northstar OOP (`&mut self`) — both valid
2. **Projection caching**: Ironstar uses ProjectionManager, northstar uses on-demand reconstitution — complementary patterns
3. **Event bus**: Ironstar migrates tokio::broadcast → Zenoh, northstar uses Zenoh from start — different philosophies but compatible
4. **Key expressions**: Different patterns (`events/session/{id}/**` vs `viz/{session}/events`) but both use Zenoh wildcards

### ❌ Critical divergences requiring resolution

1. **Event store schema**: Ironstar uses global sequence, northstar uses per-aggregate sequence
   - **Resolution**: Hybrid schema with both global_sequence (SSE) and aggregate_sequence (optimistic locking)

2. **UUID-tracked errors**: Northstar has explicit UUID + backtrace, ironstar does not
   - **Resolution**: Ironstar must adopt UUID-tracked error pattern

3. **Version tracking**: Northstar has explicit `version()` method, ironstar does not
   - **Resolution**: Ironstar should add `version(&self) -> u64` to Aggregate trait

## Recommendations for ironstar

### Must adopt from northstar

1. **Hybrid event store schema** combining global sequence (SSE) and aggregate sequence (optimistic locking)
2. **UUID-tracked errors** with backtrace for distributed error correlation
3. **Explicit version() method** in Aggregate trait for optimistic locking

### Should consider from northstar

1. **Zenoh from the start** for analytics-focused applications (keep tokio::broadcast → Zenoh migration for general template)
2. **Background task spawning pattern** for long-running DuckDB queries
3. **QuerySessionAggregate pattern** as reference implementation for analytics aggregates

### Ironstar strengths to preserve

1. **4 critical invariants** (Subscribe-Before-Replay, Pure Aggregate, Monotonic Sequence, Events-as-Truth) — excellent teaching framework
2. **8-layer crate architecture** with HasXxx traits — better modularity for complex applications
3. **Explicit migration path** from tokio::broadcast to Zenoh — lower barrier to entry for simple apps
4. **ProjectionManager pattern** — better for read-heavy workloads with complex UI state
5. **Analytics caching architecture** with moka + Zenoh invalidation — production-ready caching strategy

## Conclusion

Ironstar and northstar are highly aligned on fundamental CQRS/ES patterns: pure sync aggregates, SQLite event store, event versioning, and SSE reconnection.

The main divergences are:
1. **Event store schema** (global vs per-aggregate sequence) — resolved by hybrid approach
2. **Error tracking** (northstar has UUIDs, ironstar does not) — ironstar must adopt
3. **Event bus philosophy** (ironstar gradual migration, northstar Zenoh from start) — both valid for different use cases

By adopting the hybrid event store schema, UUID-tracked errors, and explicit version tracking from northstar, ironstar will gain production-ready CQRS capabilities while preserving its educational clarity and gradual complexity curve.

The northstar tracer bullet serves as an excellent reference implementation for analytics-specific patterns (QuerySessionAggregate, DuckDB async execution, background task spawning) that ironstar can adopt when implementing analytics features.
