# Event sourcing patterns for ironstar

This is the master index document for ironstar's event sourcing, projection, and SSE pipeline architecture.
This document provides the conceptual overview, architectural diagram, defaults, trade-offs, and references.
Detailed implementation patterns are split into focused documents linked below.

## Critical invariants

Ironstar's event sourcing implementation depends on four fundamental invariants that must never be violated.
Understanding these invariants is essential before implementing any CQRS component.

These invariants derive from Kevin Hoffman's "Ten Laws of Event Sourcing" (see `~/.claude/commands/preferences/event-sourcing.md` for complete Law definitions):
- Subscribe-before-replay relates to event ordering semantics
- Pure Decider invariant embodies **Law 7** (work is a side effect)
- Monotonic sequence invariant supports **Law 1** (events are immutable)
- Events-as-source-of-truth embodies **Laws 3 and 5** (all projection data from events)
- Failure events preserve state relates to **Law 6** (failures are events)

### Law 6: Failure events preserve previous state

When an aggregate rejects a command due to validation failure or business rule violation, the rejection should be captured as a **failure event** that does NOT modify aggregate state.

**Why this matters:**
- Audit trail of attempted but rejected operations
- Retry semantics with idempotency guarantees
- Analytics on failure patterns and error rates
- Debugging complex workflows

**Implementation pattern:**

```rust
// Failure event variant in QuerySessionEvent
QueryFailed {
    query_id: QueryId,
    error: QueryError,
    failed_at: DateTime<Utc>,
}

// apply_event for failure event returns state unchanged
fn apply_event(state: QuerySessionState, event: QuerySessionEvent) -> QuerySessionState {
    match event {
        QuerySessionEvent::QueryFailed { .. } => {
            // State machine transitions to Failed status
            // but core data (dataset_ref, sql, etc.) preserved
            QuerySessionState {
                status: QuerySessionStatus::Failed { ... },
                ..state  // Preserve other fields
            }
        }
        // ... other variants
    }
}
```

**Failure vs. Error distinction:**
- **Failure event**: Business rule violation captured in event stream (e.g., `QueryFailed`)
- **Command error**: Validation rejection before event emission (e.g., `Err(QuerySessionError::QueryAlreadyInProgress)`)

Both are valid—failure events capture rejections that should be audited; command errors handle precondition violations that don't need persistence.

### Subscribe-before-replay invariant

SSE handlers must subscribe to the event bus *before* loading historical events from the event store.
This prevents a race condition where events could be missed during the gap between replay completion and subscription.

```rust
// CORRECT: Subscribe first, then replay
let mut rx = state.event_bus.subscribe();
let historical = state.event_store.load_since(last_id).await?;

// INCORRECT: Replay first, then subscribe (creates race window)
let historical = state.event_store.load_since(last_id).await?;
let mut rx = state.event_bus.subscribe(); // Events emitted during replay are lost
```

The correct ordering ensures that even if new events arrive during historical replay, they are buffered in the broadcast channel and will be processed after replay completes.
See `sse-connection-lifecycle.md` for the complete connection state machine.

### Pure Decider invariant

Decider `decide` and `evolve` functions must be pure: synchronous, deterministic, with no side effects.
All I/O operations (database queries, API calls, random number generation, system time) must occur in the application layer before calling the Decider.

ironstar uses fmodel-rust's Decider pattern, which enforces purity via type signatures:

```rust
// CORRECT: Pure Decider with pre-validated inputs
pub fn query_session_decider<'a>() -> Decider<'a, QuerySessionCommand, QuerySessionState, QueryEvent, QueryError> {
    Decider {
        decide: Box::new(|command, state| {
            // No async, no I/O, deterministic
            match command {
                QuerySessionCommand::StartQuery { query_id, sql } => {
                    if state.status != SessionStatus::Idle {
                        return Ok(vec![QueryEvent::NotStarted {
                            reason: "Invalid state transition".into()
                        }]);
                    }
                    Ok(vec![QueryEvent::QueryStarted {
                        query_id: query_id.clone(),
                        sql: sql.clone(),
                        started_at: Utc::now()
                    }])
                }
                // ...
            }
        }),
        evolve: Box::new(|state, event| {
            // Pure state transition
            match event {
                QueryEvent::QueryStarted { .. } => QuerySessionState {
                    status: SessionStatus::Running,
                    ..state.clone()
                },
                // ...
            }
        }),
        initial_state: Box::new(|| QuerySessionState::default()),
    }
}

// INCORRECT: Attempting async in Decider (compile error)
pub fn bad_decider() -> Decider<...> {
    Decider {
        decide: Box::new(|command, state| async {  // ❌ Type error: expects sync Fn
            let results = execute_duckdb_query(&command.sql).await?;
            // ...
        }),
        // ...
    }
}
```

The Decider type signature enforces purity by construction:
- `decide: Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + Send + Sync>` — no async, no mutable state
- `evolve: Box<dyn Fn(&S, &E) -> S + Send + Sync>` — pure state transition

This purity enables testing Deciders without infrastructure, replaying events to reconstruct state, and reasoning about domain logic independently of I/O concerns.
See `../core/design-principles.md` for the complete pure Decider pattern and `command-write-patterns.md` for testing strategies with DeciderTestSpecification.

### Monotonic sequence invariant

Event sequence numbers must be strictly monotonically increasing with no gaps.
The events table uses dual sequence tracking: `global_sequence` for SSE reconnection and `aggregate_sequence` for optimistic locking.

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

The dual sequence approach serves two purposes:
- `global_sequence`: INTEGER PRIMARY KEY AUTOINCREMENT for SSE Last-Event-ID tracking (enables simple `WHERE global_sequence > ?` queries for reconnection)
- `aggregate_sequence`: Per-aggregate version for optimistic locking (UNIQUE constraint prevents concurrent modifications to the same aggregate)

When a client reconnects with `Last-Event-ID: 42`, the server replays all events with `global_sequence > 42`, guaranteeing no missed updates.
The `aggregate_sequence` prevents lost updates when two commands target the same aggregate concurrently.
See `event-replay-consistency.md` for reconnection patterns and `command-write-patterns.md` for optimistic locking implementation.

### Events-as-source-of-truth invariant

The event store is the authoritative source of truth; all other state (projections, caches, read models) is derived and ephemeral.
Projections can be deleted and rebuilt from events at any time without data loss.

```rust
// On startup: rebuild projections from scratch
let all_events = event_store.load_all().await?;
let projection = QueryResultProjection::from_events(all_events);

// Projections are never persisted; they're always derived
// For analytics workloads, projection results are cached in moka with TTL-based eviction
```

This enables:
- Schema evolution by replaying events through new projection logic
- Debugging by replaying events to specific points in time
- Audit trails with complete state reconstruction

Never mutate projections directly; always derive them from events.
See `projection-patterns.md` for caching strategies that preserve this invariant.

## Document cluster navigation

| Document | Focus |
|----------|-------|
| `event-sourcing-core.md` | Master index, architecture diagram, defaults, trade-offs (this document) |
| `sse-connection-lifecycle.md` | SSE connection state machine, subscription phases, debugging |
| `event-replay-consistency.md` | Event replay strategy, reconnection patterns, consistency guarantees |
| `projection-patterns.md` | Projection caching, DuckDB integration, projection trait |
| `performance-tuning.md` | Channel sizing, multiple projections, observability metrics |
| `performance-advanced-patterns.md` | Debouncing, batching, rate limiting, backpressure |
| `command-write-patterns.md` | Command handlers, aggregate patterns, testing DSL, event store |

For Zenoh event bus integration, see `../infrastructure/zenoh-event-bus.md`.

