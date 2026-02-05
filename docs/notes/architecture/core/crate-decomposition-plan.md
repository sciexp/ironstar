---
title: Crate decomposition plan
---

**Status: Completed** (epic 29b, 12/12 children closed).
This document was originally written as a plan for decomposing ironstar's single monolithic crate into a fine-grained workspace of spec-aligned crates.
The decomposition is now fully implemented with 11 crates in production.
This document has been updated to reflect the completed implementation, including architectural decisions made during implementation that diverged from or refined the original plan.

The decomposition superseded the incremental decomposition strategy in `crate-services-composition.md` and revised the topology in `crate-architecture.md`, which described a *layer-based* decomposition (common-enums, common-types, ironstar-domain, ironstar-commands, ironstar-events, etc.).
The implemented approach uses *bounded-context-based* decomposition where crate boundaries mirror the Idris2 `spec/` module boundaries, keeping each aggregate's 7-file structure cohesive within its context crate rather than fragmenting it across layer crates.

Patterns from the earlier documents that remain fully applicable include the HasXxx capability traits (from Golem), workspace lints (from Hyperswitch), per-crate Nix configuration via `crate.nix`, and the algebraic layer interpretation.
These are incorporated into the new topology without conflict.

## Motivation

The monolithic `crates/ironstar/` crate had reached ~38,655 lines across 12 domain aggregates, 4 architectural layers, and multiple infrastructure technologies (SQLite, DuckDB, Zenoh, moka).
Module-level separation was clean but enforced only by convention.
The decomposition introduced compiler-enforced encapsulation through crate boundaries, preventing accidental cross-domain coupling, enabling parallel compilation, and allowing nix-cargo-crane to cache and rebuild individual crates independently.

The Idris2 `spec/` tree defined the authoritative bounded context boundaries.
The implemented crate boundaries align to spec module boundaries, ensuring the type system enforces the same architectural invariants that the spec describes.

## Design influences

| Source | Crate count | Pattern adopted |
|--------|-------------|-----------------|
| Hyperswitch | ~39 | Workspace lints, feature-gated complexity, types/impl separation |
| Superposition | ~14 | Domain types as hub crate with feature flags, one crate per bounded context |
| Golem | ~25 | HasXxx capability traits, strict layering, feature-gated host/client |
| nix-cargo-crane `quick-start-workspace` | — | `fileSetForCrate`, per-crate derivations, `cargo-hakari` workspace hack |

## Spec-to-crate mapping

Each bounded context in the Idris2 spec maps to exactly one domain crate in the implemented workspace.
Sub-aggregates within a context (e.g., Catalog + QuerySession within Analytics, or the 5 workspace sub-aggregates) remain in the same crate because the spec composes them at the context level via `combine` operators, and sub-aggregates share value objects.

| Spec module | Crate | Implemented contents |
|-------------|-------|----------------------|
| `Core/*` | `ironstar-core` | EventRepository, EventNotifier, EventSubscriber traits (ports); EventEnvelope, EventId, Timestamp; DeciderType, EventType, IsFinal traits; BoundedString and validated newtypes; ErrorCode enum |
| `SharedKernel/*` | `ironstar-shared-kernel` | UserId, OAuthProvider |
| `Todo/*` | `ironstar-todo` | Commands, Events, State, Decider, Values, Errors, View |
| `Session/*` | `ironstar-session` | Commands, Events, State, Decider, Values, Errors, View |
| `Analytics/*` | `ironstar-analytics` | Catalog aggregate, QuerySession aggregate, combined analyticsDecider |
| `Workspace/*` | `ironstar-workspace` | WorkspaceAggregate, Dashboard, SavedQuery, WorkspacePreferences, UserPreferences (5 aggregates), combined workspaceContextDecider, workspace views |

**Implementation note:** `ironstar-core` does not re-export `Decider`, `View`, or `Saga` from fmodel-rust.
Domain crates depend on both `ironstar-core` and `fmodel-rust` directly.
This keeps `ironstar-core` focused on ironstar-specific abstractions (ports, event infrastructure, domain errors) rather than acting as a leaky facade.

Infrastructure crates are split by technology concern rather than by domain, because infrastructure implements generic port traits and does not contain domain logic.

