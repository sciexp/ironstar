# Crate services and composition

This document continues from `crate-architecture.md` which covers the foundational layers 0-3 (Foundation, Domain, Application, and Interfaces).
This document details layers 4-7 (Infrastructure, Services, Presentation, and Binary) along with composition patterns, service traits, and the migration path from single-crate to multi-crate architecture.

## Layered crate structure (continued)

### Layer 4: Infrastructure (depends on Layers 0-3)

Effect implementations and adapters.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar-adapters` | Storage adapters | SQLite, moka, DuckDB implementations |
| `ironstar-analytics` | Analytics layer | DuckDB queries, cache invalidation |
| `ironstar-projections` | Read model implementations | In-memory projections, snapshot support |
| `ironstar-config` | Configuration types | Config structs, adapter selection enums |

### Layer 5: Services (depends on Layers 0-4)

Service composition and dependency injection.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar-services` | HasXxx traits, All composition | Service traits, composition root, adapter factories |

### Layer 6: Presentation (depends on Layers 0-5)

HTTP boundary layer.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar-web` | HTTP + SSE + HTML | axum handlers, hypertext templates, routes |

### Layer 7: Binary (depends on all)

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar` | Main binary | main.rs, CLI, process entry point |

## HasXxx capability trait pattern

Adapted from Golem's composition pattern for fine-grained dependency injection.

### Trait definitions

```rust
// In ironstar-services/src/traits.rs
// Note: kebab-case package names become snake_case in use statements

use std::sync::Arc;
use ironstar_interfaces::{EventStore, SessionStore, AnalyticsCache, Projection};

/// Fine-grained capability traits for service access
pub trait HasEventStore {
    fn event_store(&self) -> Arc<dyn EventStore>;
}

pub trait HasSessionStore {
    fn session_store(&self) -> Arc<dyn SessionStore>;
}

pub trait HasAnalyticsCache {
    fn analytics_cache(&self) -> Arc<dyn AnalyticsCache>;
}

pub trait HasEventBus {
    fn event_bus(&self) -> tokio::sync::broadcast::Sender<StoredEvent>;
}

pub trait HasProjections {
    fn projections(&self) -> Arc<Projections>;
}

pub trait HasConfig {
    fn config(&self) -> Arc<IronstarConfig>;
}

/// Shortcut: all capabilities combined
pub trait HasAll:
    HasEventStore
    + HasSessionStore
    + HasAnalyticsCache
    + HasEventBus
    + HasProjections
    + HasConfig
    + Clone
    + Send
    + Sync
{}

/// Blanket impl: any type implementing all traits gets HasAll
impl<T> HasAll for T
where
    T: HasEventStore
        + HasSessionStore
        + HasAnalyticsCache
        + HasEventBus
        + HasProjections
        + HasConfig
        + Clone
        + Send
        + Sync
{}
```

### Composition root

```rust
// In ironstar-services/src/all.rs

use std::sync::Arc;

/// Composition root holding all services
#[derive(Clone)]
pub struct All {
    event_store: Arc<dyn EventStore>,
    session_store: Arc<dyn SessionStore>,
    analytics_cache: Arc<dyn AnalyticsCache>,
    event_bus: tokio::sync::broadcast::Sender<StoredEvent>,
    projections: Arc<Projections>,
    config: Arc<IronstarConfig>,
}

impl All {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        session_store: Arc<dyn SessionStore>,
        analytics_cache: Arc<dyn AnalyticsCache>,
        event_bus: tokio::sync::broadcast::Sender<StoredEvent>,
        projections: Arc<Projections>,
        config: Arc<IronstarConfig>,
    ) -> Self {
        Self {
            event_store,
            session_store,
            analytics_cache,
            event_bus,
            projections,
            config,
        }
    }
}

// Implement all HasXxx traits for All
impl HasEventStore for All {
    fn event_store(&self) -> Arc<dyn EventStore> {
        self.event_store.clone()
    }
}

impl HasSessionStore for All {
    fn session_store(&self) -> Arc<dyn SessionStore> {
        self.session_store.clone()
    }
}

// ... remaining trait impls
```