## Architecture diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         Browser (Datastar)                                │
│  ┌────────────────────┐                    ┌──────────────────────┐      │
│  │ Long-lived SSE GET │ ←──────────────────│ Short-lived POST     │      │
│  │ @get('/feed')      │    (read path)     │ @post('/command')    │      │
│  │                    │                    │                      │      │
│  │ Reconnects with    │                    │ Immediate response   │      │
│  │ Last-Event-ID      │                    │ + loading indicator  │      │
│  └─────────┬──────────┘                    └──────────┬───────────┘      │
│            │                                          │                  │
└────────────┼──────────────────────────────────────────┼──────────────────┘
             │                                          │
             │ SSE: text/event-stream                   │ POST: application/json
             │ id: <sequence_number>                    │
             │ data: elements <html>                    │
             │                                          │
┌────────────┼──────────────────────────────────────────┼──────────────────┐
│            ▼                                          ▼                  │
│  ┌──────────────────────────────┐         ┌─────────────────────────┐   │
│  │     SSE Handler (axum)       │         │  Command Handler (axum) │   │
│  │  - Extract Last-Event-ID     │         │  - Validate command     │   │
│  │  - Subscribe to broadcast    │         │  - Return 202 Accepted  │   │
│  │  - Replay missed events      │         └────────────┬────────────┘   │
│  │  - Stream future updates     │                      │                │
│  └──────────┬───────────────────┘                      │                │
│             │ tokio::sync::broadcast                   │                │
│             │ ::Receiver                               │                │
│             │                                          ▼                │
│  ┌──────────┴───────────────────┐         ┌─────────────────────────┐   │
│  │   Projection (in-memory)     │         │  Application Layer      │   │
│  │  - Subscribe to events       │ ◄───────│  - Emit events          │   │
│  │  - Maintain read model       │         │  - Pure logic           │   │
│  │  - Serve queries             │         └────────────┬────────────┘   │
│  └──────────────────────────────┘                      │                │
│                                                        │                │
│                                             ┌──────────▼────────────┐   │
│                                             │  Event Store (SQLite) │   │
│                                             │  - Append event       │   │
│                                             │  - Generate sequence  │   │
│                                             │  - Persist to WAL     │   │
│                                             └──────────┬────────────┘   │
│                                                        │                │
│                                             ┌──────────▼────────────┐   │
│                                             │ tokio::sync::broadcast│   │
│                                             │ ::Sender              │   │
│                                             │ - Fan-out to N subs   │   │
│                                             └───────────────────────┘   │
│                                                                          │
│                             Axum Server Process                          │
└──────────────────────────────────────────────────────────────────────────┘

Legend:
  ──►  Data flow
  ◄──  Subscribes to
```

## Ironstar defaults and recommendations

### Default architecture

The module structure below is a simplified view focused on event sourcing components.
See `../core/architecture-decisions.md` for the complete structure including the explicit `application/` layer for command and query handlers.

```
src/
├── domain/
│   ├── events.rs          # DomainEvent enum (sum type)
│   ├── commands.rs        # Command types (product types)
│   └── aggregates/        # Aggregate root logic (validation)
├── infrastructure/
│   ├── event_store.rs     # SqliteEventStore impl
│   ├── event_bus.rs       # Zenoh pub/sub setup
│   └── projections/       # Projection implementations
│       ├── mod.rs
│       ├── query_result.rs
│       └── chart_data.rs
└── presentation/
    ├── handlers/
    │   ├── sse.rs         # SSE feed handler
    │   └── commands.rs    # POST command handlers
    └── templates/         # hypertext components