| Crate | Layer | Implemented contents |
|-------|-------|----------------------|
| `ironstar-event-store` | Infrastructure | SqliteEventRepository implementing EventRepository from core; SSE stream utilities; EventStoreError with UUID tracking and backtrace |
| `ironstar-event-bus` | Infrastructure | ZenohEventBus implementing EventBus trait from core; key expression utilities; workspace subscriber factory; EventBusError with UUID tracking and backtrace |
| `ironstar-analytics-infra` | Infrastructure | DuckDBService, AnalyticsCache (moka), CachedAnalyticsService, CacheInvalidationRegistry, CacheDependency, EmbeddedCatalogs; AnalyticsInfraError with UUID tracking and backtrace |
| `ironstar-session-store` | Infrastructure | SqliteSessionStore, session cleanup; SessionStoreError with UUID tracking and backtrace |
| `ironstar` | Binary/Composition Root | main.rs, AppState, config; axum routes, SSE endpoints, extractors; hypertext templates, datastar-rust integration; application layer command/query handlers; InfrastructureError with `From` conversions; CommandPipelineError with `From` conversions; StaticAssets (embedded static files); SQL migration files; ts-rs TypeScript bindings; integration tests |

**Binary crate design note:** The application and presentation layers remain in the binary crate by design.
These are composition-root concerns that depend on all domain and infrastructure crates.
Command and query handlers orchestrate domain logic and infrastructure, requiring access to the full workspace.
They cannot be extracted into library crates without creating circular dependencies or artificial abstractions.
The binary crate retains `domain/mod.rs` and `infrastructure/mod.rs` as inline `pub mod` re-export modules to preserve `crate::domain::X` and `crate::infrastructure::X` import paths from the monolithic structure.

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

## Per-crate error type pattern

During implementation, an architectural pattern emerged for infrastructure error handling.
Each infrastructure crate defines its own error type with consistent structure:

| Crate | Error type | Features |
|-------|------------|----------|
| `ironstar-event-store` | `EventStoreError` | UUID tracking, backtrace capture, `error_code()` mapping to `ErrorCode` enum |
| `ironstar-event-bus` | `EventBusError` | UUID tracking, backtrace capture, `error_code()` mapping to `ErrorCode` enum |
| `ironstar-analytics-infra` | `AnalyticsInfraError` | UUID tracking, backtrace capture, `error_code()` mapping to `ErrorCode` enum |
| `ironstar-session-store` | `SessionStoreError` | UUID tracking, backtrace capture, `error_code()` mapping to `ErrorCode` enum |

The binary crate's `InfrastructureError` provides `From<XxxError>` conversions for each infrastructure error type, enabling unified error handling at composition boundaries.
The binary crate's `CommandPipelineError` also provides `From` conversions where needed for command handler orchestration.

This pattern preserves error context through the stack while allowing each infrastructure crate to define errors specific to its technology concern.

## Resolved design questions

Two decisions that were deferred to implementation have been resolved:

**fmodel-rust dependency strategy** (resolved: direct dependency).
Domain crates depend on both `ironstar-core` and `fmodel-rust` directly rather than using re-exports.
This keeps `ironstar-core` focused on ironstar-specific abstractions (ports, event infrastructure, domain errors) rather than acting as a leaky facade for upstream types.
The tradeoff is explicit: coordinated version bumps are required, but the dependency graph remains transparent.

**Views crate placement** (resolved: context crates).
Views live in their respective context crates following the Idris2 spec structure.
The spec defines views within each context module (`Analytics.QuerySession` has `queryHistoryView`, `Todo` has `todoListView`, `Workspace` context has workspace views).
The implementation followed this structure without requiring a separate `ironstar-views` crate.
Cross-context views have not emerged; if they do, they would be evaluated against the spec before being added.

## Implementation sequence

The decomposition was implemented in phases that maintained a compilable workspace at every step.
Each phase created one or more new crates, moved code, and updated dependencies.

### Phase 1: Extract `ironstar-core` (completed: 29b.1)

Created `crates/ironstar-core/` with:
- Port traits: EventRepository, EventNotifier, EventSubscriber
- Event infrastructure: EventEnvelope, EventId, Timestamp
- Domain traits: DeciderType, EventType, IsFinal
- Common value objects: BoundedString and validated newtypes from `domain/common/`
- Domain error types: DomainError, ValidationError
- ErrorCode enum for infrastructure error mapping

The monolithic crate replaced its internal definitions with `use ironstar_core::*` imports.
This was the lowest-risk extraction since these types had no dependencies on other ironstar code.

### Phase 2: Extract `ironstar-shared-kernel` (completed: 29b.2)

