---
title: Crate decomposition plan
---

This document presents a concrete plan for decomposing ironstar's single monolithic crate into a fine-grained workspace of spec-aligned crates.
It supersedes the incremental decomposition strategy in `crate-services-composition.md` and revises the topology in `crate-architecture.md`, which described a *layer-based* decomposition (common-enums, common-types, ironstar-domain, ironstar-commands, ironstar-events, etc.).
The revised approach uses *bounded-context-based* decomposition where crate boundaries mirror the Idris2 `spec/` module boundaries, keeping each aggregate's 7-file structure cohesive within its context crate rather than fragmenting it across layer crates.

Patterns from the earlier documents that remain fully applicable include the HasXxx capability traits (from Golem), workspace lints (from Hyperswitch), per-crate Nix configuration via `crate.nix`, and the algebraic layer interpretation.
These are incorporated into the new topology without conflict.

## Motivation

The monolithic `crates/ironstar/` crate has reached ~38,655 lines across 12 domain aggregates, 4 architectural layers, and multiple infrastructure technologies (SQLite, DuckDB, Zenoh, moka).
Module-level separation is clean but enforced only by convention.
Crate boundaries provide compiler-enforced encapsulation, prevent accidental cross-domain coupling, enable parallel compilation, and allow nix-cargo-crane to cache and rebuild individual crates independently.

The Idris2 `spec/` tree already defines the authoritative bounded context boundaries.
Aligning crate boundaries to spec module boundaries means the type system enforces the same architectural invariants that the spec describes.

## Design influences

| Source | Crate count | Pattern adopted |
|--------|-------------|-----------------|
| Hyperswitch | ~39 | Workspace lints, feature-gated complexity, types/impl separation |
| Superposition | ~14 | Domain types as hub crate with feature flags, one crate per bounded context |
| Golem | ~25 | HasXxx capability traits, strict layering, feature-gated host/client |
| nix-cargo-crane `quick-start-workspace` | — | `fileSetForCrate`, per-crate derivations, `cargo-hakari` workspace hack |

## Spec-to-crate mapping

Each bounded context in the Idris2 spec maps to exactly one domain crate.
Sub-aggregates within a context (e.g., Catalog + QuerySession + Chart within Analytics) remain in the same crate because the spec composes them at the context level via `combine` operators, and sub-aggregates share value objects.

| Spec module | Crate | Contents |
|-------------|-------|----------|
| `Core/*` | `ironstar-core` | Decider/View/Saga re-exports from fmodel-rust; EventRepository, EventNotifier, EventSubscriber traits (ports); EventEnvelope, EventId, Timestamp; DeciderType, EventType, IsFinal traits; BoundedString and validated newtypes |
| `SharedKernel/*` | `ironstar-shared-kernel` | UserId, OAuthProvider |
| `Todo/*` | `ironstar-todo` | Commands, Events, State, Decider, Values, Errors, View |
| `Session/*` | `ironstar-session` | Commands, Events, State, Decider, Values, Errors, View |
| `Analytics/*` | `ironstar-analytics` | Catalog aggregate, QuerySession aggregate, Chart value objects, combined analyticsDecider |
| `Workspace/*` | `ironstar-workspace` | WorkspaceAggregate, WorkspacePreferences, Dashboard, SavedQuery, UserPreferences (5 aggregates), combined workspaceContextDecider |

Infrastructure crates are split by technology concern rather than by domain, because infrastructure implements generic port traits and does not contain domain logic.

| Crate | Layer | Contents |
|-------|-------|----------|
| `ironstar-event-store` | Infrastructure | SqliteEventRepository implementing EventRepository from core; SSE stream composition |
| `ironstar-event-bus` | Infrastructure | ZenohEventBus, EventBus trait implementation, key expression utilities, workspace subscriber factory |
| `ironstar-analytics-infra` | Infrastructure | DuckDBService, AnalyticsCache (moka), CachedAnalyticsService, CacheInvalidationRegistry, CacheDependency, EmbeddedCatalogs |
| `ironstar-session-store` | Infrastructure | SqliteSessionStore, TTL cleanup |
| `ironstar` | Binary | main.rs, AppState, config, axum routes, SSE endpoints, extractors, hypertext templates, datastar-rust integration, application layer command/query handlers |

## Dependency DAG

```
                      ironstar-core
                    /    |    |     \
                   /     |    |      \
       shared-kernel     |    |       \
          / \     \      |    |        \
         /   \     \     |    |         \
   session workspace analytics  todo
               \       /
                \     /   (Customer-Supplier: Dashboard -> Chart)
                 \   /
                  \ /

   event-store   event-bus   analytics-infra   session-store
        \            |            /                /
         \           |           /                /
          +----------+-----------+---------------+
                          |
                      ironstar
                    (binary crate)
```

