# Crate architecture

> **Superseded by bounded-context decomposition**
>
> This document describes an obsolete layer-based crate decomposition (8 layers: Foundation through Binary, with crates like `common-enums`, `ironstar-domain`, `ironstar-commands`, `ironstar-events`, `ironstar-interfaces`, etc.).
> This approach was superseded by the bounded-context decomposition documented in `crate-decomposition-plan.md`, which organizes the codebase by domain boundary rather than architectural layer.
> The bounded-context approach is now fully implemented with 11 crates: `ironstar-core`, `ironstar-shared-kernel`, `ironstar-todo`, `ironstar-session`, `ironstar-analytics`, `ironstar-workspace`, `ironstar-event-store`, `ironstar-event-bus`, `ironstar-analytics-infra`, `ironstar-session-store`, and the `ironstar` binary.
>
> See `crate-decomposition-plan.md` for the authoritative decomposition architecture.
> This document is preserved for historical context as a design exploration.

This document details the multi-crate workspace decomposition plan for ironstar, synthesizing patterns from production Rust projects (Golem, Hyperswitch) adapted for event sourcing, CQRS, and Datastar SSE integration.

## Design influences

| Pattern Source | Patterns Adopted |
|----------------|------------------|
| Golem (~25 crates) | HasXxx capability traits, All<Ctx> composition root, port/adapter organization, configuration-driven adapter selection |
| Hyperswitch (~40 crates) | Three commons pattern, interfaces crate for ports, workspace lints, feature-gated complexity |

## Crate topology

```
                           ┌───────────────────────────────┐
                           │         ironstar              │
                           │    (Binary: wires all crates) │
                           └───────────────┬───────────────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    │                      │                      │
                    ▼                      ▼                      ▼
        ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐
        │   ironstar_web    │  │ ironstar_services │  │  ironstar_config  │
        │ (HTTP, SSE, HTML) │  │ (Adapter selection│  │ (Configuration    │
        │                   │  │  & composition)   │  │  types)           │
        └─────────┬─────────┘  └─────────┬─────────┘  └─────────┬─────────┘
                  │                      │                      │
                  └──────────────────────┼──────────────────────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
                    ▼                    ▼                    ▼
        ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐
        │ ironstar_adapters │  │ ironstar_analytics│  │ ironstar_projections│
        │ (Storage impls)   │  │ (DuckDB, cache)   │  │ (Read models)     │
        └─────────┬─────────┘  └─────────┬─────────┘  └─────────┬─────────┘
                  │                      │                      │
                  └──────────────────────┼──────────────────────┘
                                         │
                                         ▼
                           ┌───────────────────────────┐
                           │   ironstar_interfaces     │
                           │ (Port traits: EventStore, │
                           │  SessionStore, etc.)      │
                           └─────────────┬─────────────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
                    ▼                    ▼                    ▼
        ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐
        │ ironstar_domain   │  │ ironstar_app      │  │ ironstar_commands │
        │ (Aggregates,      │  │ (Handlers,        │  │ (Command types)   │
        │  events, values)  │  │  projections)     │  │                   │
        └─────────┬─────────┘  └─────────┬─────────┘  └─────────┬─────────┘
                  │                      │                      │
                  └──────────────────────┼──────────────────────┘
                                         │
                                         ▼
                           ┌───────────────────────────┐
                           │     Foundation Layer      │
                           │  ┌─────────────────────┐  │
                           │  │   common_enums      │  │
                           │  │   common_types      │  │
                           │  │   common_utils      │  │
                           │  └─────────────────────┘  │
                           └───────────────────────────┘
```

## Layered crate structure

### Layer 0: Foundation (no internal dependencies)

These crates have no ironstar dependencies and can be used by any other crate.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `common-enums` | Shared enumerations | AggregateType, EventType, ErrorCode, FilterType |
| `common-types` | Primitive wrappers | MinorUnit, Timestamp, Sequence, newtypes |
| `common-utils` | Cross-cutting utilities | Crypto, validation, serialization helpers, extension traits |

**Smart constructor examples:**

Newtypes in `common-types` use smart constructors to enforce invariants at construction time.
The inner value is private, and only validated instances can be created.

```rust
use uuid::Uuid;

/// A validated todo item ID (infallible construction).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TodoId(Uuid);

impl TodoId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TodoId {
    fn default() -> Self {
        Self::new()
    }
}
```

For types requiring validation, the constructor returns `Result`:

```rust
use crate::error::ValidationError;

/// A validated, non-empty todo text (1-500 characters).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoText(String);

impl TodoText {
    pub fn new(s: impl Into<String>) -> Result<Self, ValidationError> {
        let s = s.into();
        if s.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "text".to_string(),
            });
        }
        if s.len() > 500 {
            return Err(ValidationError::TooLong {
                field: "text".to_string(),
                max_length: 500,
                actual_length: s.len(),
            });
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

This pattern ensures invalid states are unrepresentable: code receiving a `TodoText` value knows it satisfies the length constraints without runtime checks.

**Naming convention**: Crate names use kebab-case (`ironstar-domain`) following crates.io convention. Rust normalizes these to snake_case for `use` statements (`use ironstar_domain::...`). This is consistent with ecosystem crates like `tower-http` and `tracing-subscriber`.

### Layer 1: Domain (depends on Layer 0)

Pure types with no async, no I/O, no infrastructure dependencies.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar-domain` | Decider functions | Decider pattern implementation (decide/evolve functions), state machines |
| `ironstar-commands` | Command types | Command enums, validation logic (pure) |
| `ironstar-events` | Event types | DomainEvent enum, event metadata |

### Layer 2: Application (depends on Layers 0-1)

Business logic orchestrating domain types with effect boundaries.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar-app` | Command/query handlers | EventSourcedAggregate wiring, projection updates |

### Layer 3: Interfaces (depends on Layers 0-2)

Port trait definitions for infrastructure abstractions.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar-interfaces` | Port traits | EventStore, SessionStore, AnalyticsCache, Projection traits |

### Layers 4-7: Infrastructure, Services, Presentation, and Binary

For detailed documentation on layers 4-7, including infrastructure adapters, service composition patterns, presentation layer, and binary crate organization, see `crate-services-composition.md`.

## Directory structure

```
ironstar/
├── Cargo.toml                              # Workspace root
├── crates/
│   ├── common-enums/                       # Layer 0
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/lib.rs
│   ├── common-types/                       # Layer 0
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/lib.rs
│   ├── common-utils/                       # Layer 0
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── crypto.rs
│   │       ├── validation.rs
│   │       └── ext_traits.rs
│   ├── ironstar-domain/                    # Layer 1
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── deciders/                    # Decider functions (decide/evolve pattern)
│   │       │   ├── mod.rs
│   │       │   └── todo.rs
│   │       ├── values.rs
│   │       └── signals.rs
│   ├── ironstar-commands/                  # Layer 1
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/lib.rs
│   ├── ironstar-events/                    # Layer 1
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/lib.rs
│   ├── ironstar-app/                       # Layer 2
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── command_handlers.rs
│   │       ├── query_handlers.rs
│   │       └── projection_handlers.rs
│   ├── ironstar-interfaces/                # Layer 3
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── event_store.rs              # EventStore trait
│   │       ├── session_store.rs            # SessionStore trait
│   │       ├── analytics_cache.rs          # AnalyticsCache trait
│   │       ├── projection.rs               # Projection trait
│   │       └── event_bus.rs                # EventBus trait
│   ├── ironstar-adapters/                  # Layer 4
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── event_store/
│   │       │   ├── mod.rs
│   │       │   ├── sqlite.rs
│   │       │   └── memory.rs
│   │       ├── session_store/
│   │       │   ├── mod.rs
│   │       │   └── sqlite.rs
│   │       ├── analytics_cache/
│   │       │   ├── mod.rs
│   │       │   └── moka.rs
│   │       └── event_bus/
│   │           ├── mod.rs
│   │           └── broadcast.rs
│   ├── ironstar-analytics/                 # Layer 4
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── queries.rs
│   │       └── cache_invalidation.rs
│   ├── ironstar-projections/               # Layer 4
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── todo_list.rs
│   │       └── analytics.rs
│   ├── ironstar-config/                    # Layer 4
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── adapters.rs                 # Adapter selection enums
│   │       └── environment.rs              # Environment config
│   ├── ironstar-services/                  # Layer 5
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs                   # HasXxx traits
│   │       ├── all.rs                      # All<Ctx> composition root
│   │       └── factories.rs                # Adapter factory functions
│   ├── ironstar-web/                       # Layer 6
│   │   ├── Cargo.toml
│   │   ├── crate.nix
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── routes.rs
│   │       ├── handlers/
│   │       │   ├── mod.rs
│   │       │   ├── sse.rs
│   │       │   └── commands.rs
│   │       ├── templates/
│   │       │   ├── mod.rs
│   │       │   ├── layouts.rs
│   │       │   ├── pages/
│   │       │   └── components/
│   │       ├── extractors.rs
│   │       └── assets.rs
│   └── ironstar/                           # Layer 7 (binary)
│       ├── Cargo.toml
│       ├── crate.nix
│       └── src/
│           └── main.rs
└── modules/                                # Nix configuration
    ├── rust.nix
    ├── dev-shell.nix
    └── ...
```

