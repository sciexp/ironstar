# Local dependency paths

All dependencies with local source code available for reference.

## Core Rust dependencies

| Dependency | Local Path | Description |
|------------|------------|-------------|
| axum | `~/projects/rust-workspace/axum` | Web framework with SSE support |
| tokio | `~/projects/rust-workspace/tokio` | Async runtime |
| hypertext | `~/projects/rust-workspace/hypertext` | Lazy HTML templating (maud-compatible syntax) |
| sqlx | `~/projects/rust-workspace/sqlx` | Async SQL with compile-time validation |
| moka | `~/projects/rust-workspace/moka-caching` | Async in-memory cache with TTL for analytics |
| rkyv | `~/projects/rust-workspace/rkyv-deserialization` | Zero-copy deserialization for cache serialization |
| async-duckdb | `~/projects/rust-workspace/async-duckdb` | Async DuckDB wrapper with connection pooling |
| duckdb-rs | `~/projects/omicslake-workspace/duckdb-rs` | DuckDB Rust bindings (wrapped by async-duckdb) |
| ts-rs | `~/projects/rust-workspace/ts-rs` | TypeScript type generation from Rust structs |
| rust-embed | `~/projects/rust-workspace/rust-embed` | Static asset embedding at compile time |
| fmodel-rust | `~/projects/rust-workspace/fmodel-rust` | Functional domain modeling with Decider pattern (primary ES abstraction) |

## fmodel ecosystem

The fmodel ecosystem is built upon Jeremie Chassaing's Decider pattern, the minimal algebraic interface for functional event sourcing.
The Decider pattern enforces near-pure functional semantics: `decide(command, state) -> events` and `evolve(state, event) -> state` are synchronous pure functions with no I/O, making aggregates trivially testable and composable.
Fraktalio maintains implementations across languages (Kotlin, Rust, TypeScript) and the fstore-sql persistence layer, all sharing this foundational pattern.
ironstar adopts fmodel-rust as its primary event sourcing abstraction based on the evaluation in `docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md`.

| Repository | Local Path | Description |
|------------|------------|-------------|
| fmodel-rust | `~/projects/rust-workspace/fmodel-rust` | Core library: Decider, View, Saga, EventSourcedAggregate |
| fmodel-rust-demo | `~/projects/rust-workspace/fmodel-rust-demo` | Working example implementing fstore-sql pattern with sqlx (Order + Restaurant domains) |
| fmodel-rust-postgres | `~/projects/rust-workspace/fmodel-rust-postgres` | PostgreSQL EventRepository implementing fstore-sql pattern via pgrx |
| fstore-sql | `~/projects/rust-workspace/fstore-sql` | Canonical PostgreSQL event store schema (SQL functions API, not Rust); reference for all EventRepository implementations |
| fmodel (Kotlin) | `~/projects/rust-workspace/fmodel` | Original Kotlin implementation (canonical reference) |

### fstore-sql pattern

fstore-sql is not a Rust crate but a PostgreSQL-native event store schema maintained by Fraktalio.
Both fmodel-rust-demo and fmodel-rust-postgres implement this pattern rather than importing it as a dependency.
The pattern provides:

- `events` table with `previous_id` UUID chain for optimistic locking
- `offset BIGSERIAL` for global monotonic ordering (SSE Last-Event-ID)
- Immutability via PostgreSQL rules preventing DELETE/UPDATE
- `stream_events()` function with partition locking for concurrent consumers

ironstar's SQLite schema (in the evaluation document, lines 135-198) adapts this pattern for SQLite.
Key files: `~/projects/rust-workspace/fstore-sql/schema.sql` (468 lines), `~/projects/rust-workspace/fstore-sql/README.md`.

### Key abstractions

- `Decider<C, S, E>`: Pure decision-making (`decide`, `evolve`, `initial_state`)
- `View<S, E>`: Pure event projection for read models
- `Saga<AR, A>`: Event-to-command choreography for process managers
- `EventSourcedAggregate`: Application layer wiring Decider + EventRepository

### Composition

- `combine()`: Merge independent Deciders (different command/event types)
- `merge()`: Merge Deciders/Views with same event type

See `~/projects/rust-workspace/fmodel-rust/README.md` for complete API documentation.

## CQRS/Event sourcing references (historical)

These libraries were evaluated during ironstar's architecture design phase.
fmodel-rust was selected over cqrs-es/esrs due to its pure function enforcement and algebraic composition model.
See `docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md` for the complete evaluation.

For theoretical foundations and cross-cutting event sourcing principles, see `~/.claude/commands/preferences/event-sourcing.md`.
That document synthesizes Hoffman's Laws with category-theoretic grounding and provides decision frameworks for when to use event sourcing.

### Primary source materials