```

### Configuration defaults

Key settings: Broadcast channel capacity 256 events, SQLite WAL mode with synchronous=FULL, SSE keep-alive 15s, Brotli compression enabled (200:1 ratio for HTML), projection rebuild on startup.

For complete configuration, SQLite tuning, and monitoring patterns, see `performance-tuning.md` and `../decisions/observability-decisions.md`.

## Trade-off analysis

### Event replay: sequence numbers vs timestamps

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Sequence numbers** | Monotonic, no clock skew, efficient indexing | Couples event identity to storage | **Use for ironstar** |
| **Timestamps** | Natural ordering, works across distributed systems | Clock skew, not unique, slower queries | Use with distributed event store (Zenoh future) |

### Projection caching: in-memory vs persisted snapshots

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **In-memory (rebuild)** | Simple, no cache invalidation, always consistent | Slow startup with many events | **Use for ironstar v1** |
| **Persisted snapshots** | Fast startup | Cache invalidation, snapshot versioning, migrations | Add later if startup becomes slow |
| **DuckDB views** | Optimized for analytics, incremental updates | Overkill for UI state, extra dependency | Use only for analytics projections |

### SSE replay: fat morph vs incremental

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Fat morph** | Resilient to missed events, simple | Larger payload per event | **Default for ironstar** |
| **Incremental (append/remove)** | Smaller payload | Brittle if events missed, complex | Use only when payload size is proven bottleneck |

### Broadcast channel: small vs large capacity

| Capacity | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Small (16)** | Low memory, fail fast on slow consumers | Lagged receivers trigger reconnects | Use for real-time apps with strict latency requirements |
| **Large (1024)** | Tolerates slow consumers | Higher memory usage, delayed error detection | Use for batch/analytics workloads |
| **Medium (256)** | Balanced | - | **Use for ironstar (default)** |

## Event schema evolution with upcasters

As the domain evolves, event schemas change.
Upcasters transform old event formats to current schemas during event loading, avoiding costly data migrations.

```rust
use serde_json::Value;

/// Transforms events from old schema versions to current schema
pub trait EventUpcaster: Send + Sync {
    /// Check if this upcaster handles the given event type and version
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool;

    /// Transform the event payload to the current schema
    fn upcast(&self, payload: Value) -> Value;
}

/// Registry of upcasters applied during event loading
pub struct UpcasterChain {
    upcasters: Vec<Box<dyn EventUpcaster>>,
}

impl UpcasterChain {
    pub fn new() -> Self {
        Self { upcasters: Vec::new() }
    }

    pub fn register(mut self, upcaster: Box<dyn EventUpcaster>) -> Self {
        self.upcasters.push(upcaster);
        self
    }

    /// Apply all matching upcasters to transform event to current schema
    pub fn upcast(&self, event_type: &str, event_version: &str, mut payload: Value) -> Value {
        for upcaster in &self.upcasters {
            if upcaster.can_upcast(event_type, event_version) {
                payload = upcaster.upcast(payload);
            }
        }
        payload
    }
}

