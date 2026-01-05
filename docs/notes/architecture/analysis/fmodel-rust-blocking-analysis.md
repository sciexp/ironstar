# fmodel-rust blocking analysis

## Executive summary

The ironstar project has **critical architectural inconsistency** in its dependency graph.
Epic `ironstar-a9b` (Implement fmodel-rust event sourcing foundation) and epic `ironstar-nyp` (Event sourcing infrastructure) contain **overlapping responsibilities** with **conflicting implementations**, creating a hidden architectural debt that will manifest as merge conflicts and refactoring costs.

The current graph shows `nyp` blocking `a9b` dependencies (via the epic-level dependency), but **the opposite should be true**: fmodel-rust adoption should block and **replace** significant portions of the existing NYP event store tasks.

### Key findings

1. **Duplicate event store implementations**: NYP.2 (EventStore trait) and NYP.3 (SQLite event store) **duplicate** a9b.1 (SQLite EventRepository)
2. **Conflicting schemas**: NYP.35 (hybrid schema with dual sequences) and a9b.3 (fmodel-rust schema with previous_id chain) are **incompatible**
3. **Abandoned abstractions**: NYP.2's EventStore trait will be **unused** if fmodel-rust is adopted, replaced by fmodel-rust's EventRepository trait
4. **Missing blocking relationships**: 13 NYP tasks should depend on a9b completion but currently don't

## Dependency structure analysis

### Current epic-level blocking

```
ironstar-2nt (Domain layer) ──blocks──> ironstar-nyp (Event sourcing infrastructure)
ironstar-nyp ──blocks──> ironstar-3gd (Scientific Data Integration)
ironstar-nyp ──blocks──> ironstar-r62 (Presentation layer)
ironstar-nyp ──blocks──> ironstar-jqv (Authentication)
ironstar-a9b (fmodel-rust) has NO epic-level blockers or blocks
```

**Problem**: `ironstar-a9b` is a P1 epic with 13 children but no epic-level blocking relationships despite being foundational to event sourcing.

### Epics with event sourcing dependencies

| Epic | Children | Blocks/Blocked | Needs fmodel-rust? |
|------|----------|----------------|-------------------|
| **ironstar-nyp** (Event sourcing infrastructure) | 41 tasks | Blocks: 3gd, r62, jqv | **CONFLICTS** with a9b |
| **ironstar-a9b** (fmodel-rust foundation) | 13 tasks | No epic blocks | **FOUNDATIONAL** |
| **ironstar-2nt** (Domain layer) | 17 tasks | Blocks: nyp, 3gd | Partial (Decider types) |
| **ironstar-r62** (Presentation layer) | 18 tasks | Blocked by: nyp, ny3 | Yes (AppState, handlers) |
| **ironstar-e6k** (Todo example) | 8 tasks | Blocked by: r62 | Yes (aggregate wiring) |
| **ironstar-3gd** (Scientific data) | 10 tasks | Blocked by: 2nt, nyp | Maybe (analytics events) |

### Task-level conflicts

#### Direct duplicates

| NYP Task | a9b Task | Conflict Type |
|----------|----------|---------------|
| **nyp.2**: Create EventStore trait | **a9b.1**: Implement SQLite EventRepository | **Abandoned abstraction** - NYP.2's EventStore trait duplicates fmodel-rust's EventRepository |
| **nyp.3**: Implement SQLite event store with sqlx | **a9b.1**: Implement SQLite EventRepository | **Direct duplicate** - both implement SQLite event persistence |
| **nyp.35**: Hybrid schema (dual sequences) | **a9b.3**: fmodel-rust schema (previous_id chain) | **Schema conflict** - incompatible optimistic locking strategies |
| **nyp.1**: Create migrations/ directory | **a9b.3**: Create event store SQLite schema | **Overlapping scope** - both define event table schema |

#### Semantic conflicts

| NYP Task | Assumption | fmodel-rust Reality |
|----------|-----------|---------------------|
| **nyp.6**: Create Projection trait | Custom trait for read models | fmodel-rust provides `View` trait |
| **nyp.7**: Implement ProjectionManager | Custom event replay logic | fmodel-rust provides `MaterializedView` with built-in replay |
| **nyp.2**: EventStore::append(events: Vec\<Event>) | Batch append | fmodel-rust EventRepository appends per-command (single-event or multi-event from Decider) |

