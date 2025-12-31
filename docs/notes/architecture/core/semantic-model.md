# Semantic model

This document describes the unified theoretical model of ironstar's data flow, grounding each architectural layer in algebraic structures from category theory.
The model serves as both a reference for implementation decisions and a pedagogical guide to how abstract mathematical concepts manifest in production systems.

For the theoretical foundations underlying these patterns, see the preference documents at `~/.claude/commands/preferences/theoretical-foundations.md`.

## Mathematical rigor

This document uses category theory as an organizational framework and source of intuition for ironstar's architecture.
The algebraic structures (monads, comonads, profunctors, coalgebras) provide precise vocabulary for describing data flow and composition properties.

Claims are grounded in established mathematical structures but presented at an accessible engineering level.
Formal proofs are omitted; the document aims for "informed by category theory" rather than "formally derived from it."
For complete categorical definitions, see the theoretical-foundations.md preference document.

## Overview

ironstar implements a **profunctor-structured architecture** where:

- Write side is Kleisli composition in the Result monad
- Event log is a free monoid providing the canonical source of truth
- State is reconstructed via catamorphism (unique fold from initiality)
- Read models are Galois-connected quotients enabling independent scaling
- SSE transport is a deterministic projection function
- Client signals are comonadic (dual to server-side monadic effects)
- Web components are coalgebraic Moore machines with bisimulation equivalence
- Analytics are quotients with memoized query profunctors

## The interpretation function

The semantic model defines an interpretation function mapping architectural descriptions to categorical structures:

```
⟦−⟧ : ArchitecturalDescription → Category
```

| Domain (syntax) | Codomain (semantics) |
|-----------------|----------------------|
| Component names ("event store", "projection") | Objects in a category |
| Data flow descriptions ("commands flow to events") | Morphisms |
| Composition patterns ("handler then persist") | Morphism composition |
| Architectural invariants ("append-only") | Algebraic laws (monoid identity, associativity) |

Concretely for ironstar:

| Architectural term | Denotation |
|--------------------|------------|
| `⟦event store⟧` | Free(Event) — the free monoid on Event |
| `⟦command handler⟧` | Kleisli arrow `A → T B` in the Result monad |
| `⟦fold_events⟧` | The unique catamorphism from initial algebra |
| `⟦projection⟧` | Upper adjoint in a Galois connection |
| `⟦SSE stream⟧` | Deterministic function `Event → Patch` |
| `⟦datastar signals⟧` | Comonad (extract + extend) |
| `⟦web component⟧` | Coalgebra for `F(S) = Output × (Input → S)` |
| `⟦whole system⟧` | Profunctor `P : Cmd^op × View → Set` |

The interpretation is *semantic* because it assigns mathematical meanings (denotations) to architectural descriptions (signs), enabling reasoning about system properties through the properties of the mathematical objects rather than the syntactic form of the description.

## The complete data flow