/// Load events with automatic schema upcasting
pub fn load_events_with_upcasting<E>(
    raw_events: Vec<StoredEvent>,
    upcaster_chain: &UpcasterChain,
) -> Vec<E>
where
    E: serde::de::DeserializeOwned,
{
    raw_events
        .into_iter()
        .filter_map(|stored| {
            let event_version = stored.metadata
                .as_ref()
                .and_then(|m| m.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("1");

            let payload = upcaster_chain.upcast(
                &stored.event_type,
                event_version,
                stored.payload,
            );

            serde_json::from_value(payload).ok()
        })
        .collect()
}

// Example upcaster: QueryStarted v1 -> v2 (added dataset_ref field)
struct QueryStartedV1ToV2;

impl EventUpcaster for QueryStartedV1ToV2 {
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool {
        event_type == "QueryStarted" && event_version == "1"
    }

    fn upcast(&self, mut payload: Value) -> Value {
        // Add default dataset_ref if missing (for queries before multi-dataset support)
        if let Value::Object(ref mut map) = payload {
            if !map.contains_key("dataset_ref") {
                map.insert("dataset_ref".to_string(), Value::Null);
            }
        }
        payload
    }
}
```

Upcasters are applied lazily during event loading, not as batch migrations.
This keeps the event store immutable (events are facts that cannot change) while allowing the domain model to evolve.

## Analytics-specific projection patterns

Analytics projections differ from UI projections in that they cache expensive query results rather than simple in-memory state.
For ironstar, analytics projections combine event sourcing with DuckDB query caching.

### Query result caching pattern

```rust
use moka::future::Cache;
use std::sync::Arc;

/// Projection that caches DuckDB query results
pub struct QueryResultProjection {
    cache: Cache<String, Arc<Vec<serde_json::Value>>>,
    event_rx: tokio::sync::mpsc::Receiver<QueryEvent>,
}

impl QueryResultProjection {
    pub fn new(capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self { cache, event_rx }
    }

    /// React to QueryCompleted events by invalidating stale cache entries
    pub async fn handle_event(&mut self, event: QueryEvent) {
        match event {
            QueryEvent::QueryCompleted { query_id, .. } => {
                // Invalidate any cached results for this query
                self.cache.invalidate(&query_id).await;
            }
            QueryEvent::QueryFailed { query_id, .. } => {
                // Also invalidate on failure to allow retry
                self.cache.invalidate(&query_id).await;
            }
            _ => {}
        }
    }

    /// Serve cached result or execute query via DuckDB
    pub async fn get_or_compute(
        &self,
        query_id: &str,
        compute_fn: impl Future<Output = Result<Vec<serde_json::Value>, AppError>>,
    ) -> Result<Arc<Vec<serde_json::Value>>, AppError> {
        self.cache
            .try_get_with(query_id.to_string(), async move {
                compute_fn.await.map(Arc::new)
            })
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}
```

### Zenoh-based cache invalidation

When using Zenoh for event distribution, projections can subscribe to specific key expressions to receive invalidation events:

```rust
use zenoh::prelude::r#async::*;

/// Subscribe to query completion events for cache invalidation
pub async fn subscribe_query_events(
    session: Arc<zenoh::Session>,
    cache: Arc<Cache<String, Vec<serde_json::Value>>>,
) -> Result<(), AppError> {
    let subscriber = session
        .declare_subscriber("events/QuerySession/**")
        .res()
        .await?;

    tokio::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            if let Ok(event) = serde_json::from_slice::<QueryEvent>(sample.payload.contiguous().as_ref()) {
                match event {
                    QueryEvent::QueryCompleted { query_id, .. } |
                    QueryEvent::QueryFailed { query_id, .. } => {
                        cache.invalidate(&query_id).await;
                    }
                    _ => {}
                }
            }
        }
    });

    Ok(())
}
```

See `../infrastructure/analytics-cache-architecture.md` for complete cache design including TTL-based eviction and Zenoh invalidation patterns.

## Decider pattern with optimistic locking

ironstar uses fmodel-rust's Decider pattern for event sourcing.
The Decider pattern separates pure domain logic (decide/evolve) from effect-laden infrastructure (EventRepository).

### Decider type signatures

```rust
use fmodel_rust::decider::Decider;

/// Core Decider (pure domain)
pub struct Decider<'a, C, S, E, Error = ()> {
    /// Validates command against current state and emits events
    /// Pure function: no async, no I/O, no side effects
    pub decide: Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a + Send + Sync>,

    /// Applies an event to produce new state
    /// Pure state transition function
    pub evolve: Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>,

    /// Returns the initial state for a new aggregate
    pub initial_state: Box<dyn Fn() -> S + 'a + Send + Sync>,
}
```

### Mapping from previous Aggregate trait

The Decider pattern maps directly to ironstar's previous Aggregate trait:

| Previous Aggregate Trait | fmodel-rust Decider | Notes |
|--------------------------|---------------------|-------|
| `handle_command(state, cmd) → Result<Vec<Event>, Error>` | `decide: Fn(&C, &S) → Result<Vec<E>, Error>` | Direct 1:1 |
| `apply_event(state, event) → State` | `evolve: Fn(&S, &E) → S` | Direct 1:1 |
| `State::default()` | `initial_state: Fn() → S` | Direct 1:1 |
| `version()` method | `EventRepository::version_provider` | Version tracking moved to repository |

The key difference: version tracking moves from the Aggregate trait to the EventRepository implementation.
This separates pure domain logic (Decider) from infrastructure concerns (version management).

### Optimistic locking with EventRepository

The EventRepository trait provides `version_provider()` for optimistic concurrency control.
This prevents lost updates when two concurrent commands target the same aggregate.

```rust
use fmodel_rust::aggregate::EventRepository;