### Blocking relationships that should exist

**These NYP tasks should depend on a9b completion:**

| Task | Current Dependencies | Should Add | Reason |
|------|---------------------|------------|--------|
| nyp.2 | 2nt.2, 2nt.11 | **a9b.1** | EventStore trait conflicts with fmodel EventRepository |
| nyp.3 | nyp.1, nyp.2 | **a9b.1, a9b.3** | Direct duplicate of SQLite EventRepository |
| nyp.1 | nyp.35, 2nt.11 | **a9b.3** | Schema must match fmodel-rust's event table |
| nyp.6 | 2nt.2 | **a9b.5** | Projection trait should align with fmodel View |
| nyp.7 | nyp.6, nyp.3, nyp.5, nyp.27 | **a9b.8** | Should use MaterializedView, not custom manager |
| e6k.2 | nyp.7 | **a9b.8** | TodoListProjection should use MaterializedView |
| e6k.3 | e6k.2, r62.6 | **a9b.10** | add_todo handler needs Todo command handler |
| e6k.4 | e6k.3 | **a9b.10** | mark_todo handler needs command handler wiring |
| e6k.5 | e6k.3 | **a9b.10** | delete_todo handler needs command handler wiring |
| e6k.8 | e6k.3-7 | **a9b.7, a9b.9** | Route mounting needs EventSourcedAggregate + Zenoh |
| r62.4 | nyp.3, nyp.5, nyp.7, nyp.10, nyp.27 | **a9b.7** | AppState needs EventSourcedAggregate, not raw EventStore |
| r62.6 | r62.4 | **a9b.10** | Command handlers call aggregate.handle(), not EventStore directly |
| r62.7 | nyp.7, r62.4 | **a9b.11** | Query handlers should use MaterializedView queries |

## Architectural conflict resolution

### Strategy 1: Replace NYP event store with fmodel-rust (RECOMMENDED)

**Action**: Mark NYP.2, NYP.3, NYP.35 as superseded by a9b tasks.

**Rationale**:
- fmodel-rust adoption evaluation (docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md) already concluded fmodel-rust is superior
- The hybrid schema (NYP.35) and fmodel-rust schema (a9b.3) serve the same purpose with different mechanisms
- Custom EventStore trait (NYP.2) is unnecessary when using fmodel-rust's EventRepository

**Dependencies to add**:
```bash
# NYP tasks that should block on a9b
bd dep add ironstar-nyp.2 ironstar-a9b.1 --type blocks-on  # EventStore trait waits for EventRepository
bd dep add ironstar-nyp.3 ironstar-a9b.1 --type blocks-on  # SQLite impl waits for fmodel pattern
bd dep add ironstar-nyp.3 ironstar-a9b.3 --type blocks-on  # Needs schema from a9b.3
bd dep add ironstar-nyp.1 ironstar-a9b.3 --type blocks-on  # Migrations need fmodel schema
bd dep add ironstar-nyp.6 ironstar-a9b.5 --type blocks-on  # Projection trait aligns with View
bd dep add ironstar-nyp.7 ironstar-a9b.8 --type blocks-on  # ProjectionManager uses MaterializedView

# Todo example tasks
bd dep add ironstar-e6k.2 ironstar-a9b.8 --type blocks-on  # TodoListProjection needs MaterializedView
bd dep add ironstar-e6k.3 ironstar-a9b.10 --type blocks-on # Handlers need command wiring
bd dep add ironstar-e6k.4 ironstar-a9b.10 --type blocks-on
bd dep add ironstar-e6k.5 ironstar-a9b.10 --type blocks-on
bd dep add ironstar-e6k.8 ironstar-a9b.7 --type blocks-on  # Routing needs aggregate
bd dep add ironstar-e6k.8 ironstar-a9b.9 --type blocks-on  # Routing needs Zenoh

# Presentation layer tasks
bd dep add ironstar-r62.4 ironstar-a9b.7 --type blocks-on  # AppState needs EventSourcedAggregate
bd dep add ironstar-r62.6 ironstar-a9b.10 --type blocks-on # Command handlers
bd dep add ironstar-r62.7 ironstar-a9b.11 --type blocks-on # Query handlers
```

**Deprecations** (mark as superseded, close without implementation):
- NYP.2 → Replaced by fmodel-rust's EventRepository trait
- NYP.35 → Replaced by a9b.3's fmodel schema (previous_id chain covers optimistic locking)