Domain crates depend only on `ironstar-core` and optionally `ironstar-shared-kernel`.
The sole cross-domain dependency is `ironstar-workspace` depending on `ironstar-analytics` for the Customer-Supplier relationship (Dashboard references ChartType, SavedQuery references DatasetRef).
Infrastructure crates depend on `ironstar-core` for port traits and on their respective domain crates for domain-specific types.
The binary crate depends on everything and serves as the composition root.

## Cross-domain references

The Idris2 spec encodes exactly three cross-context couplings, all of which become explicit Cargo.toml dependencies:

| From | To | Type reference | Relationship |
|------|----|----------------|--------------|
| `ironstar-workspace` (Dashboard) | `ironstar-analytics` (Chart) | `ChartDefinitionRef` contains `ChartType` | Customer-Supplier |
| `ironstar-workspace` (SavedQuery) | `ironstar-analytics` (QuerySession) | `DatasetRef` (catalogUri + datasetName) | Customer-Supplier |
| `ironstar-session`, `ironstar-workspace` | `ironstar-shared-kernel` | `UserId` | Shared Kernel |

No other cross-context imports exist in the spec.
If additional cross-references arise during implementation, they should be evaluated against the spec before being added, as they may indicate a domain modeling issue rather than a legitimate dependency.

## Why bounded-context-based rather than layer-based

The earlier plan in `crate-architecture.md` proposed separating `ironstar-commands`, `ironstar-events`, and `ironstar-domain` as distinct crates at Layer 1.
This fragments each aggregate's 7-file structure (commands.rs, events.rs, state.rs, decider.rs, values.rs, errors.rs, mod.rs) across three crates, breaking the cohesion that the Idris2 spec modules define.

For example, under the layer-based approach, `Analytics.Catalog.Commands` and `Analytics.Catalog.Events` would live in different crates, even though the Decider's `decide` function takes a command and returns events within the same aggregate.
The spec keeps commands, events, state, and decider together in `Analytics.Catalog` because they form a single algebraic unit (the Decider).

The bounded-context approach preserves this cohesion.
Each domain crate contains the complete Decider for its context, including commands, events, state, values, errors, and views.
The fmodel-rust `combine` operators compose sub-aggregates within the crate, and the composed decider is the crate's public API.

The "three commons" pattern from Hyperswitch (common-enums, common-types, common-utils) was designed for a project with 39 crates where cross-cutting enums and types are shared by many independent domain crates.
In ironstar's case, the equivalent shared types (BoundedString, Timestamp, Sequence, AggregateType, ErrorCode) fit naturally in `ironstar-core` since every domain crate already depends on it for the Decider/View/Saga abstractions.
A separate foundation layer would add three crates with no independent consumers.

## Compilation parallelism

With the proposed 11-crate workspace, the compilation DAG has three waves:

1. `ironstar-core` and `ironstar-shared-kernel` compile first (minimal dependencies, fast)
2. All 4 domain crates compile in parallel (each depends only on wave 1)
3. All 4 infrastructure crates compile in parallel (each depends on wave 1 + its domain crate)
4. `ironstar` binary crate compiles last (depends on all)

On a machine with sufficient cores, the domain layer compiles in the time of the slowest single domain crate rather than the sum of all of them.
This is a significant improvement over the current monolithic crate where any change recompiles ~38K lines.

## nix-cargo-crane integration

The `flake.nix` follows the `quick-start-workspace` example pattern from nix-cargo-crane:

```nix
# Shared dependency artifacts built once
cargoArtifacts = craneLib.buildDepsOnly commonArgs;

# Per-crate source filtering
fileSetForCrate = crate:
  lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      (craneLib.fileset.commonCargoSources ./crates/ironstar-core)
      (craneLib.fileset.commonCargoSources crate)
    ];
  };

# Individual crate derivations
ironstar-analytics = craneLib.buildPackage (commonArgs // {
  pname = "ironstar-analytics";
  cargoExtraArgs = "-p ironstar-analytics";
  src = fileSetForCrate ./crates/ironstar-analytics;
});
```

Each domain crate's `fileSetForCrate` includes `ironstar-core` sources (since all domain crates depend on it) plus its own sources.
A change to `ironstar-analytics/` rebuilds only that crate plus downstream dependents (`ironstar-analytics-infra`, `ironstar-workspace`, and `ironstar`), leaving the event store, event bus, session store, todo, and session crates untouched.

Each crate may include a `crate.nix` for custom build inputs as described in `crate-services-composition.md`.

## HasXxx capability traits

The HasXxx pattern from `crate-services-composition.md` (adapted from Golem) remains applicable.
The trait definitions move into `ironstar-core` alongside the port traits, and the `All` composition root lives in the binary crate since it wires all concrete adapters.

Handler functions declare only the capabilities they need:

```rust
async fn handle_start_query<S: HasEventStore + HasEventBus + HasDuckDB>(
    services: &S,
    cmd: StartQueryCommand,
) -> Result<(), AppError> { ... }
```

This keeps handlers testable with minimal mock surface.

## Open design questions

Two decisions deferred to implementation:

*Re-export vs. direct dependency on fmodel-rust.*
Should `ironstar-core` re-export fmodel-rust's `Decider`, `View`, and `Saga` types so that domain crates only depend on `ironstar-core`?
Or should domain crates depend on both `ironstar-core` and `fmodel-rust` directly?
Re-exporting centralizes version management but creates a leaky abstraction if `ironstar-core` only passes types through.
Direct dependency is explicit but requires coordinated version bumps.

*Views crate placement.*
The `views/` module contains CQRS read models that span aggregate boundaries (e.g., `WorkspaceView` projects from multiple workspace aggregates).
These could live in their respective context crates (since the spec defines views per context) or in a separate `ironstar-views` crate if cross-context views emerge.
The spec currently defines views within each context module (`Analytics.QuerySession` has `queryHistoryView`, `Todo` has `todoListView`), so the default is to keep them in context crates unless cross-context views are needed.

## Migration sequence

The migration proceeds in phases that maintain a compilable workspace at every step.
Each phase creates one or more new crates, moves code, and updates dependencies.

### Phase 1: Extract `ironstar-core`

Create `crates/ironstar-core/` with:
- Port traits: EventRepository, EventNotifier, EventSubscriber, EventBus
- Event infrastructure: EventEnvelope, EventId, Timestamp
- Domain traits: DeciderType, EventType, IsFinal
- Common value objects: BoundedString and validated newtypes from `domain/common/`
- Domain error types: DomainError, ValidationError

The monolithic crate replaces its internal definitions with `use ironstar_core::*` imports.
This is the lowest-risk extraction since these types have no dependencies on other ironstar code.

### Phase 2: Extract `ironstar-shared-kernel`

Create `crates/ironstar-shared-kernel/` with:
- UserId, OAuthProvider

Depends on: nothing (or `ironstar-core` if UserId uses BoundedString).

### Phase 3: Extract domain crates

Create four domain crates in parallel:
- `crates/ironstar-todo/` — moves `domain/todo/` + `domain/views/todo.rs`
- `crates/ironstar-session/` — moves `domain/session/` + related view
- `crates/ironstar-analytics/` — moves `domain/analytics/`, `domain/catalog/`, `domain/query_session/` + related views
- `crates/ironstar-workspace/` — moves `domain/workspace/`, `domain/dashboard/`, `domain/saved_query/`, `domain/user_preferences/`, `domain/workspace_preferences/` + related views

Each depends on `ironstar-core` and optionally `ironstar-shared-kernel`.
`ironstar-workspace` additionally depends on `ironstar-analytics` for the Customer-Supplier references.

### Phase 4: Extract infrastructure crates

Create four infrastructure crates:
- `crates/ironstar-event-store/` — moves `infrastructure/event_store.rs`, `infrastructure/sse_stream.rs`
- `crates/ironstar-event-bus/` — moves `infrastructure/event_bus/`, `infrastructure/key_expr.rs`
- `crates/ironstar-analytics-infra/` — moves `infrastructure/analytics.rs`, `infrastructure/analytics_cache.rs`, `infrastructure/cached_analytics.rs`, `infrastructure/cache_invalidation.rs`, `infrastructure/cache_dependency.rs`, `infrastructure/embedded_catalogs.rs`
- `crates/ironstar-session-store/` — moves `infrastructure/session_store.rs`

Each depends on `ironstar-core` for port traits and on respective domain crates for domain-specific types.

### Phase 5: Collapse the monolith

The remaining `crates/ironstar/` becomes the binary crate containing:
- `main.rs` and startup sequence
- `config.rs` and `state.rs` (AppState)
- Application layer: command handlers, query handlers (from `application/`)
- Presentation layer: axum routes, SSE endpoints, extractors, templates (from `presentation/`)
- `All` composition root and HasXxx trait implementations

At this point, the `domain/` and `infrastructure/` directories are empty and can be removed.

### Phase 6: Workspace configuration

- Update root `Cargo.toml` workspace members
- Add `[workspace.lints]` configuration (from `crate-services-composition.md`)
- Add `[lints] workspace = true` to each crate's `Cargo.toml`
- Update `flake.nix` with per-crate derivations and `fileSetForCrate` functions
- Add `cargo-hakari` workspace hack crate if dependency divergence causes cache misses

## Verification

Each phase must pass `cargo check --workspace` and `cargo test --workspace` before proceeding.
The existing DeciderTestSpecification tests (1248 lines for QuerySession alone) serve as regression guards.
No behavioral changes should occur during the migration; this is a purely structural refactoring.

## Related documentation

- Earlier layer-based decomposition: `crate-architecture.md`
- HasXxx traits, workspace lints, incremental strategy: `crate-services-composition.md`
- Algebraic layer interpretation: `crate-architecture.md` section "Algebraic interpretation of layers"
- Design principles: `design-principles.md`
- Architecture decisions: `architecture-decisions.md`
- Idris2 spec cross-references: `../../spec/`