Created `crates/ironstar-shared-kernel/` with:
- UserId, OAuthProvider

Depends on `ironstar-core` (UserId uses BoundedString).

### Phase 3: Extract domain crates (completed: 29b.3, 29b.4, 29b.5, 29b.6)

Created four domain crates in parallel:
- `crates/ironstar-todo/` (29b.3) — moved `domain/todo/` + `domain/views/todo.rs`
- `crates/ironstar-session/` (29b.4) — moved `domain/session/` + related view
- `crates/ironstar-analytics/` (29b.5) — moved `domain/analytics/`, `domain/catalog/`, `domain/query_session/` + related views
- `crates/ironstar-workspace/` (29b.6) — moved `domain/workspace/`, `domain/dashboard/`, `domain/saved_query/`, `domain/user_preferences/`, `domain/workspace_preferences/` + related views

Each depends on `ironstar-core` and optionally `ironstar-shared-kernel`.
`ironstar-workspace` additionally depends on `ironstar-analytics` for the Customer-Supplier references.

### Phase 4: Extract infrastructure crates (completed: 29b.7, 29b.8, 29b.9, 29b.10)

Created four infrastructure crates:
- `crates/ironstar-event-store/` (29b.7) — moved event store implementation, SSE stream utilities; introduced EventStoreError with UUID tracking and backtrace
- `crates/ironstar-event-bus/` (29b.8) — moved Zenoh event bus, key expression utilities, workspace subscriber factory; introduced EventBusError with UUID tracking and backtrace
- `crates/ironstar-analytics-infra/` (29b.9) — moved DuckDB service, moka cache, cache invalidation, embedded catalogs; introduced AnalyticsInfraError with UUID tracking and backtrace
- `crates/ironstar-session-store/` (29b.10) — moved session store implementation, TTL cleanup; introduced SessionStoreError with UUID tracking and backtrace

Each depends on `ironstar-core` for port traits and on respective domain crates for domain-specific types.
The per-crate error type pattern emerged during this phase, establishing a consistent error handling architecture across all infrastructure crates.

### Phase 5: Collapse the monolith to binary (completed: 29b.11)

The remaining `crates/ironstar/` became the binary crate containing:
- `main.rs` and startup sequence
- `config.rs` and `state.rs` (AppState)
- Application layer: command handlers, query handlers (from `application/`)
- Presentation layer: axum routes, SSE endpoints, extractors, templates (from `presentation/`)
- `domain/mod.rs` and `infrastructure/mod.rs` as inline `pub mod` re-export modules (not empty)
- InfrastructureError with `From` conversions for infrastructure crate error types
- CommandPipelineError with `From` conversions where needed
- StaticAssets for embedded static files
- SQL migration files in `migrations/`
- ts-rs TypeScript bindings in `bindings/`
- Integration tests in `tests/`

**Implementation note:** The `domain/` and `infrastructure/` directories were not removed.
Instead, they remain as thin module re-export layers (`pub mod session { pub use ironstar_session::*; }`) to preserve import paths like `crate::domain::session` from the monolithic structure.
This minimized churn in the binary crate's application and presentation layers during the transition.

### Phase 6: Workspace configuration (completed: 29b.12)

- Updated root `Cargo.toml` workspace members
- Added `[workspace.lints]` configuration
- Added `[lints] workspace = true` to each crate's `Cargo.toml`
- Updated `flake.nix` with per-crate derivations using rust-flake autowiring
- Created `crates/ironstar/crate.nix` as a custom override for the binary crate (only crate requiring custom configuration)
- Evaluated `cargo-hakari` workspace hack crate; deferred as optional since crane shared `cargoArtifacts` caching is sufficient

## Verification

Each phase passed `cargo check --workspace` and `cargo test --workspace` before proceeding.
The existing DeciderTestSpecification tests (1248 lines for QuerySession alone) served as regression guards.
No behavioral changes occurred during the migration; this was a purely structural refactoring.

**Final test counts:** 897 tests passing across the workspace (891 baseline + 6 from new per-crate error type tests added during infrastructure extraction).

## Related documentation

- Earlier layer-based decomposition: `crate-architecture.md`
- HasXxx traits, workspace lints, incremental strategy: `crate-services-composition.md`
- Algebraic layer interpretation: `crate-architecture.md` section "Algebraic interpretation of layers"
- Design principles: `design-principles.md`
- Architecture decisions: `architecture-decisions.md`
- Idris2 spec cross-references: `../../spec/`