**Schema reconciliation**:
- Keep NYP.35's `global_sequence` for SSE Last-Event-ID semantics
- Adopt a9b.3's `previous_id` chain for optimistic locking
- Merge into single schema: `events` table with both columns

### Strategy 2: Parallel implementation with later convergence (NOT RECOMMENDED)

**Action**: Keep both NYP and a9b event stores, use feature flags to switch.

**Problems**:
- Doubles implementation cost
- Creates testing surface explosion (NYP tests × a9b tests)
- Violates "one way to do things" principle
- Delays architectural decision that evaluation already made

### Strategy 3: Hybrid approach with abstraction layer (NOT RECOMMENDED)

**Action**: Create EventStore trait that both NYP.3 and a9b.1 implement.

**Problems**:
- fmodel-rust already provides the abstraction (EventRepository)
- Creates unnecessary indirection
- Loses fmodel-rust's type safety benefits (Identifier, EventType, DeciderType traits)

## Recommended action plan

### Phase 1: Establish fmodel-rust foundation (unblock a9b.2, a9b.3)

These tasks are **currently unblocked** and should be started immediately:

1. **a9b.2**: Implement fmodel-rust identifier traits (Identifier, EventType, DeciderType, IsFinal)
2. **a9b.3**: Create event store SQLite schema (adapted from fmodel-rust-postgres)

**Start these first** to unblock downstream a9b tasks.

### Phase 2: Add blocking dependencies (wire graph)

```bash
# Block NYP event store on fmodel-rust
bd dep add ironstar-nyp.1 ironstar-a9b.3 --type blocks-on
bd dep add ironstar-nyp.2 ironstar-a9b.1 --type blocks-on
bd dep add ironstar-nyp.3 ironstar-a9b.1 --type blocks-on
bd dep add ironstar-nyp.3 ironstar-a9b.3 --type blocks-on

# Block NYP projections on fmodel-rust
bd dep add ironstar-nyp.6 ironstar-a9b.5 --type blocks-on
bd dep add ironstar-nyp.7 ironstar-a9b.8 --type blocks-on

# Block Todo example on fmodel-rust
bd dep add ironstar-e6k.2 ironstar-a9b.8 --type blocks-on
bd dep add ironstar-e6k.3 ironstar-a9b.10 --type blocks-on
bd dep add ironstar-e6k.4 ironstar-a9b.10 --type blocks-on
bd dep add ironstar-e6k.5 ironstar-a9b.10 --type blocks-on
bd dep add ironstar-e6k.8 ironstar-a9b.7 --type blocks-on
bd dep add ironstar-e6k.8 ironstar-a9b.9 --type blocks-on

# Block presentation layer on fmodel-rust
bd dep add ironstar-r62.4 ironstar-a9b.7 --type blocks-on
bd dep add ironstar-r62.6 ironstar-a9b.10 --type blocks-on
bd dep add ironstar-r62.7 ironstar-a9b.11 --type blocks-on
```

### Phase 3: Deprecate conflicting NYP tasks

Close these issues as superseded:

```bash
bd close ironstar-nyp.2 --comment "Superseded by ironstar-a9b.1 (fmodel-rust EventRepository)"
bd close ironstar-nyp.35 --comment "Schema reconciled into ironstar-a9b.3 (fmodel schema + global_sequence)"
```

Reclassify NYP.3 as **implementation task for a9b.1** (rename/retarget):
- Change title: "Implement SQLite event store with sqlx" → "Implement SQLite EventRepository adapter for fmodel-rust"
- Update description: Reference fmodel-rust-demo's event_store.rs pattern
- Keep as child of NYP epic (infrastructure), but depend on a9b.1

### Phase 4: Reconcile schemas

Create **merged schema** in a9b.3:

```sql
CREATE TABLE events (
    -- fmodel-rust columns
    event TEXT NOT NULL,
    event_id TEXT NOT NULL UNIQUE CHECK(length(event_id) = 36),
    decider TEXT NOT NULL,
    decider_id TEXT NOT NULL,
    data TEXT NOT NULL CHECK(json_valid(data)),
    command_id TEXT,
    previous_id TEXT UNIQUE REFERENCES events(event_id),
    final INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT(datetime('now', 'utc')),

    -- NYP.35 addition: global sequence for SSE Last-Event-ID
    offset INTEGER PRIMARY KEY AUTOINCREMENT,

    FOREIGN KEY (decider, event) REFERENCES deciders (decider, event)
) STRICT;
```