/// Infrastructure boundary for event persistence
pub trait EventRepository<C, E, Version, Error> {
    /// Fetch events for an aggregate identified by command
    fn fetch_events(&self, command: &C) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;

    /// Save events with optimistic locking
    /// Returns error if version conflict detected
    fn save(&self, events: &[E]) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;

    /// Get current version for optimistic lock
    fn version_provider(&self, event: &E) -> impl Future<Output = Result<Option<Version>, Error>> + Send;
}
```

**Command handler pattern with EventSourcedAggregate:**

```rust
use fmodel_rust::aggregate::EventSourcedAggregate;

async fn handle_command(
    repository: Arc<SqliteEventRepository>,
    decider: Decider<QueryCommand, QueryState, QueryEvent, QueryError>,
    command: QueryCommand,
) -> Result<Vec<(QueryEvent, Uuid)>, AppError> {
    let aggregate = EventSourcedAggregate::new(repository, decider);

    // EventSourcedAggregate handles:
    // 1. fetch_events() to load current state
    // 2. Fold events via evolve() to reconstruct state
    // 3. Call decide() with command and state
    // 4. save() new events with version_provider() for optimistic lock
    let events = aggregate.handle(&command).await?;

    Ok(events)
}
```

The EventRepository implementation (SQLite) uses the `previous_id` field for optimistic locking:
- First event in stream: `previous_id = NULL`
- Subsequent events: `previous_id = event_id` of previous event
- `UNIQUE(previous_id)` constraint prevents concurrent appends
- Conflict detection: second writer violates unique constraint → retry with fresh state

See `command-write-patterns.md` for complete command handler patterns including error handling and retry strategies.
For the SQLite EventRepository implementation, see the fmodel-rust adoption evaluation: `../decisions/fmodel-rust-adoption-evaluation.md`.

## Reference implementation: QuerySession Decider

For a concrete example of the Decider pattern applied to analytics workloads, see the northstar tracer bullet specification, which ironstar adapts to use fmodel-rust's Decider:

**Original specification (custom Aggregate trait):** `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/docs/notes/architecture/rust-cqrs-es-datastar/domain-layer.md`

**ironstar implementation:** QuerySession as a Decider function with the following characteristics:
- Session-scoped analytics with query lifecycle states (Pending → Executing → Completed/Failed)
- DuckDB async query execution spawned from application layer after event persistence
- Zenoh publication of result events for SSE broadcast
- Value objects with smart constructors (DatasetRef, SqlQuery, QueryId)

The Decider pattern enforces that query execution happens in the application layer (after `EventSourcedAggregate::handle()` persists events), not within the pure `decide()` function.
This preserves the async/sync boundary: Decider functions are pure and synchronous, while I/O operations occur in the application layer using the EventSourcedAggregate wrapper.

## Appendix: Common type definitions

The code examples throughout this document cluster reference types like `AppError`, `StoredEvent`, `DomainEvent`, and `ValidationError`.
These are example types that would typically live in `src/domain/` and `src/infrastructure/`.
Adapt them to your specific domain requirements.

```rust
//! Common types used in event sourcing examples
//! These would typically live in src/domain/ and src/infrastructure/

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application error type for handler responses
#[derive(Error, Debug)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