```
╔══════════════════════════════════════════════════════════════════════════════════╗
║                           COMMAND SIDE (Contravariant)                           ║
╠══════════════════════════════════════════════════════════════════════════════════╣
║                                                                                  ║
║  User Input ──(validate)──▶ Command ──(handle)──▶ Events ──(persist)──▶ EventStore
║      │                          │                    │                     │     ║
║  RawInput              TodoCommand            TodoEvent::Created      SQLite     ║
║  validation            QuerySessionCmd        QueryStarted            append     ║
║      │                          │                    │                 only      ║
║      └──── Kleisli arrows ──────┴──── Pure fn ───────┘                           ║
║             Result<T,E>              no I/O                                      ║
║                                                                                  ║
╠══════════════════════════════════════════════════════════════════════════════════╣
║                              EVENT LOG (Pivot Point)                             ║
║                                                                                  ║
║                              Free Monoid over Event                              ║
║                         [] = identity, ++ = composition                          ║
║                                                                                  ║
║                    global_sequence: total order for SSE replay                   ║
║                    aggregate_sequence: per-aggregate for OCC                     ║
║                                                                                  ║
╠══════════════════════════════════════════════════════════════════════════════════╣
║                            QUERY SIDE (Covariant)                                ║
╠══════════════════════════════════════════════════════════════════════════════════╣
║                                                                                  ║
║  EventStore ──(subscribe)──▶ Event Stream                                        ║
║      │                             │                                             ║
║      │                    ┌────────┼────────┐                                    ║
║      │                    ▼        ▼        ▼                                    ║
║      │               ReadModel  DuckDB   SSE Bus                                 ║
║      │                  │       Query      │                                     ║
║      │                  │         │        │                                     ║
║      │            (catamorphism)  │   (subscribe)                                ║
║      │                  │         │        │                                     ║
║      │                  ▼         ▼        ▼                                     ║
║      │              TodoMVC   Analytics  Zenoh/broadcast                         ║
║      │               State     Results     │                                     ║
║      │                  │         │        │                                     ║
║      │                  │    (memoize)     │                                     ║
║      │                  │         │        │                                     ║
║      │                  │      Moka        │                                     ║
║      │                  │      Cache       │                                     ║
║      │                  │         │        │                                     ║
║      └──(fold_events)───┴─────────┼────────┴── Galois Connection ──────────────  ║
║                                   │                                              ║
╠═══════════════════════════════════╪══════════════════════════════════════════════╣
║                          SSE TRANSPORT (Function)                                ║
║                                   │                                              ║
║             DomainEvent ────(project)────▶ PatchEvent                            ║
║                                   │                                              ║
║                    ┌──────────────┼──────────────┐                               ║
║                    ▼              ▼              ▼                               ║
║             PatchElements   PatchSignals   ExecuteScript                         ║
║                HTML            JSON            JS                                ║
║                    │              │              │                               ║
║                    └──────────────┼──────────────┘                               ║
║                                   │                                              ║
║                           DatastarEvent                                          ║
║                        (canonical repr)                                          ║
║                                   │                                              ║
║                              axum SSE                                            ║
║                                   │                                              ║
╠═══════════════════════════════════╪══════════════════════════════════════════════╣
║                          CLIENT (Comonad + Coalgebra)                            ║
║                                   │                                              ║
║                                   ▼                                              ║
║                           Datastar Signals ◀──── Comonad                         ║
║                           $todoList, $queryStatus                                ║
║                                   │                                              ║
║                    extract: get current value                                    ║
║                    extend: derive computed signals                               ║
║                                   │                                              ║
║                    ┌──────────────┼──────────────┐                               ║
║                    ▼              ▼              ▼                               ║
║               data-text      data-show      data-attr:option                     ║
║                    │              │              │                               ║
║                    └──────────────┼──────────────┘                               ║
║                                   │                                              ║
║                                   ▼                                              ║
║                          Web Components ◀──── Coalgebra (Moore machine)          ║
║                           ds-echarts                                             ║
║                                   │                                              ║
║                    state → render → DOM                                          ║
║                    input → transition → state'                                   ║
║                                   │                                              ║
║                                   ▼                                              ║
║                                 DOM                                              ║
║                                   │                                              ║
║                           User sees UI                                           ║
║                                   │                                              ║
║                    (chart-click, form submit)                                    ║
║                                   │                                              ║
║                                   └─────────────────────▶ Back to Command Side   ║
║                                                                                  ║
╚══════════════════════════════════════════════════════════════════════════════════╝
```

## Layer-by-layer algebraic structure

### Command handling as Kleisli composition

The write path is a sequence of Kleisli arrows in the `Result` monad:

```
validate : RawInput → Result<Command, ValidationError>
handle   : (State, Command) → Result<Vec<Event>, DomainError>
persist  : Vec<Event> → Result<GlobalSequence, InfraError>
```

In the `Aggregate` trait:

```rust
fn handle_command(state: &Self::State, cmd: Self::Command)
    -> Result<Vec<Self::Event>, Self::Error>;
```

This is a pure function with no I/O and no async.
The Kleisli composition ensures errors short-circuit correctly (railway-oriented programming).

### Event log as free monoid

The event store is the free monoid over domain event types:

| Property | Implementation |
|----------|----------------|
| Identity | `[]` (empty event sequence) |
| Binary operation | `++` (event concatenation) |
| Generators | `TodoEvent`, `QuerySessionEvent` variants |