| Source | Local Path | Key Patterns |
|--------|------------|--------------|
| Kevin Hoffman, *Real World Event Sourcing* (2024) | `~/projects/functional-programming-workspace/real-world-event-sourcing/` | Ten Laws of Event Sourcing, process managers, injectors/notifiers, event schema evolution |
| Debasish Ghosh, *Functional and Reactive Domain Modeling* (2016) | `~/projects/functional-programming-workspace/functional-and-reactive-domain-modeling/` | Module algebra (signatures/algebras/interpreters), algebraic laws, free monads, reactive streams |
| Scott Wlaschin, *Domain Modeling Made Functional* (2018) | `~/projects/functional-programming-workspace/domain-modeling-made-functional/` | Aggregates as consistency boundaries, workflows as pipelines, railway-oriented programming |

The preference documents synthesize all three approaches:
- Wlaschin provides practical patterns (smart constructors, workflows as pipelines, making illegal states unrepresentable)
- Ghosh provides algebraic foundations (signatures/algebras/interpreters, laws as specifications, abstraction hierarchy)
- Hoffman provides event sourcing depth (aggregate design, projection patterns, process managers, operational concerns)

### Rust pattern libraries (evaluated, not adopted as dependencies)

| Reference | Local Path | Patterns Studied | Why Not Adopted |
|-----------|------------|------------------|-----------------|
| cqrs-es | `~/projects/rust-workspace/cqrs-es` | Aggregate trait, EventStore abstraction, TestFramework DSL | Mutable `apply(&mut self)` violates purity |
| sqlite-es | `~/projects/rust-workspace/sqlite-es` | SQLite event schema, optimistic locking | Thin adapter; value in patterns, not library |
| esrs | `~/projects/rust-workspace/event_sourcing.rs` | Pure sync aggregates, Upcaster pattern | PostgreSQL-only; no SQLite backend |
| kameo_es | `~/projects/rust-workspace/kameo_es` | Actor + ES composition | Alpha maturity |
| SierraDB | `~/projects/rust-workspace/sierradb` | Distributed event store design | Pre-production; overkill for single-node |

### Patterns retained from these references (now implemented via fmodel-rust)

- Pure synchronous decision logic: fmodel's `Decider::decide` and `Decider::evolve`
- Event schema evolution: custom Upcaster pattern at deserialization layer
- TestFramework DSL: fmodel's `DeciderTestSpecification` with given/when/then
- SQLite event store schema: custom `EventRepository` implementation with global sequence for SSE Last-Event-ID

## Datastar ecosystem

The Datastar core developers are primarily Go developers, so the Go SDK and templates represent the most mature, fleshed-out examples of high-performance Datastar integration patterns.
When implementing ironstar, study the Go examples (especially northstar) as primary references for patterns like web component integration and SSE streaming architecture, then adapt them for Rust's type system and functional idioms.
Note: northstar uses Tailwind+DaisyUI while ironstar uses Open Props + Open Props UI, so CSS styling patterns will differ.

| Dependency | Local Path | Description |
|------------|------------|-------------|
| datastar | `~/projects/lakescope-workspace/datastar` | Main datastar repository (frontend JS + SDK specs) |
| datastar-doc | `~/projects/lakescope-workspace/datastar-doc` | Local markdown copy of datastar documentation |
| datastar-rust | `~/projects/rust-workspace/datastar-rust` | Rust SDK for SSE generation |
| datastar-rust-lince | `~/projects/rust-workspace/datastar-rust-lince` | Real-world usage example |
| datastar-go | `~/projects/lakescope-workspace/datastar-go` | Go SDK (reference implementation) |
| datastar-go-template-minimal | `~/projects/lakescope-workspace/datastar-go-template-minimal` | Minimal Go template (SSE basics without NATS) |
| northstar | `~/projects/lakescope-workspace/datastar-go-nats-template-northstar` | Full-featured Go template with NATS, Templ, web components |

**Canonical SDK specification**: `~/projects/lakescope-workspace/datastar/sdk/ADR.md`

All ironstar SSE implementations must conform to this specification.
Key types: `PatchElements`, `PatchSignals`, `ReadSignals<T>`.

## Alternative Datastar implementations (reference)

| Implementation | Local Path | Description |
|----------------|------------|-------------|
| http-nu | `~/projects/rust-workspace/http-nu` | Nushell-scriptable HTTP server with Datastar SDK |
| xs | `~/projects/rust-workspace/xs` | Event store with http-nu + Datastar examples |

These projects take a different architectural approach (Nushell scripting vs compiled Rust) but contain useful Datastar integration patterns and edge case handling.
See their TodoMVC implementations for SSE formatting and signal parsing patterns.

## Infrastructure and tooling

