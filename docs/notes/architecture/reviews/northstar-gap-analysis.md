---
title: Gap analysis: ironstar vs northstar tracer bullet
---

# Gap analysis: ironstar vs northstar tracer bullet

This document identifies architectural gaps between ironstar's documentation and the northstar Rust CQRS/ES tracer bullet proposal.

## High-priority gaps

### 1. UUID-tracked error context for distributed tracing

**Gap**: Ironstar error types lack UUID field for log correlation.

**Pattern from northstar**: Each error carries `id: Uuid` + `backtrace: Backtrace` for tracing failed operations across async boundaries.

**Why it matters**: In event-driven systems, errors often occur in background tasks (projections, DuckDB query execution). UUID linking is essential for distributed debugging.

**Address in**: `docs/notes/architecture/decisions/error-types.md` — add mandatory UUID + backtrace patterns with correlation examples.

### 2. Aggregate version() method for optimistic locking verification

**Gap**: Ironstar's Aggregate trait doesn't expose version tracking at trait level.

**Pattern from northstar**: `fn version(&self) -> u64` enables CommandHandler to verify sequence before appending events.

**Why it matters**: Without explicit version(), CommandHandler must infer version from event count—fragile and error-prone.

**Address in**: `docs/notes/architecture/core/crate-architecture.md` — update Aggregate trait definition.

### 3. Hybrid event store schema with global AND per-aggregate sequence

**Gap**: CQRS bootstrapping guide only shows per-aggregate sequence for SSE Last-Event-ID.

**Pattern from northstar**: Uses per-aggregate sequence in UNIQUE constraint for DB-level optimistic locking.

**Critical schema detail**: The UNIQUE constraint pattern `UNIQUE(aggregate_type, aggregate_id, sequence)` prevents concurrent modifications at the database level.

**Address in**: `docs/notes/architecture/cqrs/event-sourcing-core.md` — add hybrid schema diagram with both columns.

### 4. Background task spawning for long-running operations

**Gap**: No guidance on spawning async work after command persistence.

**Pattern from northstar**: Commands that trigger DuckDB queries spawn background tasks that emit follow-up events (QueryCompleted/QueryFailed).

**Why it matters**: Without this pattern, long-running queries block SSE streams.

**Address in**: `docs/notes/architecture/cqrs/background-task-patterns.md` — add "spawn-after-persist" section.

## Medium-priority gaps

### 5. QuerySessionAggregate reference pattern

**Gap**: No reference for how to model analytics-specific state.

**Pattern from northstar**: QuerySessionAggregate encodes dataset URL, active query, query history, execution state.

**Address in**: `docs/notes/architecture/cqrs/event-sourcing-core.md` — add analytics domain model example section.

### 6. Session-scoped key expression patterns

**Gap**: Documentation shows generic `events/Counter/**`; northstar shows `viz/{session}/events` for per-session subscriptions.

**Why it matters**: Session-scoped subscriptions prevent cross-session contamination; essential for multi-user analytics.

**Address in**: `docs/notes/architecture/infrastructure/session-management.md` — add key expression pattern.

### 7. Optimistic locking check in CommandHandler

**Gap**: CQRS bootstrapping guide doesn't show version check before persisting.

**Pattern from northstar**: Explicit `current_version = aggregate.version()` before calling `append_events(..., current_version)`.

**Address in**: `docs/notes/architecture/implementation/cqrs-bootstrapping-guide.md` Step 4.2.

### 8. Subscribe-before-replay invariant prominence

**Gap**: Mentioned but not prominently in SSE lifecycle guide.

**Address in**: `docs/notes/architecture/cqrs/sse-connection-lifecycle.md` — elevate to Critical Invariant section.

## Summary

| Gap | Priority | Status |
|-----|----------|--------|
| UUID-tracked errors | HIGH | Document needed |
| Aggregate version() | HIGH | Trait update needed |
| Hybrid event store schema | HIGH | Schema update needed |
| Background task spawning | HIGH | Pattern doc needed |
| QuerySessionAggregate example | MEDIUM | Example needed |
| Session-scoped key expressions | MEDIUM | Doc update needed |
| Optimistic locking in handler | MEDIUM | Example update needed |
| Subscribe-before-replay | MEDIUM | Doc reorganization needed |

These gaps relate to production CQRS patterns that northstar's analytics focus naturally exposes (long-running queries, session routing, error correlation).