SQLite enforces the monoid structure:
- `global_sequence` (AUTOINCREMENT) ensures total ordering
- `aggregate_sequence` enables per-aggregate versioning for optimistic concurrency
- Append-only semantics: no updates or deletes (except compaction)

The initiality of the free monoid guarantees that for any monoid homomorphism `h : Event → M`, there exists a unique fold `fold_h : [Event] → M`.

### State reconstruction as catamorphism

The `fold_events` function is the unique catamorphism from the initial algebra:

```rust
fn fold_events(events: impl IntoIterator<Item = Self::Event>) -> Self::State {
    events.into_iter().fold(Self::State::default(), Self::apply_event)
}
```

Given the `apply_event` algebra, there is exactly one correct way to reconstruct state.
This is why event sourcing enables deterministic replay.

Uniqueness requires `apply_event` to be a pure, deterministic function with no side effects.
Non-determinism (time-dependent logic, randomness, I/O) breaks the initiality guarantee.

Event schema evolution via Upcasters preserves catamorphism uniqueness only when each Upcaster is a monoid homomorphism: `upcast(∅) = ∅` and `upcast(e₁ ++ e₂) = upcast(e₁) ++ upcast(e₂)`.

### Projections as abstraction-concretion pairs

The relationship between event log and read models forms an abstraction-concretion pair:

```
abstract : EventLog → ReadModel    (projection, lossy)
concrete : ReadModel → EventLog    (reconstruction, partial)
```

For ironstar:
- `TodoMVC` read model: current todo list state
- `QuerySessionStatus`: execution state machine position
- DuckDB analytics: aggregated OLAP views

This pair exhibits closure properties rather than a strict Galois connection:

```
abstract ∘ concrete ∘ abstract = abstract  (projection is idempotent)
concrete ∘ abstract ∘ concrete = concrete  (reconstruction is stable)
```

The relationship is weaker than a formal Galois connection because projection is lossy: multiple event sequences map to the same read model.
This many-to-one property explains why read models are disposable—they can always be rebuilt from events.

The key insight is not the categorical adjunction but the *reconstruction guarantee*: given any read model state, there exists at least one event sequence that produces it.
This enables event log compaction, snapshotting, and parallel projection without loss of information.

### Read models as quotients

Each materialized view is a quotient monoid—the free monoid modulo an equivalence relation:

```
EventLog/≡ ≅ ReadModel
```

Where `e1 ≡ e2` iff `project(e1) = project(e2)`.

For the todo list:
- `[Created{id:1}, Completed{id:1}]` and `[Created{id:1}, Completed{id:1}, Uncompleted{id:1}, Completed{id:1}]` are equivalent
- Both produce the same final state: todo #1 is completed

This quotient structure enables:
- Log compaction (remove redundant events)
- Snapshots (store quotient representatives)
- Parallel projection (commutative events can be processed concurrently)

### SSE streaming as projection function

The transformation from domain events to transport events is a deterministic function `F: Event → Patch`.
Each event type maps to exactly one patch type:

```
F(TodoCreated{...})    = PatchElements { selector: "#todos", html: "..." }
F(QueryCompleted{...}) = PatchSignals { signals: {"status": "completed"} }
```

This function is well-defined (total, deterministic) and preserves event identity (no-op events produce no patches).

In datastar-rust, the `DatastarEvent` is the canonical representation that all specific types convert into:

```rust
impl From<PatchElements> for DatastarEvent { ... }
impl From<PatchSignals> for DatastarEvent { ... }
```

### DuckDB analytics as quotient with memoization

DuckDB queries produce quotients of source data:
- SQL `GROUP BY` defines equivalence classes
- Aggregations like `SUM` and `COUNT` are monoid homomorphisms on those classes.
Note that `AVG` is *not* a monoid homomorphism because averaging does not distribute over concatenation: `avg(xs ++ ys) ≠ (avg(xs) + avg(ys)) / 2` in general.
- The result set is the quotient

Moka caching implements memoization over the query profunctor:

```
Cache : (Query, DatasetRef, ChartConfig) → Option<Result>
```

