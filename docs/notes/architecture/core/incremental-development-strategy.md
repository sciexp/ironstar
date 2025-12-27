---
title: Incremental development strategy
---

# Incremental development strategy

This document addresses the layer-by-layer blocking problem in the beads issue graph by distinguishing real architectural dependencies from artificial sequential ordering.

## Problem statement

The current issue graph enforces waterfall dependencies where Domain → Event Sourcing → Presentation → Example must complete sequentially.
This is problematic because many components are architecturally independent, yet the ordering implies they are not.

The core issue: Early phases contain too many orthogonal concerns bundled together, creating artificial blocking points.
A developer cannot make progress on frontend tooling while waiting for domain types to stabilize, even though these are completely independent.

## Real vs artificial dependencies

### Genuinely sequential dependencies

These MUST be done in order because they build on each other:

1. **Domain types** → Define aggregate, events, commands with precise signatures
2. **EventStore trait** → Abstract interface for persistence (depends on Event types)
3. **Projection trait** → Abstract interface for read models (depends on domain types)
4. **Command handler pattern** → POST → Command → Aggregate → Events (depends on all above)
5. **Query handler pattern** → GET → Query → Projection (depends on Projection trait)

However, these can progress with **stubs and contracts** for unfinished implementations.
The trait is the interface; the implementation can follow.

### Genuinely parallel components

These can be developed independently with clear interfaces:

1. **Frontend bundler configuration** — Requires only TypeScript config, CSS token setup, build output directory structure, manifest.json format. Zero backend dependency. Can be tested with dummy HTML.

2. **SQLite schema and migrations** — Requires only event table schema, session table schema, and migration tooling. Can be tested independently with `sqlite3 < migrations/schema.sql` and schema validation.

3. **Domain aggregate models** — Pure Rust types with no async effects. Unit testable with `proptest` and reference types. Compiles independently.

4. **Presentation layer templates** — Only depend on domain types (concrete) and trait contracts for rendering. Hypertext template compilation and structure tests are independent.

### Conditionally dependent components

These depend on *trait interfaces* not *specific implementations*:

- **Event replay and SSE stream** depends on `EventStore` trait (abstract)
- **Command handlers** depend on domain types (concrete) and `AppState` trait (abstract)
- **Query handlers** depend on `Projection` trait (abstract)
- **Full integration test** depends on implementations (concrete)

The key insight: you can write command handler code against the `EventStore` trait before the SQLite implementation exists.
The test uses a mock, but the production implementation swaps in at the boundary.

## Proposed development phases

### Phase 1: Core abstractions and type contracts

Build the interfaces and domain types that everything else depends on.
This phase produces no executable code—only trait definitions and type signatures.

**Deliverables:**

| Task | Scope | Validation |
|------|-------|------------|
| Define `Aggregate` trait | `version(&self) -> u64`, version field, aggregate ID type | Trait compiles, docs examples |
| Define `Event` enum for chosen aggregate (e.g., Todo) | All event variants for the aggregate | Serialize/deserialize with serde_json in test |
| Define `Command` enum for aggregate | All command variants | Serialize/deserialize with serde_json in test |
| Create `EventStore` trait with stub | `append_events`, `load_events`, `get_snapshot` | Test compiles with mock impl using `mockall` |
| Create `Projection` trait with stub | `handle_event`, `rebuild` | Test compiles with mock impl |
| Create `AppState` trait | Connection pool, event bus, cache accessors | Trait compiles, documents what services are available |

**Dependencies:**

- None (Phase 1 is the foundation)

**Exit criteria:**

- `cargo check` passes
- All traits have at least one mock implementation that compiles
- `cargo doc --open` produces readable trait documentation
- No `#[allow(dead_code)]` or `#[allow(unused)]` needed

**Effort estimate:** 2–3 days

---

### Phase 2A: Frontend bundler setup (parallel)

Configure Rolldown, PostCSS, Open Props tokens, and TypeScript.
This phase is completely independent and can start immediately after Phase 1.

**Deliverables:**

| Task | Scope | Validation |
|------|-------|------------|
| Configure Rolldown entry point | Point to `web/index.ts`, output to `static/dist/` with manifest.json | `rolldown` builds without errors, manifest.json exists |
| Add Open Props to PostCSS pipeline | Import Open Props tokens, compile CSS tokens | PostCSS produces usable CSS with `--` custom properties |
| Create base TypeScript skeleton | Import statements, datastar type stubs, Web Component registration | `tsc` type-checks successfully |
| Create manifest.json format test | Test that bundle output creates valid manifest | Integration test parses manifest, finds expected keys |
| Integrate into dev/prod flow | Dev mode: `tower-http::ServeDir` static, Prod mode: embed static/dist/ | `cargo build` succeeds, dev mode serves files |