**Optimistic locking**: Use `previous_id` chain (fmodel-rust pattern), not separate aggregate_sequence column.

**SSE semantics**: Use `offset` (global_sequence) for Last-Event-ID.

### Phase 5: Update documentation

Files to update:

1. **docs/notes/architecture/cqrs/event-sourcing-core.md**
   - Remove references to custom EventStore trait
   - Add fmodel-rust EventRepository, Decider, View, EventSourcedAggregate
   - Document identifier traits (Identifier, EventType, DeciderType, IsFinal)

2. **docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md**
   - Add "Migration executed" status
   - Document schema reconciliation decision

3. **CLAUDE.md**
   - Update "Event sourcing model" section to reference fmodel-rust abstractions
   - Update "Event store schema" to show merged schema

## Impact on unblocked work

### Currently ready (80 issues)

After adding blocking relationships, **these will become blocked**:

| Issue | Current Status | Will Become |
|-------|----------------|-------------|
| e6k.3, e6k.4, e6k.5 | Ready (only depends on e6k.2) | Blocked by a9b.10 |
| e6k.8 | Ready (depends on e6k.3-7) | Blocked by a9b.7, a9b.9 |
| r62.4 | Blocked by nyp.3, nyp.5, nyp.7, nyp.10, nyp.27 | Also blocked by a9b.7 |
| r62.6 | Blocked by r62.4 | Also blocked by a9b.10 |
| r62.7 | Blocked by nyp.7, r62.4 | Also blocked by a9b.11 |

### New unblocked work (after Phase 1)

Completing a9b.2 and a9b.3 will **unblock**:

| Issue | Unblocked By |
|-------|-------------|
| a9b.1 | a9b.2 + a9b.3 |
| a9b.4 | a9b.2 |
| a9b.6 | a9b.2 |

Completing a9b.4 (Todo Decider) will unblock:

| Issue | Unblocked By |
|-------|-------------|
| a9b.5 | a9b.4 |
| a9b.7 | a9b.1 + a9b.4 |
| a9b.12 | a9b.4 |

## Tier structure after rewiring

### Tier 0 (foundation, ready now)

- **a9b.2**: Implement identifier traits (NO dependencies except epic)
- **a9b.3**: Create event store schema (NO dependencies except epic)

### Tier 1 (depends on Tier 0)

- **a9b.1**: Implement SQLite EventRepository (depends on a9b.2, a9b.3)
- **a9b.4**: Implement Todo Decider (depends on a9b.2)
- **a9b.6**: Implement QuerySession Decider (depends on a9b.2)

### Tier 2 (depends on Tier 1)

- **a9b.5**: Implement Todo View (depends on a9b.4)
- **a9b.7**: Wire Todo EventSourcedAggregate (depends on a9b.1, a9b.4)
- **a9b.12**: Implement Decider specification tests (depends on a9b.4)

### Tier 3 (depends on Tier 2)

- **a9b.8**: Wire Todo MaterializedView (depends on a9b.5)
- **a9b.9**: Integrate Zenoh event publishing (depends on a9b.7)
- **a9b.13**: Implement View specification tests (depends on a9b.5)

### Tier 4 (application layer, depends on Tier 3)

- **a9b.10**: Implement Todo command handler (depends on a9b.7, a9b.9)
- **a9b.11**: Implement Todo query handler (depends on a9b.8)

### Tier 5 (presentation integration, depends on Tier 4)

- **e6k.2**: TodoListProjection (depends on a9b.8)
- **e6k.3-5**: Todo handlers (depend on a9b.10)
- **e6k.8**: Route mounting (depends on a9b.7, a9b.9)
- **r62.4**: AppState (depends on a9b.7)
- **r62.6**: Command handlers (depend on a9b.10)
- **r62.7**: Query handlers (depend on a9b.11)

## Cross-epic dependency visualization

