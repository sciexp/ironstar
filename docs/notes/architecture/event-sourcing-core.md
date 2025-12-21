# Event sourcing patterns for ironstar

This is the master index document for ironstar's event sourcing, projection, and SSE pipeline architecture.
This document provides the conceptual overview, architectural diagram, defaults, trade-offs, and references.
Detailed implementation patterns are split into focused documents linked below.

## Document cluster navigation

| Document | Focus |
|----------|-------|
| `event-sourcing-core.md` | Master index, architecture diagram, defaults, trade-offs (this document) |
| `sse-connection-lifecycle.md` | SSE connection state machine, subscription phases, debugging |
| `event-replay-consistency.md` | Event replay strategy, reconnection patterns, consistency guarantees |
| `projection-patterns.md` | Projection caching, DuckDB integration, projection trait |
| `performance-tuning.md` | Debouncing, batching, rate limiting, backpressure, metrics |
| `command-write-patterns.md` | Command handlers, aggregate patterns, testing DSL, event store |

For Zenoh event bus integration, see `zenoh-event-bus.md`.

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
See `architecture-decisions.md` for the complete structure including the explicit `application/` layer for command and query handlers.

```
src/
├── domain/
│   ├── events.rs          # DomainEvent enum (sum type)
│   ├── commands.rs        # Command types (product types)
│   └── aggregates/        # Aggregate root logic (validation)
├── infrastructure/
│   ├── event_store.rs     # SqliteEventStore impl
│   ├── event_bus.rs       # Broadcast channel setup
│   └── projections/       # Projection implementations
│       ├── mod.rs
│       ├── todo_list.rs
│       └── analytics.rs
└── presentation/
    ├── handlers/
    │   ├── sse.rs         # SSE feed handler
    │   └── commands.rs    # POST command handlers
    └── templates/         # hypertext components
```

### Configuration defaults

```toml
# config.toml
[event_sourcing]
# Broadcast channel capacity (number of events buffered)
broadcast_capacity = 256

# SQLite WAL mode (default: WAL)
sqlite_journal_mode = "WAL"

# SQLite synchronous mode (default: FULL for durability)
sqlite_synchronous = "FULL"

[sse]
# Keep-alive interval (prevent proxy timeouts)
keep_alive_seconds = 15

# Enable compression (Brotli via tower-http)
enable_compression = true

[projections]
# Projection rebuild on startup (default: true for simplicity)
rebuild_on_startup = true

# Future: snapshot interval (not implemented yet)
# snapshot_every_n_events = 1000
```

### Production considerations

#### SQLite tuning

```rust
use sqlx::SqlitePool;

// Optimize SQLite for event sourcing workload
sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
sqlx::query("PRAGMA synchronous=FULL").execute(&pool).await?;
sqlx::query("PRAGMA cache_size=-64000").execute(&pool).await?; // 64MB cache
sqlx::query("PRAGMA temp_store=MEMORY").execute(&pool).await?;
```

#### Compression

Enable Brotli compression for SSE responses:

```rust
use axum::{Router, routing::get};
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .route("/feed", get(sse_feed))
    .layer(CompressionLayer::new());
```

Datastar documentation claims 200:1 compression ratios for HTML over SSE with Brotli.

#### Monitoring

```rust
use prometheus::{IntCounter, IntGauge, Registry};

pub struct Metrics {
    events_appended: IntCounter,
    sse_connections: IntGauge,
    projection_lag: IntGauge,
}
// Note: Prometheus metrics require the prometheus crate in Cargo.toml
```

Track:
- Events appended per second
- Active SSE connections
- Projection lag (last processed sequence vs last appended sequence)
- Broadcast channel lag events

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
pub fn load_events_with_upcasting<A: Aggregate>(
    raw_events: Vec<StoredEvent>,
    upcaster_chain: &UpcasterChain,
) -> Vec<A::Event> {
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

// Example upcaster: TodoCreated v1 -> v2 (added priority field)
struct TodoCreatedV1ToV2;

impl EventUpcaster for TodoCreatedV1ToV2 {
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool {
        event_type == "TodoCreated" && event_version == "1"
    }

    fn upcast(&self, mut payload: Value) -> Value {
        // Add default priority if missing
        if let Value::Object(ref mut map) = payload {
            if !map.contains_key("priority") {
                map.insert("priority".to_string(), Value::String("normal".to_string()));
            }
        }
        payload
    }
}
```

Upcasters are applied lazily during event loading, not as batch migrations.
This keeps the event store immutable (events are facts that cannot change) while allowing the domain model to evolve.

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

/// Domain events (example for a Todo aggregate)
/// Sum type representing all possible events in the domain
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    TodoCreated { id: String, text: String },
    TodoCompleted { id: String },
    TodoDeleted { id: String },
    TodoTextUpdated { id: String, text: String },
}

impl DomainEvent {
    /// Extract event type name for storage in event_type column
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::TodoCreated { .. } => "TodoCreated",
            Self::TodoCompleted { .. } => "TodoCompleted",
            Self::TodoDeleted { .. } => "TodoDeleted",
            Self::TodoTextUpdated { .. } => "TodoTextUpdated",
        }
    }

    /// Extract aggregate ID for storage in aggregate_id column
    pub fn aggregate_id(&self) -> &str {
        match self {
            Self::TodoCreated { id, .. }
            | Self::TodoCompleted { id }
            | Self::TodoDeleted { id }
            | Self::TodoTextUpdated { id, .. } => id,
        }
    }
}

/// Commands (requests to change state)
/// Product types containing validated user input
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    CreateTodo { text: String },
    CompleteTodo { id: String },
    DeleteTodo { id: String },
    UpdateTodoText { id: String, text: String },
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

## References

### Datastar and SSE

- Datastar SDK ADR: `/Users/crs58/projects/lakescope-workspace/datastar/sdk/ADR.md`
- Tao of Datastar: `/Users/crs58/projects/lakescope-workspace/datastar-doc/guide_the_tao_of_datastar.md`
- Northstar Go template: `/Users/crs58/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
- Lince Rust example: `/Users/crs58/projects/rust-workspace/datastar-rust-lince/`
- SSE spec: https://html.spec.whatwg.org/multipage/server-sent-events.html

### CQRS and event sourcing frameworks

- cqrs-es TestFramework: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/framework.rs`
- cqrs-es TestExecutor: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/executor.rs`
- cqrs-es TestValidator: `/Users/crs58/projects/rust-workspace/cqrs-es/src/test/validator.rs`
- esrs pure Aggregate trait: `/Users/crs58/projects/rust-workspace/event_sourcing.rs/src/aggregate.rs`
- sqlite-es event repository: `/Users/crs58/projects/rust-workspace/sqlite-es/src/event_repository.rs`
- CQRS pattern: https://martinfowler.com/bliki/CQRS.html

### Analytics caching

- Analytics cache architecture: `analytics-cache-architecture.md` — moka cache with TTL, Zenoh-based invalidation, rkyv serialization

## Related documentation

- `sse-connection-lifecycle.md`: SSE connection state machine and debugging
- `event-replay-consistency.md`: Event replay patterns and consistency boundaries
- `projection-patterns.md`: Projection caching and DuckDB integration
- `performance-tuning.md`: Performance optimization strategies
- `command-write-patterns.md`: Command handlers and aggregate testing
- `zenoh-event-bus.md`: Zenoh integration for distributed event bus