**Dependencies:**

- None (completely independent)

**Exit criteria:**

- `just build-frontend` produces output in `static/dist/`
- `manifest.json` has correct format
- Dev server can serve static assets
- `cargo build --release` produces binary with embedded assets

**Effort estimate:** 2–3 days

---

### Phase 2B: Domain aggregate implementation (parallel)

Implement the aggregate state machine, command validation, and event application.
Pure Rust types with no async.

**Deliverables:**

| Task | Scope | Validation |
|------|-------|------------|
| Implement aggregate apply logic | `impl Aggregate for Todo`: load state from events, apply new event | Unit tests with reference events |
| Implement command validation | Validate command preconditions (e.g., can't mark complete if not created) | Unit tests for each validation rule |
| Implement event application | Each event variant updates aggregate state immutably | Serialization/deserialization round-trip tests |
| Add event upcasting pattern | Support schema versioning for events (if evolving schema) | Test upcaster converts old→new events |
| Add aggregate snapshot pattern (optional) | Snapshot every N events for faster replay | Test snapshot + delta load equals full replay |

**Dependencies:**

- Phase 1 (types and traits)

**Exit criteria:**

- `cargo test --lib domain::*` passes with >90% coverage
- All aggregate operations are pure functions (no side effects)
- `proptest` fuzzing validates invariants (e.g., version always increments)

**Effort estimate:** 3–4 days

---

### Phase 2C: SQLite event store implementation (parallel)

Implement the EventStore trait using SQLite and sqlx.
Can proceed as soon as Phase 1 defines the trait.

**Deliverables:**

| Task | Scope | Validation |
|------|-------|------------|
| Create event store schema | Primary key, aggregate ID + sequence compound index, JSON payload column | `sqlite3 < migrations/001_event_store.sql` runs without error |
| Implement `EventStore::append_events` | Insert events atomically, check optimistic locking (aggregate version) | Integration test appends, retrieves, validates version |
| Implement `EventStore::load_events` | SELECT by aggregate ID, deserialize JSON, order by sequence | Integration test loads events, verifies order |
| Implement `EventStore::get_snapshot` | Snapshot table with version + state JSON, return if recent | Integration test snapshot speeds up replay |
| Add event bus publisher | After appending events, publish to Zenoh | Integration test appends event, verifies broadcast |
| Add Last-Event-ID tracking | Store global sequence ID for SSE reconnection | Integration test reads last ID, validates monotonic |

**Dependencies:**

- Phase 1 (Event type)

**Exit criteria:**

- `cargo test --test '*event_store'` passes
- All operations are transactional (append + publish is atomic)
- Last-Event-ID is monotonically increasing

**Effort estimate:** 3–4 days

---

### Phase 3: Presentation layer templates

Implement hypertext rendering templates and DatastarRequest extractor.
Depends only on domain types and trait contracts.

**Deliverables:**

| Task | Scope | Validation |
|------|-------|------------|
| Implement `DatastarRequest` extractor | Parse `datastar-events` header, extract connection ID | Unit test with mock headers |
| Create base layout template | HTML boilerplate, Open Props CSS includes, datastar.js link | Hypertext compiles, produces valid HTML |
| Implement `RenderableToDatastar` trait | Render domain types to HTML + signal updates | Test renders aggregate state to HTML string |
| Create Todo list presentation | Render aggregate state as HTML list items | Template renders to valid HTML |
| Create signal contract file | TypeScript types for Todo signals (id, title, completed) | `tsc` type-checks against signal types |

**Dependencies:**

- Phase 1 (domain types)
- Phase 2B (aggregate state structure)

**Exit criteria:**

- `cargo test --lib presentation::*` passes
- Hypertext templates compile without runtime errors
- TypeScript signal types match Rust domain types (validated with ts-rs)

**Effort estimate:** 2–3 days

---

### Phase 4: Integration with real architecture

Wire all components together for a working example with REAL architecture.
This is the first point where all layers connect.

**Deliverables:**

| Task | Scope | Validation |
|------|-------|------------|
| Implement command handler | POST handler validates command, calls aggregate, appends to EventStore, returns rendered HTML | Integration test: POST command, verify event in store, verify SSE signal |
| Implement SSE feed endpoint | GET /sse: subscribe, send existing projection, stream updates from Zenoh | Integration test: open SSE, check Last-Event-ID, receive updates |
| Implement projection manager | Listen on Zenoh events, rebuild projection state incrementally | Integration test: append event, verify projection updates within 100ms |
| Implement GET query handler | GET /{resource}: read from projection, render to HTML | Integration test: query returns current aggregate state |
| Full end-to-end test | Browser simulation: POST todo, receive update via SSE, verify DOM would update | Integration test: command → event → projection → SSE → datastar update |

**Dependencies:**

- Phase 1 (traits)
- Phase 2A (frontend build pipeline)
- Phase 2B (domain aggregates)
- Phase 2C (SQLite event store)
- Phase 3 (presentation layer)

**Exit criteria:**

- `cargo test --test '*integration'` passes
- Events persist to SQLite
- SSE clients receive updates within 200ms
- Full CQRS pipeline is real (not simulated)

**Effort estimate:** 4–5 days

---

### Phase 5: Example application

With patterns proven in Phase 4, additional examples (TodoMVC, Blog, etc.) follow the same architecture.
This phase is optional; Phase 4 proves the patterns work.

**Deliverables:**

| Task | Scope | Validation |
|------|-------|------------|
| Implement complete TodoMVC example | CRUD for todos with completion, deletion, filtering | Full CQRS pipeline with persistence |
| Add visualization example | ECharts or Vega-Lite chart embedded as Web Component | Backend queries DuckDB, frontend displays interactive chart |
| Deploy example | Build release binary, verify embedded assets, test in browser | Binary size <50MB, asset serving works |

**Dependencies:**

- Phase 4 (working architecture)

**Exit criteria:**

- Examples demonstrate full feature set
- Real use cases validate architectural patterns

**Effort estimate:** 3–4 days

---

## Validation testing by phase

Each phase has **validation tests** that demonstrate architectural correctness before proceeding.
This ensures the architecture is sound without requiring everything to be complete.

| Phase | Validation Test | Purpose |
|-------|-----------------|---------|
| 1 | Mock implementations compile | Trait contracts are coherent |
| 2A | Frontend build produces manifest.json | Build pipeline works end-to-end |
| 2B | `proptest` fuzzing of aggregates | Invariants hold under all inputs |
| 2C | EventStore integration test with real SQLite | Persistence works, versioning is correct |
| 3 | Hypertext template compiles and renders | Presentation is type-safe |
| 4 | Full CQRS pipeline with real (not mocked) services | Architecture is proven, not theoretical |
| 5 | Browser loads example, submits command, receives update | End-to-end works in production |

## Key principle: dependencies are architectural, not waterfall

**Preserve these true dependencies:**

- Domain types must be defined (Phase 1) before anything uses them
- EventStore trait must be defined (Phase 1) before implementations (Phase 2C)
- Aggregate implementation (Phase 2B) must exist before command handlers (Phase 4)
- Command handlers (Phase 4) must exist before full integration (Phase 4)

**Parallelize these independent paths:**

- **Phase 2A** (frontend bundler): completely independent, can start immediately after Phase 1 types are defined
- **Phase 2B** (domain aggregates): independent, can progress in parallel with Phase 2A and 2C
- **Phase 2C** (SQLite implementation): independent of presentation, can progress in parallel with Phase 2A and 2B

**Result:** Instead of 5 sequential phases (22 days minimum), three parallel tracks complete in ~12 days:

- Phases 1 + 2A (frontend) = 5–6 days
- Phases 1 + 2B (domain) = 5–7 days
- Phases 1 + 2C (SQLite) = 5–7 days
- Phase 3 (presentation, after 2B) = 2–3 days
- Phase 4 (integration, after all above) = 4–5 days
- Phase 5 (examples, optional) = 3–4 days

**Total with parallelism: ~14–17 days** vs **22+ days with pure waterfall**.

## Addressing the "tracer bullet" concern

A true tracer bullet would build end-to-end with simplified implementations everywhere (in-memory aggregates, mock event store, hardcoded HTML).
This document proposes the opposite: **build each layer with REAL architecture from the start**.

- Real EventStore uses SQLite, not HashMap
- Real aggregates use the domain model, not toy state
- Real presentation uses hypertext and datastar, not string templates
- Real integration uses Zenoh, not tokio::broadcast

The goal is to discover architectural problems early, not to defer them to a mythical "refactor later" phase.
By validating each phase independently before proceeding, we ensure the real architecture works at each step.