```
┌─────────────────────────────────────────────────────────────────┐
│                     Foundation Layer (Tier 0)                    │
│  a9b.2 (identifier traits)    a9b.3 (event store schema)        │
└────────────────────────┬──────────────────┬─────────────────────┘
                         │                  │
         ┌───────────────┴─────┬────────────┴───────┬─────────────┐
         │                     │                    │             │
    ┌────▼─────┐         ┌────▼─────┐        ┌─────▼────┐  ┌────▼─────┐
    │  a9b.1   │         │  a9b.4   │        │  a9b.6   │  │  2nt.5   │
    │EventRepo │         │Todo      │        │QuerySess │  │Datastar  │
    │          │         │Decider   │        │Decider   │  │signals   │
    └────┬─────┘         └────┬─────┘        └──────────┘  └──────────┘
         │                    │
         │              ┌─────┴──────┬────────────┐
         │              │            │            │
    ┌────▼──────┐  ┌───▼────┐  ┌───▼────┐  ┌───▼─────┐
    │  a9b.7    │  │ a9b.5  │  │a9b.12  │  │ 2nt.6   │
    │EventSrcd  │  │Todo    │  │Decider │  │camelCase│
    │Aggregate  │  │View    │  │tests   │  │enforce  │
    └────┬──────┘  └───┬────┘  └────────┘  └─────────┘
         │             │
    ┌────┴──────┬──────┴─────┐
    │           │            │
┌───▼────┐ ┌───▼────┐  ┌───▼─────┐
│ a9b.9  │ │ a9b.8  │  │ a9b.13  │
│Zenoh   │ │Material│  │View     │
│publish │ │izedView│  │tests    │
└───┬────┘ └───┬────┘  └─────────┘
    │          │
    │     ┌────┴─────┬──────────┬──────────┐
    │     │          │          │          │
┌───▼─────▼──┐  ┌───▼────┐ ┌──▼─────┐ ┌──▼─────┐
│  a9b.10    │  │a9b.11  │ │ e6k.2  │ │ r62.4  │
│Todo cmd    │  │Todo qry│ │TodoList│ │AppState│
│handler     │  │handler │ │Proj    │ │        │
└────┬───────┘  └───┬────┘ └───┬────┘ └───┬────┘
     │              │          │          │
     │         ┌────┴──────────┴──┬───────┴─────┬──────┐
     │         │                  │             │      │
┌────▼─────────▼───┐         ┌───▼────┐   ┌───▼────┐ │
│  e6k.3,4,5       │         │ r62.6  │   │ r62.7  │ │
│  Todo handlers   │         │Command │   │Query   │ │
│                  │         │handlers│   │handlers│ │
└────┬─────────────┘         └───┬────┘   └───┬────┘ │
     │                           │            │      │
     └───────────┬───────────────┴────────────┴──────┘
                 │
            ┌────▼────┐
            │ e6k.8   │
            │Route    │
            │mounting │
            └─────────┘
```

## Recommendations

1. **Immediate**: Start a9b.2 and a9b.3 (unblocked, foundational)
2. **Before next session**: Add all blocking dependencies listed in Phase 2
3. **Document**: Update CLAUDE.md and event-sourcing-core.md to reflect fmodel-rust adoption
4. **Deprecate**: Close NYP.2 and NYP.35 as superseded
5. **Retarget**: Rename NYP.3 to "Implement SQLite EventRepository adapter" and make it depend on a9b.1

## Risk assessment

**If fmodel-rust blocking is NOT established:**

- Medium risk: Developer starts NYP.3, implements custom EventStore, discovers incompatibility with fmodel-rust later
- High cost: Rework NYP.3 implementation after a9b.1 completion
- Schedule impact: 2-4 days rework + merge conflict resolution
- Cognitive load: Maintaining two event store abstractions in parallel

**If fmodel-rust blocking IS established:**

- Low risk: Clear implementation path, no conflicting abstractions
- Low cost: Linear dependency chain, no rework
- Schedule impact: None (correct dependencies prevent wasted work)
- Cognitive load: Single abstraction (fmodel-rust), easier to reason about

## References

- fmodel-rust adoption evaluation: `/Users/crs58/projects/rust-workspace/ironstar/docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md`
- fmodel-rust: `~/projects/rust-workspace/fmodel-rust/`
- fmodel-rust-demo: `~/projects/rust-workspace/fmodel-rust-demo/`
- fmodel-rust-postgres: `~/projects/rust-workspace/fmodel-rust-postgres/`
- fstore-sql: `~/projects/rust-workspace/fstore-sql/`
