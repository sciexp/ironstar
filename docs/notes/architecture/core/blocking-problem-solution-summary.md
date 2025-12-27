# Blocking problem solution summary

## The problem

The ironstar beads issue graph exhibited a waterfall dependency structure where:

- **93 issues marked as blocked** (55% of 150 open issues)
- **57 issues marked as ready** (38% of 150 open issues)
- **Tracer Bullet MVP (ironstar-v72)** artificially blocked development of Domain, Event Sourcing, Presentation, and Frontend layers

This prevented parallel development and created false sequential ordering.

## Root cause

The Tracer Bullet was designed to:
1. Prove the end-to-end path from frontend to backend
2. Validate SSE streaming works
3. Test Datastar signal integration

But it became:
1. A sequential gate blocking all other work
2. An incentive to use shortcuts (Arc<AtomicI64>) instead of real architecture
3. A blocker on work that had zero architectural dependency on it

## The solution approach

The solution identifies which dependencies are **genuine architectural constraints** vs. **artificial sequential ordering**.

### Genuine architectural constraints (must preserve)

1. **Domain types are foundational** — Everything else depends on knowing what Commands and Events look like
2. **Domain types → Event sourcing traits** — Event store must know the shape of events
3. **Domain types → Presentation templates** — Templates must know what types to display
4. **EventStore trait → Projection trait → SSE endpoints** — Read models depend on abstract persistence
5. **All layers → Tracer Bullet** — Tracer proves the whole stack works together

### False sequential ordering (can be parallelized)

1. ~~Frontend build pipeline depends on tracer bullet~~ → Frontend is completely independent
2. ~~Event sourcing implementation depends on tracer bullet~~ → Event sourcing is foundational
3. ~~Presentation layer depends on event sourcing implementation~~ → Presentation only depends on traits
4. ~~Domain types depend on tracer bullet~~ → Domain is foundational
5. ~~Example app depends on tracer bullet being complete~~ → Example only needs pattern proof

## The proposed solution

**6-phase incremental development with real architecture (no shortcuts):**

| Phase | Duration | Work | Validation | Key Insight |
|-------|----------|------|-----------|--|
| **1** | Week 1 | Domain types + traits (EventStore, Projection) | Unit + trait tests compile | Foundational types enable all other work |
| **2** | Week 1.5-2 | Frontend bundler + Domain aggregates in parallel | Bundler produces output + unit tests | Frontend has zero backend dependency |
| **3** | Week 2-2.5 | SQLite event store + Projection managers | Integration tests with real database | Event persistence validated independently |
| **4** | Week 2.5-3 | Presentation extractors + templates | Handler tests + template rendering | Presentation layer works with trait abstractions |
| **5** | Week 3 | Counter with REAL architecture | E2E tests + browser verification | Tracer proves actual CQRS (not fake Arc<Atomic>) |
| **6** | Week 3.5 | Todo example | Same tests as Counter | Pattern proven, example is straightforward |

## What makes this work

### 1. No fake tracer bullet

**Wrong approach (Arc<AtomicI64>):**
```rust
// Doesn't prove architecture, prevents learning real patterns
pub struct Counter {
    value: Arc<AtomicI64>,
}
```

**Correct approach (Real event sourcing):**
```rust
// Proves the actual architecture: Commands → Events → SQLite → Projection → SSE
pub struct Counter {
    version: u64,
    value: u64,  // Rebuilt from events during replay
}

// Commands are validated, emit events (pure function)
fn handle(&self, cmd: CounterCommand) -> Result<Vec<CounterEvent>, Error>

// Events are persisted to SQLite
event_store.append_events("counter", id, events).await?

// Projection rebuilds from event log
let counter = load_events().await?.fold(Counter::new(), |mut c, e| {
    c.apply(&e); c
})

// SSE streams deltas to frontend
SSE::new(projection_stream).send(signals).await
```

### 2. Parallel independent tracks

**Frontend team:**
- Implements Rolldown bundler
- Configures PostCSS and Open Props
- Sets up TypeScript
- Zero dependency on backend work

**Domain team:**
- Defines Counter aggregate
- Writes domain unit tests
- Only depends on closed workspace setup
- Done by end of Week 1

**Infrastructure team:**
- Creates SQLite schema
- Implements EventStore trait
- Implements ProjectionManager trait
- Runs integration tests with real database

**Presentation team:**
- Implements extractors (DatastarRequest)
- Creates templates (hypertext)
- Wires components together
- No dependency on implementation details, only traits

**Integration team (Phase 5-6):**
- Combines all pieces for Counter
- Validates end-to-end
- Replicates pattern for Todo

### 3. Real validation at each milestone

**Phase 1:** Domain types compile, traits exist
```bash
cargo check
cargo test --lib
# TypeScript types generate correctly
```

**Phase 3:** Events persist to SQLite, projections rebuild
```bash
cargo test --test '*event_store'
cargo test --test '*projection'
# Verify SQLite file has events
sqlite3 data.db "SELECT COUNT(*) FROM events"
```

**Phase 5:** Counter works end-to-end with real CQRS
```bash
cargo test --test '*counter_e2e'
# Manual browser test
# Start server, click buttons, verify:
# 1. Counter updates in browser
# 2. Page refresh shows persisted value
# 3. SQLite file has event log
```

