---
title: Issue Graph Reconciliation for fmodel-rust Adoption
---

# Issue graph reconciliation for fmodel-rust adoption

This document explains the issue graph changes required after the decision to adopt fmodel-rust as ironstar's event sourcing foundation.

## Context

The fmodel-rust adoption evaluation (docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md) concluded that fmodel-rust provides superior algebraic foundations compared to the previously planned custom Aggregate trait pattern.
This requires reconciling the existing issue graph to reflect the new architecture.

## Superseded issues

Four issues from the nyp epic are superseded by the new a9b epic tasks.

### ironstar-nyp.1: Create database migrations/ directory with schema.sql

**Status:** CLOSE

**Superseded by:** ironstar-a9b.3 (Create event store SQLite schema)

**Reason:**
The original nyp.1 planned a schema with `aggregate_sequence` for optimistic locking.
The fmodel-rust pattern uses `previous_id` (UUID chain) instead, requiring a different schema design.
a9b.3 implements the fmodel-rust-postgres adapted schema with:
- `previous_id UNIQUE REFERENCES events(event_id)` for optimistic locking
- `offset INTEGER PRIMARY KEY AUTOINCREMENT` for global SSE ordering
- `deciders` registry table for event type validation

### ironstar-nyp.3: Implement SQLite EventRepository with sqlx

**Status:** CLOSE

**Superseded by:** ironstar-a9b.1 (Implement SQLite EventRepository)

**Reason:**
The original nyp.3 planned a custom EventRepository trait without SSE support.
a9b.1 implements fmodel-rust's `EventRepository<C, E, Version, Error>` trait plus SSE-specific extension methods:
- `query_since_sequence(offset: i64)` for Last-Event-ID reconnection
- `earliest_sequence()` / `latest_sequence()` for stream bounds
- `query_all()` for projection rebuild on startup

This provides a complete foundation for both command handling (via EventSourcedAggregate) and query handling (via SSE streams).

### ironstar-nyp.6: Create Projection trait for read models

**Status:** CLOSE

**Superseded by:** fmodel-rust's `View` struct (implemented in a9b.5)

**Reason:**
The original nyp.6 planned a custom Projection trait with mutable `apply(&mut self, event: &E)`.
fmodel-rust's View uses a pure function signature: `evolve(&S, &E) -> S`.
This provides:
- Referential transparency (easier testing)
- Composability via `merge()` combinator
- Mathematical grounding in Galois connections (fold :: [Event] -> State)

The theoretical foundations documented in nyp.6 (Galois connections, adjoint functors) remain valid and transfer directly to fmodel's View abstraction.

### ironstar-nyp.7: Implement ProjectionManager with in-memory state

**Status:** CLOSE

**Superseded by:** ironstar-a9b.8 (Wire Todo MaterializedView)

**Reason:**
The original nyp.7 planned a centralized ProjectionManager holding all read model state.
fmodel-rust uses per-aggregate MaterializedView instances composed at the application layer.
This provides:
- Better separation of concerns (one view per aggregate type)
- Explicit dependency injection (ViewStateRepository)
- Type-safe wiring via EventSourcedAggregate

The application layer (ironstar-app) composes MaterializedView instances in AppState rather than using a global manager.

## Dependency rewiring

### Issues depending on nyp.3 → wire to a9b.1

These issues require EventRepository but do not need the old custom trait:

| Issue ID | Title | New Dependency |
|----------|-------|----------------|
| ironstar-amw | Command/Query separation | a9b.1 |
| ironstar-nyp.4 | Implement command handler workflow | a9b.1 |
| ironstar-apx.4 | Document event sourcing example | a9b.1 |
| ironstar-zuv.1 | Create EventRepository integration tests | a9b.1 |

**Rationale:**
All command handling flows through EventSourcedAggregate, which requires EventRepository.
a9b.1 provides both the fmodel-rust trait implementation and SSE extensions needed for downstream consumers.

### Issues depending on nyp.7 → wire to a9b.8

These issues require projection infrastructure but do not need the old centralized manager:

| Issue ID | Title | New Dependency |
|----------|-------|----------------|
| ironstar-r62.7 | Implement SSE projection updates | a9b.8 |
| ironstar-e6k.2 | Wire Todo projection | a9b.8 |
| ironstar-zuv.2 | Test projection updates | a9b.8 |

**Rationale:**
SSE streams serve MaterializedView state.
a9b.8 wires Todo's MaterializedView with ViewStateRepository, providing the foundation for SSE handlers and integration tests.

### Issues depending on nyp.6 → wire to a9b.5

These issues require the View abstraction but do not need the old mutable Projection trait:

| Issue ID | Title | New Dependency |
|----------|-------|----------------|
| ironstar-nyp.37 | Document Projection mathematical foundations | a9b.5 |
| ironstar-nyp.38 | Implement View composition patterns | a9b.5 |

**Rationale:**
The mathematical foundations (Galois connections) transfer directly to fmodel's View.
a9b.5 implements Todo's View as a pure `evolve(&S, &E) -> S` function, providing the concrete example for documentation and composition patterns.

## Reconciliation script

The reconciliation is automated via `reconcile-issue-graph.sh` with four phases:

1. **Remove stale dependencies** (11 removals)
   - Prevents "dependency on closed issue" errors
2. **Add new dependencies** (11 additions)
   - Wires to a9b.1, a9b.5, a9b.8
3. **Close superseded issues** (4 closes with comments)
   - nyp.1, nyp.3, nyp.6, nyp.7
4. **Verify reconciliation** (`bd status`)
   - Shows final state

## Impact on other epics

The reconciliation affects tasks in multiple epics:

- **ironstar-nyp (Event sourcing infrastructure)**: 4 tasks closed, remaining tasks updated
- **ironstar-amw (Command/Query separation)**: Rewired to a9b.1
- **ironstar-r62 (Presentation layer)**: Rewired to a9b.8
- **ironstar-e6k (Example application)**: Rewired to a9b.8
- **ironstar-apx (Documentation and template)**: Rewired to a9b.1
- **ironstar-zuv (Testing and integration)**: Rewired to a9b.1, a9b.8

## Post-reconciliation workflow

After running the script:

1. Review `bd status` output for any remaining blockers
2. Commit beads changes:
   ```bash
   bd hooks run pre-commit
   git add .beads/
   git commit -m "chore(issues): reconcile for fmodel-rust adoption"
   ```
3. Begin implementation with a9b.* tasks in dependency order:
   - a9b.2 (identifier traits) → a9b.3 (schema) → a9b.1 (EventRepository)
   - a9b.4 (Todo Decider) → a9b.5 (Todo View)
   - a9b.1 + a9b.5 → a9b.8 (MaterializedView wiring)

## References

- fmodel-rust adoption evaluation: `docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md`
- fmodel-rust repository: `~/projects/rust-workspace/fmodel-rust/`
- fmodel-rust-demo (reference implementation): `~/projects/rust-workspace/fmodel-rust-demo/`
- fstore-sql pattern (PostgreSQL): `~/projects/rust-workspace/fstore-sql/`