/// Validation errors for command handling
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("field '{field}' is required")]
    Required { field: &'static str },

    #[error("field '{field}' must be at most {max} characters")]
    TooLong { field: &'static str, max: usize },

    #[error("invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
}

/// Event stored in SQLite event store
/// Note: Derives Clone for use in tokio::sync::broadcast channels
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredEvent {
    pub sequence: i64,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Domain events (example for a QuerySession aggregate)
/// Sum type representing all possible events in the domain
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QueryEvent {
    QueryStarted {
        query_id: String,
        sql: String,
        dataset_ref: Option<String>,
        started_at: chrono::DateTime<chrono::Utc>,
    },
    QueryCompleted {
        query_id: String,
        row_count: usize,
        duration_ms: u64,
        completed_at: chrono::DateTime<chrono::Utc>,
    },
    QueryFailed {
        query_id: String,
        error: String,
        failed_at: chrono::DateTime<chrono::Utc>,
    },
}

impl QueryEvent {
    /// Extract event type name for storage in event_type column
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::QueryStarted { .. } => "QueryStarted",
            Self::QueryCompleted { .. } => "QueryCompleted",
            Self::QueryFailed { .. } => "QueryFailed",
        }
    }

    /// Extract aggregate ID for storage in aggregate_id column
    pub fn aggregate_id(&self) -> &str {
        match self {
            Self::QueryStarted { query_id, .. }
            | Self::QueryCompleted { query_id, .. }
            | Self::QueryFailed { query_id, .. } => query_id,
        }
    }
}

/// Commands (requests to change state)
/// Product types containing validated user input
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum QueryCommand {
    StartQuery {
        sql: String,
        dataset_ref: Option<String>,
    },
    CompleteQuery {
        query_id: String,
        row_count: usize,
        duration_ms: u64,
    },
    FailQuery {
        query_id: String,
        error: String,
    },
}
```

**Dependencies referenced in these types:**

- `serde` (v1.0): Serialization/deserialization
- `thiserror` (v1.0): Error derive macros
- `chrono` (v0.4): Date/time types
- `sqlx` (v0.9): Database errors

**Implementation notes:**

- `StoredEvent` derives `Clone` because it's sent through `tokio::sync::broadcast` channels, which require cloneable types.
- `DomainEvent` uses `#[serde(tag = "type")]` for tagged union JSON serialization, making event payloads human-readable.
- `ValidationError` uses `thiserror::Error` to automatically implement `std::error::Error` with proper Display formatting.
- `AppError` uses `#[from]` attribute to enable automatic conversion from `ValidationError` and `sqlx::Error` via the `?` operator.

## Future: Multi-aggregate coordination with Saga

For complex workflows requiring multi-aggregate coordination, fmodel-rust provides the Saga pattern for event-driven choreography.

**Saga pattern overview:**
- Pure function: `react: Fn(&ActionResult) → Vec<Action>`
- Maps action results (events from one aggregate) to new actions (commands for other aggregates)
- Enables process manager semantics without external orchestration

**When to use:**
- Multi-aggregate workflows requiring coordination (e.g., Order → Payment → Shipping)
- Compensating transactions for rollback scenarios
- Event-driven choreography between bounded contexts

**Example from fmodel-rust-demo:**

```rust
use fmodel_rust::saga::Saga;

pub fn order_payment_saga<'a>() -> Saga<'a, OrderEvent, PaymentCommand> {
    Saga {
        react: Box::new(|event| match event {
            OrderEvent::Placed { order_id, amount, .. } => {
                vec![PaymentCommand::InitiatePayment {
                    order_id: order_id.clone(),
                    amount: amount.clone(),
                }]
            }
            OrderEvent::Cancelled { order_id, .. } => {
                vec![PaymentCommand::RefundPayment {
                    order_id: order_id.clone(),
                }]
            }
            _ => vec![],
        }),
    }
}
```

Ironstar v1 uses single-aggregate Deciders with direct command handling.
Saga-based coordination is available when multi-aggregate workflows are needed.

**Alternative (deferred):** Free monad layer for advanced command composition and simulation.
See `~/.claude/commands/preferences/event-sourcing.md` lines 494-636 for free monad implementation patterns.