### 4. Scope reduction (not shortcutting)

The solution doesn't fake the stack; it reduces scope while keeping the real architecture:

**What Counter DOES prove:**
- Commands can be structured as enums
- Events serialize/deserialize as JSON
- Aggregates handle commands synchronously
- Events persist durably to SQLite
- Projections rebuild deterministically
- SSE streams projection deltas
- Datastar signals update the frontend
- Page refresh shows persisted values

**What Counter DOESN'T include yet:**
- Authentication (comes later)
- Zenoh event bus (SQLite event log + tokio::broadcast first)
- Advanced error handling (basic Result types first)
- Observability (structured logging later)
- Cache invalidation (in-memory projections first)
- DuckDB analytics (out of scope)

**Todo does include:**
- Everything Counter does
- More complex aggregate state (list of items)
- Multiple command types (create/mark/delete)
- List projection instead of single value
- Same tests, same patterns

## Why previous approach failed

The Arc<AtomicI64> "tracer bullet":
1. **Didn't prove any real architecture** — No SQLite, no events, no projections, no event sourcing
2. **Created a false gate** — Made everything block on a task that didn't validate the real system
3. **Incentivized shortcuts** — Made it tempting to build on fake state instead of real architecture
4. **Didn't reduce risk** — Completing it didn't reduce risk of downstream work; all real problems remained
5. **Couldn't scale** — Patterns didn't transfer to Todo; Counter's approach couldn't be replicated

## Why the new approach works

1. **Validates real architecture** — Counter proves Commands → Events → Persistence → Projection → SSE
2. **Enables parallel work** — Frontend, domain, infra teams work independently on genuinely separate concerns
3. **Reduces scope without faking** — Fewer features (no auth, no analytics) but all are real
4. **Clear validation tests** — Each phase has concrete passing/failing criteria
5. **Patterns transfer** — Todo reuses Counter's architecture; pattern proven
6. **Git history clarity** — Each phase is a coherent, testable unit of work
7. **Risk reduction** — Each phase validates actual architectural decisions

## Concrete next steps

### Immediate (start Phase 1)

```bash
# 1. Review domain type tasks
bd list --parent ironstar-2nt --status open --priority 0

# 2. Start Phase 1 work
bd update ironstar-2nt.2 --status in_progress
# Implement Counter aggregate with handle() and apply() methods

bd update ironstar-2nt.3 --status in_progress
# Implement value objects and smart constructors

bd update ironstar-2nt.5 --status in_progress
# Create Datastar signal types with ts-rs

bd update ironstar-nyp.2 --status in_progress
# Create EventStore trait

bd update ironstar-nyp.6 --status in_progress
# Create Projection trait

# 3. Validate Phase 1
cargo check
cargo test --lib
# cargo run -- --generate-ts (for ts-rs)
```

### After Phase 1 (Week 1 end)

```bash
# Close Phase 1 tasks
bd close ironstar-2nt.2 --comment "Counter domain with handle()/apply()"
bd close ironstar-2nt.3 --comment "Value objects with validation"
bd close ironstar-2nt.5 --comment "Datastar signal types generated"
bd close ironstar-nyp.2 --comment "EventStore trait abstraction"
bd close ironstar-nyp.6 --comment "Projection trait abstraction"

# Check readiness for Phase 2
bd status
# Should show: Ready count increased, Blocked count decreased

# Start frontend and domain parallel work
bd update ironstar-ny3.2 --status in_progress  # Rolldown config
bd update ironstar-2nt.4 --status in_progress  # Aggregate state machines
```

### Phase 3: SQLite integration

```bash
# After Phase 2 complete
bd update ironstar-nyp.1 --status in_progress  # Schema
bd update ironstar-nyp.3 --status in_progress  # SQLite impl
bd update ironstar-nyp.7 --status in_progress  # Projections

# Validate
cargo test --test '*event_store'
cargo test --test '*projection'

# Verify SQLite works
sqlite3 target/data.db ".schema events"
```

### Phase 5: Real Counter tracer

```bash
# After all foundational layers exist
bd update ironstar-v72 --status in_progress

# Implement Counter tracer WITH REAL ARCHITECTURE
# - No Arc<AtomicI64>
# - Use EventStore.append_events()
# - Use ProjectionManager.get_state()
# - Stream SSE from projection

cargo test --test '*counter_e2e'

# Manual browser test
# Server starts, browser loads, clicks work, persists
```

## Summary

The proper solution to the blocking problem:

1. **Identifies real architectural constraints** and preserves them
2. **Removes false sequential ordering** to enable parallel work
3. **Keeps the real architecture** (SQLite, CQRS, Datastar SSE) instead of faking it
4. **Reduces scope without shortcuts** (fewer features, but all real)
5. **Validates at each phase** with integration tests
6. **Proves patterns with Counter**, replicates with Todo

This approach solves the blocking problem while building a genuine CQRS/Event Sourcing architecture that scales to the full system.

The tracer bullet becomes what it should be: a validator that the whole stack works, not a gate that blocks everything else.