### Usage in handlers

```rust
// Handlers declare only the capabilities they need
async fn handle_add_todo<S: HasEventStore + HasEventBus>(
    services: &S,
    cmd: AddTodoCommand,
) -> Result<(), AppError> {
    let events = Todo::handle_command(&state, cmd)?;

    for event in events {
        services.event_store().append(event.clone()).await?;
        let _ = services.event_bus().send(event);
    }

    Ok(())
}

// Full capability access when needed
async fn bootstrap<S: HasAll>(services: &S) -> Result<(), AppError> {
    // Access all services
}
```

## Port trait organization

Adapted from Golem's storage pattern with observability labels.

```rust
// In ironstar-interfaces/src/event_store.rs

use async_trait::async_trait;

/// Port trait for event persistence
#[async_trait]
pub trait EventStore: Send + Sync + std::fmt::Debug {
    /// Append event and return assigned sequence number
    async fn append(&self, event: NewEvent) -> Result<StoredEvent, EventStoreError>;

    /// Load all events for replay
    async fn load_all(&self) -> Result<Vec<StoredEvent>, EventStoreError>;

    /// Load events since sequence (for SSE reconnection)
    async fn load_since(&self, sequence: i64) -> Result<Vec<StoredEvent>, EventStoreError>;

    /// Load events for specific aggregate
    async fn load_aggregate(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;
}
```

## Configuration-driven adapter selection

Adapted from Golem's tagged enum pattern.

```rust
// In ironstar-config/src/adapters.rs

use serde::{Deserialize, Serialize};

/// Event store backend selection
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum EventStoreConfig {
    Sqlite(SqliteConfig),
    InMemory(InMemoryConfig),
}

/// Analytics cache backend selection
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum AnalyticsCacheConfig {
    Moka(MokaConfig),
    None(NoCacheConfig),
}

/// Session store backend selection
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum SessionStoreConfig {
    Sqlite(SqliteConfig),
    InMemory(InMemoryConfig),
}

// In ironstar-services/src/factories.rs

/// Create event store from configuration
pub async fn create_event_store(
    config: EventStoreConfig,
) -> Result<Arc<dyn EventStore>, AdapterError> {
    match config {
        EventStoreConfig::Sqlite(cfg) => {
            let pool = SqlitePool::connect(&cfg.database_url).await?;
            Ok(Arc::new(SqliteEventStore::new(pool).await?))
        }
        EventStoreConfig::InMemory(_) => {
            Ok(Arc::new(InMemoryEventStore::new()))
        }
    }
}
```

## Three commons pattern

Adapted from Hyperswitch for foundation types.

### common_enums

Shared enumerations used across request/response and domain layers.

```rust
// In common-enums/src/lib.rs

use serde::{Deserialize, Serialize};

/// Aggregate type identifiers
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AggregateType {
    Todo,
    User,
    Session,
}

/// Error classification codes
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ValidationFailed,
    NotFound,
    Unauthorized,
    InternalError,
}

/// Filter types for queries
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    #[default]
    All,
    Active,
    Completed,
}
```

### common_types

Primitive wrappers with validation.

```rust
// In common-types/src/lib.rs

use serde::{Deserialize, Serialize};

/// Monotonic sequence number (event ordering)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sequence(i64);

impl Sequence {
    pub const ZERO: Self = Sequence(0);

    pub fn new(value: i64) -> Option<Self> {
        if value >= 0 { Some(Self(value)) } else { None }
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

/// Unix timestamp in milliseconds
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp_millis())
    }

    pub fn as_millis(&self) -> i64 {
        self.0
    }
}
```

### common_utils

Cross-cutting utilities.