## References

### Datastar and SSE

- Datastar SDK ADR: `/Users/crs58/projects/lakescope-workspace/datastar/sdk/ADR.md`
- Tao of Datastar: `/Users/crs58/projects/lakescope-workspace/datastar-doc/guide_the_tao_of_datastar.md`
- Northstar Go template: `/Users/crs58/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
- Lince Rust example: `/Users/crs58/projects/rust-workspace/datastar-rust-lince/`
- SSE spec: https://html.spec.whatwg.org/multipage/server-sent-events.html

### Primary references

- Kevin Hoffman, *Real World Event Sourcing* — Ten Laws of Event Sourcing, process managers, injectors/notifiers
- Scott Wlaschin, *Domain Modeling Made Functional* — Aggregates as consistency boundaries, workflows as pipelines
- `~/.claude/commands/preferences/event-sourcing.md` — Theoretical synthesis and decision frameworks

### CQRS and event sourcing frameworks

**Primary framework (dependency):**

| Library | Role | Maturity |
|---------|------|----------|
| fmodel-rust | Decider pattern, EventRepository trait, Saga coordination, DeciderTestSpecification | Production |

Local paths:

- fmodel-rust library: `/Users/crs58/projects/rust-workspace/fmodel-rust/`
- fmodel-rust demo (Order + Restaurant): `/Users/crs58/projects/rust-workspace/fmodel-rust-demo/`
- fmodel-rust PostgreSQL adapter: `/Users/crs58/projects/rust-workspace/fmodel-rust-postgres/` (reference for SQLite port)
- fmodel (Kotlin original): `/Users/crs58/projects/rust-workspace/fmodel/` (canonical reference)

See `../decisions/fmodel-rust-adoption-evaluation.md` for complete evaluation and adoption rationale.

**Alternative frameworks (study material, not dependencies):**

| Library | Patterns Studied | Why Not Adopted |
|---------|------------------|-----------------|
| cqrs-es | TestFramework DSL, GenericQuery, Aggregate trait | Mutable `apply(&mut self)` violates purity |
| esrs | Pure sync aggregates, Schema/Upcaster pattern | PostgreSQL-only; no SQLite backend |
| sqlite-es | SQLite event store schema, optimistic locking | Thin adapter; patterns adopted, not library |
| kameo_es | Actor + ES composition, causation tracking, projection backends | Alpha maturity |
| SierraDB | Distributed event store design, partition-based sharding | Pre-production; overkill for single-node |

Local paths for pattern study:

- cqrs-es TestFramework: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/framework.rs`
- cqrs-es TestExecutor: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/executor.rs`
- cqrs-es TestValidator: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/validator.rs`
- esrs pure Aggregate trait: `/Users/crs58/projects/rust-workspace/event_sourcing.rs/src/aggregate.rs`
- sqlite-es event repository: `/Users/crs58/projects/rust-workspace/sqlite-es/src/event_repository.rs`
- kameo_es Entity trait: `/Users/crs58/projects/rust-workspace/kameo_es/` (Alpha — actor patterns)
- SierraDB event store: `/Users/crs58/projects/rust-workspace/sierradb/` (Pre-production — distributed design reference)
- CQRS pattern: https://martinfowler.com/bliki/CQRS.html

### Analytics caching

- Analytics cache architecture: `../infrastructure/analytics-cache-architecture.md` — moka cache with TTL, Zenoh-based invalidation, rkyv serialization

## Related documentation

- `sse-connection-lifecycle.md`: SSE connection state machine and debugging
- `event-replay-consistency.md`: Event replay patterns and consistency boundaries
- `projection-patterns.md`: Projection caching and DuckDB integration
- `performance-tuning.md`: Performance optimization strategies
- `command-write-patterns.md`: Command handlers and aggregate testing
- `../infrastructure/zenoh-event-bus.md`: Zenoh integration for distributed event bus