| Dependency | Local Path | Description |
|------------|------------|-------------|
| nix-cargo-crane | `~/projects/nix-workspace/nix-cargo-crane` | Crane library for Nix-based Rust builds (cargoNextest, cargoTest, cargoDocTest) |
| rust-flake | `~/projects/rust-workspace/rust-flake` | Flake module abstraction over crane with rust-overlay integration |
| cargo-nextest | `~/projects/rust-workspace/nextest` | Fast test runner with partitioning, JUnit output, retries |
| process-compose | `~/projects/nix-workspace/process-compose` | Process orchestration |
| process-compose-flake | `~/projects/nix-workspace/process-compose-flake` | Nix flake integration |
| rolldown | `~/projects/rust-workspace/rolldown` | Rust-native JS/CSS bundler |
| rollup-plugin-output-manifest | `~/projects/rust-workspace/rollup-plugin-output-manifest` | Rolldown manifest.json generation (maps entry names to hashed filenames) |
| zenoh | `~/projects/rust-workspace/zenoh` | Distributed pub/sub + storage (future) |

## CSS and styling

| Dependency | Local Path | Description |
|------------|------------|-------------|
| open-props | `~/projects/lakescope-workspace/open-props` | CSS design tokens library |
| open-props-ui | `~/projects/lakescope-workspace/open-props-ui` | Pure CSS component library (copy-paste model) |

## Reference implementations (alternative approaches)

| Dependency | Local Path | Description |
|------------|------------|-------------|
| maud | `~/projects/rust-workspace/maud` | Alternative HTML templating (eager evaluation) |
| askama | `~/projects/rust-workspace/askama` | Alternative HTML templating (file-based) |
| nats.rs | `~/projects/rust-workspace/nats.rs` | NATS Rust client (for reference, not used) |
| nats-server | `~/projects/lakescope-workspace/nats-server` | NATS server (Go, for reference) |

## Template references

| Template | Local Path | Description |
|----------|------------|-------------|
| rust-nix-template | `~/projects/rust-workspace/rust-nix-template` | Rust + Nix template pattern |
| typescript-nix-template | `~/projects/nix-workspace/typescript-nix-template` | TypeScript + Nix template pattern |
| python-nix-template | `~/projects/nix-workspace/python-nix-template` | Python + Nix template pattern |
| hypertext-typst-nix-youwen5-web | `~/projects/rust-workspace/hypertext-typst-nix-youwen5-web` | Hypertext + Nix pattern (uses Tailwind, not datastar) |

## Integration pattern references

| Pattern | Local Path | Description |
|---------|------------|-------------|
| rust-duckdb-huggingface-ducklake-query | `~/projects/rust-workspace/rust-duckdb-huggingface-ducklake-query` | DuckDB + DuckLake + HuggingFace query pattern (hf:// protocol) |
| marhar-frozen | `~/projects/omicslake-workspace/marhar-frozen` | DuckLake fixture data creation tools |
| marhar-duckdb-tools | `~/projects/omicslake-workspace/marhar-duckdb-tools` | DuckDB tooling for data lake operations |
| ducklake | `~/projects/lakescope-workspace/ducklake` | DuckDB extension for versioned data lake catalogs |
| sciexp-fixtures | `~/projects/omicslake-workspace/sciexp-fixtures` | DuckLake test fixtures on HuggingFace (`hf://datasets/sciexp/fixtures`) |

These patterns enable the axum backend to query remote datasets (HuggingFace Hub, S3-compatible storage) via DuckDB's httpfs extension, serving data for ECharts/Vega visualizations without local data ingestion.
See "Remote data sources via httpfs" in `docs/notes/architecture/core/architecture-decisions.md` section 6 for implementation details.

### Analytics test fixtures

The `sciexp-fixtures` repository provides the canonical DuckLake catalog for ironstar analytics development and testing.

| Property | Value |
|----------|-------|
| Local path | `~/projects/omicslake-workspace/sciexp-fixtures` |
| HuggingFace URL | `hf://datasets/sciexp/fixtures` |
| Catalog file | `lakes/frozen/space.db` (git-lfs, Xet-backed) |
| Query examples | `queries/space.sql` |

**Attachment pattern**:

```sql
INSTALL httpfs; INSTALL ducklake;
LOAD httpfs; LOAD ducklake;
ATTACH 'ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db' AS space;
SHOW TABLES FROM space.main;  -- astronauts, missions, mission_crew, spacecraft
```

The catalog contains absolute `hf://datasets/sciexp/fixtures/...` paths, so queries resolve data files from HuggingFace Hub.

Use this dataset for validating 9b1 (httpfs extension), c7z (DuckLake/hf:// integration), and 753.4 (ds-echarts backend).

## Visualization libraries

| Library | Local Path | Description |
|---------|------------|-------------|
| echarts | `~/projects/lakescope-workspace/echarts` | Apache ECharts (see northstar ds-echarts Lit component for reference) |
| vega-embed | `~/projects/lakescope-workspace/vega-embed` | Vega-Lite chart embedding (wrap in web component) |
| mosaic | `~/projects/lakescope-workspace/mosaic` | Grammar of graphics for large datasets (candidate for integration) |