```rust
// In common-utils/src/lib.rs

pub mod crypto;
pub mod validation;
pub mod ext_traits;

// In common-utils/src/ext_traits.rs

/// Extension trait for Option with error context
pub trait OptionExt<T> {
    fn ok_or_not_found(self, entity: &'static str, id: &str) -> Result<T, NotFoundError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self, entity: &'static str, id: &str) -> Result<T, NotFoundError> {
        self.ok_or_else(|| NotFoundError {
            entity,
            id: id.to_string(),
        })
    }
}
```

## Workspace lint configuration

Adapted from Hyperswitch for consistent code quality.

```toml
# In Cargo.toml (workspace root)

[workspace.lints.rust]
unsafe_code = "forbid"
rust_2018_idioms = { level = "warn", priority = -1 }
unused_qualifications = "warn"

[workspace.lints.clippy]
# Panic prevention
panic = "warn"
panic_in_result_fn = "warn"
unwrap_used = "warn"
expect_used = "warn"
unwrap_in_result = "warn"
todo = "warn"
unimplemented = "warn"

# Code quality
as_conversions = "warn"
cloned_instead_of_copied = "warn"
dbg_macro = "warn"
use_self = "warn"

# Safety
indexing_slicing = "warn"
large_futures = "warn"

# Debugging
print_stdout = "warn"
print_stderr = "warn"

# Per-crate Cargo.toml inherits workspace lints:
# [lints]
# workspace = true
```

## Per-crate Nix configuration

Each crate can have a `crate.nix` file for customized build requirements.

```nix
# Example: crates/ironstar-adapters/crate.nix
{ config, pkgs, lib, ... }:
{
  # Additional build inputs for this crate
  crane.args = {
    buildInputs = with pkgs; [ openssl sqlite ];
    nativeBuildInputs = with pkgs; [ pkg-config ];
  };

  # Which outputs to auto-wire to flake
  autoWire = [ "crate" "doc" "clippy" ];
}

# Example: crates/ironstar-analytics/crate.nix
{ config, pkgs, lib, ... }:
{
  crane.args = {
    # DuckDB bundled build requires extra dependencies
    nativeBuildInputs = with pkgs; [ cmake ];
  };
}
```

## Incremental decomposition strategy

The workspace structure supports incremental migration from single crate:

### Phase 1: Foundation (current)

Single `crates/ironstar/` crate with module organization matching layer structure.

### Phase 2: Extract interfaces

Create `ironstar-interfaces` crate with port traits.
Existing code depends on concrete implementations until Phase 3.

### Phase 3: Extract domain

Create `common-*` and `ironstar-domain` crates.
Pure types with no infrastructure dependencies.

### Phase 4: Extract adapters

Create `ironstar-adapters` implementing port traits.
Configuration-driven adapter selection.

### Phase 5: Extract services

Create `ironstar-services` with HasXxx pattern.
Full dependency injection via composition root.

### Phase 6: Extract presentation

Create `ironstar-web` for HTTP layer.
Binary crate becomes thin wiring.

## Dependency rules (enforced by layer)

| From Layer | Can Depend On |
|------------|---------------|
| Layer 0 (Foundation) | External crates only |
| Layer 1 (Domain) | Layer 0 |
| Layer 2 (Application) | Layers 0-1 |
| Layer 3 (Interfaces) | Layers 0-2 |
| Layer 4 (Infrastructure) | Layers 0-3 |
| Layer 5 (Services) | Layers 0-4 |
| Layer 6 (Presentation) | Layers 0-5 |
| Layer 7 (Binary) | All layers |

**Anti-pattern**: Lower layers depending on higher layers (e.g., domain depending on infrastructure).

## Related documentation

- Foundational layers 0-3: `crate-architecture.md`
- Design principles: `design-principles.md`
- Architecture decisions: `architecture-decisions.md`
- Event sourcing core concepts: `event-sourcing-core.md`
- Command write patterns: `command-write-patterns.md`