Cache invalidation corresponds to naturality failure.
When an update affects data relevant to a query, the cached result becomes invalid.
ironstar uses TTL (5 minutes) as a conservative invalidation strategy.

### Client signals as comonad

Datastar signals exhibit comonadic structure:

```typescript
// Comonad operations
extract : Signal a → a                        // $signal.value
extend  : (Signal a → b) → Signal a → Signal b // computed(() => ...)
```

Comonad laws:

```
extend extract = id                           -- extracting then extending is identity
extract ∘ extend f = f                       -- extending then extracting gives f
extend f ∘ extend g = extend (f ∘ extend g)  -- extension composes
```

This is the categorical dual of monads:
- Monads: effect production (server-side event sourcing)
- Comonads: context consumption (client-side signal derivation)

The duality manifests architecturally:
- Server produces events via monadic Kleisli composition
- Client consumes updates via comonadic signal derivation

### Web components as coalgebras (Moore machines)

Lit components like `ds-echarts` are Moore machines:

```typescript
// Coalgebra for functor F S = Output × (Input → S)
coalgebra : State → (Output, Input → State)
coalgebra(state) = (render(state), (input) => transition(state, input))
```

Where:
- State: `{ option, theme, chart instance, event handlers }`
- Output: Rendered canvas/SVG via `render()`
- Input: Attribute changes (`option`, `theme`) and chart events (`click`, `datazoom`)
- Transition: `updated()` lifecycle and event handlers

Bisimulation defines behavioral equivalence: two component states are equivalent if they produce the same output and transition to equivalent states on all inputs.
This is why morphing boundaries work—bisimilar states can be safely swapped.

### The complete profunctor

The entire system is a profunctor `P : Command^op × View → Set`:

```
P(cmd, view) = { data flows from cmd to view via event log }
```

Contravariant in commands (input transformations compose contravariantly):

```rust
let adapted_cmd = adapter(raw_input);  // contravariant
let events = handle(state, adapted_cmd);
```

Covariant in views (output transformations compose covariantly):

```rust
let patch = project(event);
let enhanced_view = transform(patch);  // covariant
```

The event log acts as the mediating data structure between the command-side (contravariant in commands) and query-side (covariant in views).
It is not a natural transformation in the categorical sense but rather the pivot point through which all information flows—the shared interface that enables independent scaling of the two sides.
This explains CQRS's core insight: the two sides can scale independently because they only share the event log interface.

## Temporal structure

The system has bitemporal semantics:

| Time Axis | Implementation | Purpose |
|-----------|----------------|---------|
| Event time | `created_at`, `completed_at` in events | When domain event occurred |
| Processing time | `global_sequence` in SQLite | When event was persisted |
| Table version time | DuckLake snapshots (if enabled) | When analytics snapshot was taken |

SSE reconnection uses `global_sequence` as `Last-Event-ID`:
- Client sends last received ID
- Server replays events since that sequence
- Monotonic sequences with no gaps ensure no missed updates

## The spawn-after-persist pattern

The `QuerySession` aggregate demonstrates effect ordering:

1. User submits SQL query
2. StartQuery command validated
3. QueryStarted event persisted to SQLite
4. Then async DuckDB task spawned (spawn-after-persist)
5. Task completion emits CompleteQuery/FailQuery
6. Those events persisted, streamed via SSE

This is crucial for deterministic replay:
- The event log records QueryStarted before execution begins
- If server crashes mid-execution, replay sees QueryStarted but no completion
- Recovery logic can detect incomplete queries and retry

## Composition laws summary

| Structure | Laws | Consequence |
|-----------|------|-------------|
| Kleisli (command) | Left/right identity, associativity | Commands compose, errors propagate |
| Free monoid (events) | Associativity, identity | Events can be batched, replayed |
| Catamorphism (fold) | Uniqueness from initiality | Deterministic state reconstruction |
| Galois (projection) | Adjunction properties | Read models are rebuildable |
| Function (SSE) | Total, deterministic | Patches are independent |
| Comonad (signals) | Extract/extend laws | Derived signals compose correctly |
| Coalgebra (components) | Bisimulation equivalence | Morphing preserves behavior |

