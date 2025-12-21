# Crate architecture

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

**Naming convention**: Crate names use kebab-case (`ironstar-domain`) following crates.io convention. Rust normalizes these to snake_case for `use` statements (`use ironstar_domain::...`). This is consistent with ecosystem crates like `tower-http` and `tracing-subscriber`.

### Layer 1: Domain (depends on Layer 0)

Pure types with no async, no I/O, no infrastructure dependencies.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar-domain` | Aggregate definitions | Aggregate trait, state machines, apply_event |
| `ironstar-commands` | Command types | Command enums, validation logic (pure) |
| `ironstar-events` | Event types | DomainEvent enum, event metadata |

### Layer 2: Application (depends on Layers 0-1)

Pure business logic orchestrating domain types.

| Crate | Purpose | Contains |
|-------|---------|----------|
| `ironstar-app` | Command/query handlers | handle_command orchestration, projection updates |

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
│   │       ├── aggregates/
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

## Related documentation

- Layers 4-7 and composition patterns: `crate-services-composition.md`
- Design principles: `design-principles.md`
- Architecture decisions: `architecture-decisions.md`
- Event sourcing core concepts: `../cqrs/event-sourcing-core.md`
- Command write patterns: `../cqrs/command-write-patterns.md`