## Workspace bounded context module organization

The Workspace context contains 5 aggregates, each following the standard module structure.
This section provides a concrete example of how the abstract Layer 1 domain structure is instantiated.

### Directory layout

```
crates/ironstar/src/domain/
├── mod.rs                      # Domain module root, re-exports
├── traits.rs                   # Identifier, EventType, DeciderType, IsFinal
├── common/                     # Shared value objects
│   └── bounded_string.rs
├── shared_kernel/              # Cross-context types (Shared Kernel pattern)
│   ├── mod.rs
│   └── user_id.rs              # UserId (shared with Session context)
├── workspace/                  # WorkspaceAggregate
│   ├── mod.rs
│   ├── decider.rs
│   ├── commands.rs
│   ├── events.rs
│   ├── state.rs
│   ├── errors.rs
│   └── values.rs               # WorkspaceId, WorkspaceName, Visibility
├── workspace_preferences/      # WorkspacePreferences aggregate
│   └── ...                     # Same structure
├── dashboard/                  # Dashboard aggregate
│   └── ...
├── saved_query/                # SavedQuery aggregate
│   └── ...
├── user_preferences/           # UserPreferences aggregate
│   └── ...
└── views/                      # Read-side projections
    ├── mod.rs
    ├── todo.rs                 # TodoListView (reference)
    └── workspace.rs            # WorkspaceListView, DashboardLayoutView, etc.
```

### Per-aggregate module structure

Each aggregate follows this pattern:

| File | Purpose | Key types |
|------|---------|-----------|
| `mod.rs` | Re-exports public API | `pub use` statements |
| `decider.rs` | Pure Decider factory | `aggregate_decider()` function |
| `commands.rs` | Command sum type | Enum with `Identifier`, `DeciderType` |
| `events.rs` | Event sum type | Enum with `Identifier`, `EventType`, `IsFinal` |
| `state.rs` | State + status enum | Struct with helper methods |
| `errors.rs` | Error + ErrorKind | Struct with factory constructors |
| `values.rs` | Value objects | Smart constructors, validation |

### Aggregate ID patterns

Each aggregate uses a hierarchical ID scheme for routing:

| Aggregate | ID Pattern | Example |
|-----------|------------|---------|
| Workspace | `workspace_{uuid}` | `workspace_a1b2c3d4` |
| WorkspacePreferences | `workspace_{uuid}/preferences` | `workspace_a1b2c3d4/preferences` |
| Dashboard | `workspace_{uuid}/dashboard_{name}` | `workspace_a1b2c3d4/dashboard_main` |
| SavedQuery | `workspace_{uuid}/query_{name}` | `workspace_a1b2c3d4/query_sales` |
| UserPreferences | `user_{uuid}/preferences` | `user_x1y2z3/preferences` |

## Algebraic interpretation of layers

The 8-layer crate structure corresponds to semantic boundaries in the algebraic model:

| Crate Layer | Algebraic Structure | Example |
|-------------|---------------------|---------|
| Layer 0 (Foundation) | Initial objects, primitive types | `Sequence`, `Timestamp` |
| Layer 1 (Domain) | Free structures (sum types) | `TodoEvent`, `Command` enums, Decider functions |
| Layer 2 (Application) | Algebras and catamorphisms | `EventSourcedAggregate::handle`, `fold_events` |
| Layer 3 (Interfaces) | Port abstractions (type classes) | `EventRepository`, `EventStore` traits |
| Layer 4 (Infrastructure) | Effect implementations | SQLite EventRepository adapter |
| Layer 5 (Services) | Composition roots | `All`, `HasXxx` traits |
| Layer 6 (Presentation) | Projection functions | SSE handlers |
| Layer 7 (Binary) | Fixpoint (main loop) | `main.rs` |

The layer dependency rule (Layer N depends only on layers below) reflects algebraic generality: lower layers provide more general structures that higher layers specialize.
fmodel-rust's Decider pattern enforces purity at Layer 1 via type signatures: `decide` and `evolve` are synchronous functions with no async or I/O capabilities.

See [semantic-model.md](semantic-model.md) for the complete algebraic architecture.

## Related documentation

- Layers 4-7 and composition patterns: `crate-services-composition.md`
- Design principles: `design-principles.md`
- Architecture decisions: `architecture-decisions.md`
- Event sourcing core concepts: `../cqrs/event-sourcing-core.md`
- Command write patterns: `../cqrs/command-write-patterns.md`