## Integration map

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ironstar Integration                        │
├──────────────────┬──────────────────┬───────────────────────────────┤
│    Domain        │   Application    │      Infrastructure           │
├──────────────────┼──────────────────┼───────────────────────────────┤
│ Aggregate trait  │ Command handlers │ SQLite event store            │
│ TodoAggregate    │ Query handlers   │ Moka cache                    │
│ QuerySession     │ Projections      │ Zenoh event bus               │
│ Smart ctors      │ SSE streaming    │ DuckDB analytics              │
│                  │                  │ axum HTTP                     │
├──────────────────┴──────────────────┴───────────────────────────────┤
│                         Presentation                                │
├─────────────────────────────────────────────────────────────────────┤
│ hypertext templates │ Datastar signals │ ds-echarts component        │
│ ts-rs bindings      │ PatchElements    │ Open Props CSS              │
└─────────────────────────────────────────────────────────────────────┘
```

## Sessions as indexed profunctor

The semantic model above treats commands and events as globally visible.
In practice, ironstar partitions event delivery via sessions, creating an indexed family of projections.

### Session structure

Sessions do not modify the source-of-truth event log—they partition the pub/sub delivery layer:

```
EventLog (global, authoritative)
    ↓ persist (all events)
SQLite events table
    ↓ publish to Zenoh
Key expression: events/{aggregate_type}/{aggregate_id}
    ↓ filter by session subscription
events/session/{session_id}/**
    ↓ per-session SSE stream
Client receives only subscribed events
```

### Indexed profunctor interpretation

The profunctor `P: Command^op × View → Set` extends to an indexed family:

```
P_s : Command^op × View → Set    (for each session s)
P_s(cmd, view) = { data flows visible to session s }
```

Where:
- Commands are session-agnostic (global write path)
- Event log remains the single source of truth
- Zenoh key expressions implement the indexing: `events/session/{s}/**`
- Each session receives a filtered projection of the event stream

### What sessions are NOT

Sessions do not:
- Affect the free monoid structure (events are global)
- Create per-session event logs (single log, filtered views)
- Modify the catamorphism (state reconstruction is global)
- Change the Galois connection (projections derive from full log)

Sessions are a delivery-layer optimization, not a semantic-layer modification.
The algebraic properties (monoid laws, catamorphism uniqueness, adjunction) hold on the global event log; sessions filter which events reach which clients.

### Session state vs domain state

Session state (preferences, auth context) is orthogonal to event-sourced domain state:

| Aspect | Domain State | Session State |
|--------|--------------|---------------|
| Storage | Event log (append-only) | Sessions table (mutable) |
| Lifetime | Permanent | Ephemeral (TTL) |
| Reconstruction | Catamorphism from events | Direct lookup |
| Consistency | Strong (total order) | Eventual (per-session) |

Session data enables personalization without polluting the event stream.

## Related documents

### Within ironstar

- `design-principles.md` — foundational philosophy and coding standards
- `architecture-decisions.md` — technology choices and rationale
- `event-sourcing-core.md` — detailed event store implementation
- `projection-patterns.md` — read model derivation patterns
- `signal-contracts.md` — Datastar type generation via ts-rs
- `ds-echarts-integration-guide.md` — web component patterns
- `analytics-cache-architecture.md` — DuckDB and moka caching

### Preference documents (via ~/.claude/commands/preferences/)

- `theoretical-foundations.md` — category theory foundations (comonads, coalgebras, Galois connections)
- `distributed-systems.md` — event sourcing, CQRS, idempotency
- `domain-modeling.md` — aggregates, smart constructors, state machines
- `hypermedia-development/07-event-architecture.md` — SSE as projection channel
- `hypermedia-development/03-datastar.md` — signal system patterns
- `hypermedia-development/05-web-components.md` — coalgebra interpretation
- `data-modeling.md` — materialized views, query caching
- `rust-development/` — Rust-specific patterns

## Version history

| Date | Change |
|------|--------|
| 2025-12-29 | Rename from "Algebraic architecture" to "Semantic model"; add interpretation function tables |
| 2025-12-29 | Initial synthesis from preference documents and codebase analysis |
