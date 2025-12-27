# Comprehensive Architecture Review: Ironstar vs Northstar Tracer Bullet

## Executive Summary

This review compares ironstar's architecture documentation (38 files across 5 categories) with the northstar Rust CQRS/ES Datastar tracer bullet architecture (9 files).
The analysis reveals that ironstar and northstar are complementary halves of a unified vision: northstar provides concrete implementation patterns while ironstar delivers comprehensive design rationale.

**Key findings:**

1. **Internal contradiction in ironstar** requires immediate resolution (event bus architecture)
2. **7 critical gaps** from northstar's tracer bullet are missing in ironstar
3. **4 missing documentation files** referenced but not created in ironstar
4. **13 aligned pattern areas** demonstrate strong architectural compatibility
5. **8 ironstar innovations** are valuable for Phase 1 implementation

**Recommendation**: Resolve contradictions, fill critical gaps, then proceed with implementation using northstar's tracer bullet as the reference implementation.

---

## Critical Issues Requiring Immediate Resolution

### Issue 1: Event Bus Architecture Contradiction

**Severity**: Blocks implementation

**Location**: Internal ironstar documentation contradiction

**Problem**:
- `architecture-decisions.md` states: "Zenoh (embedded mode) as primary event bus from day one"
- `distributed-event-bus-migration.md` described: "migration path from tokio::broadcast to Zenoh"

These appeared mutually exclusive, but the document's scope has been clarified.

**Resolution**: Rename `distributed-event-bus-migration.md` to `event-bus-compatibility-patterns.md` to reflect its actual scope: compatibility patterns for optional tokio::broadcast fallback in resource-constrained environments, not migration guidance for ironstar itself.

**Rationale**:
- Zenoh embedded mode requires only 4 lines of configuration
- Scales to ~10K concurrent SSE subscribers
- Distribution-ready via peer mode configuration change (no code changes)
- Eliminates entire category of dual-bus coordination bugs
- Template should teach best practices, not migration paths

---

### Issue 2: Missing Error Types Documentation

**Severity**: High - Referenced but not created

**Location**: `docs/notes/architecture/decisions/error-types.md`

**Problem**: Multiple documents reference this file but it doesn't exist.

**Required content**:
- Error type hierarchy (domain vs infrastructure)
- Concrete type definitions (`CommandError`, `QueryError`, `InfraError`)
- `From` implementations for error conversion
- HTTP status code mapping
- User-facing message extraction patterns

---

### Issue 3: Missing Metrics Reference Documentation

**Severity**: High - Referenced but not created

**Location**: `docs/notes/architecture/decisions/metrics-reference.md`

**Problem**: Multiple documents reference this file but it doesn't exist.

**Required content**:
- Prometheus metrics catalog (counters, histograms, gauges)
- Naming conventions (`ironstar_*` prefix)
- Cardinality guidelines
- Alert threshold recommendations

---

### Issue 4: Projection Consistency Strategy Unclear

**Severity**: Medium - Conflicting patterns

**Problem**: Ironstar documents eventual consistency (cache-aside) but northstar implements strong consistency (sequence tracking for read-your-writes).

**Resolution**: Adopt dual strategy:
- Domain projections (TodoList): Strong consistency with sequence tracking
- Analytics projections (Dashboard): Eventual consistency with TTL cache

---

## Gap Analysis Summary

### Critical Gaps: Northstar Patterns Missing from Ironstar

| # | Pattern | Northstar Location | Impact |
|---|---------|-------------------|--------|
| 1 | DatastarRequest Extractor | web-layer.md | Blocks SSE implementation |
| 2 | Concrete Aggregate Example | domain-layer.md | No reference implementation |
| 3 | Event Store Transactions | infrastructure-layer.md | Risk of data corruption |
| 4 | Query Service + ChartTransformer | application-layer.md | Blocks analytics |
| 5 | Progressive Enhancement SSE | web-layer.md | Core Datastar pattern |
| 6 | Zenoh Key Expression Strategy | infrastructure-layer.md | Ad-hoc routing likely |
| 7 | CQRS Bootstrapping Guide | implementation-roadmap.md | No clear starting point |

### High-Priority Gaps (12 items)

- TypeScript signal contracts with ts-rs (concrete workflow)
- Lazy HTML rendering with hypertext (caching integration)
- Event schema evolution (Upcaster pattern)
- Projection rebuilding (CLI commands, atomic swap)
- DuckDB query parameterization (security)
- SSE error recovery (circuit breaker patterns)
- Command validation boundary (syntactic vs semantic)
- DuckDB projection schema (indexing recommendations)
- Zenoh embedded configuration (complete example)
- Todo complete flow (end-to-end example)
- Testing strategy per layer (aggregate, integration, e2e)
- Session-scoped event routing (Zenoh key patterns)

### Ironstar Innovations Beyond Tracer Bullet

| Pattern | Assessment | Phase |
|---------|------------|-------|
| Multi-crate workspace (HasXxx traits) | Valuable | Phase 2 |
| CQRS event sourcing (pure sync aggregates) | Core | Phase 1 |
| Zenoh-first event bus | Core | Phase 1 |
| Analytics cache (moka + Zenoh invalidation) | Defer | Phase 2 |
| Session management + security | Core | Phase 1 |
| OAuth-only authentication | Core | Phase 1 |
| TypeScript type generation (ts-rs) | Valuable | Phase 1 |
| Lit component integration | Valuable | Phase 1 |
| Observability (metrics, logging) | Defer | Phase 2 |
| Remote data via httpfs | Optional | Phase 3 |
| Nix template machinery | Core | Phase 1 |
| Category-based CI | Defer | Phase 2 |
| Error handling patterns | Core | Phase 1 |

---

## Architectural Alignment Analysis

### Strongly Aligned Patterns (13 areas)

The following patterns are compatible between ironstar and northstar, enabling cross-pollination:

1. **Pure Synchronous Aggregates**: `handle_command(state, cmd) -> Result<Vec<Event>, Error>`
2. **Zenoh-First Event Bus**: Embedded mode with key expression filtering
3. **SQLite Event Store**: Global sequence for SSE Last-Event-ID
4. **SSE Connection Lifecycle**: Reconnection with event replay
5. **Session Management**: SQLite storage + per-session Zenoh keys
6. **Analytics Cache**: Moka + Zenoh invalidation
7. **Projection Strategies**: SQLite (operational) + DuckDB (analytics)
8. **Frontend Integration**: Datastar SSE + Lit web components
9. **Testing Strategies**: TestFramework DSL + property testing
10. **Performance Tuning**: Channel sizing, backpressure handling
11. **Error Handling**: Railway-oriented programming with Result types
12. **Observability**: tracing + Prometheus metrics
13. **Migration Path**: DualEventBus abstraction (future scaling)

### Synergy Opportunities

| Opportunity | Description |
|-------------|-------------|
| Shared pattern catalog | Markdown table linking ironstar docs ↔ northstar code |
| Cross-referenced documentation | "See also" sections connecting theory to implementation |
| Reusable components | Package DualEventBus, TestFramework extensions as shared crate |
| Joint testing playbook | Combine northstar tests with ironstar strategy docs |
| Performance benchmark suite | k6 scripts testing both implementations |

---

## Recommended Action Plan

### Phase 0: Documentation Cleanup (1-2 days)

**Priority 1 - Resolve contradictions:**
1. Rename `distributed-event-bus-migration.md` to `event-bus-compatibility-patterns.md`
2. Update all documentation references to use the new filename
3. Update `event-bus-compatibility-patterns.md` header to clarify scope: compatibility patterns, not migration guidance

**Priority 2 - Create missing documents:**
4. Create `error-types.md` with error hierarchy
5. Create `metrics-reference.md` with Prometheus catalog

**Priority 3 - Update existing docs:**
6. Add projection consistency strategy (domain: strong, analytics: eventual)
7. Enhance Zenoh key expression documentation with complete taxonomy

### Phase 1: Fill Critical Gaps (3-5 days)

Create 7 new documents addressing critical gaps:

1. `docs/notes/architecture/web/datastar-request-extractor.md`
2. `docs/notes/architecture/examples/todo-aggregate-complete.md`
3. `docs/notes/architecture/cqrs/event-store-transactions.md`
4. `docs/notes/architecture/application/query-service-pattern.md`
5. `docs/notes/architecture/web/progressive-enhancement-sse.md`
6. `docs/notes/architecture/infrastructure/zenoh-key-taxonomy.md`
7. `docs/notes/architecture/implementation/cqrs-bootstrapping-guide.md`

### Phase 2: High-Priority Enhancements (2-3 days)

Enhance existing documents with 12 high-priority patterns from northstar.

### Phase 3: Implementation

With documentation complete, implement ironstar following:
- Northstar tracer bullet as reference implementation
- Ironstar architecture decisions as design guidance
- TodoMVC as first aggregate (matching northstar's example)

---

## Document Relationship Diagram

```
NORTHSTAR (Tracer Bullet)          IRONSTAR (Architecture Design)
========================          ============================

index.md ──────────────────────── docs/notes/architecture/core/architecture-decisions.md
architecture-overview.md ───────── docs/notes/architecture/core/design-principles.md
domain-layer.md ────────────────── docs/notes/architecture/cqrs/event-sourcing-core.md
application-layer.md ───────────── docs/notes/architecture/cqrs/command-write-patterns.md
infrastructure-layer.md ────────── docs/notes/architecture/infrastructure/zenoh-event-bus.md
web-layer.md ───────────────────── docs/notes/architecture/cqrs/sse-connection-lifecycle.md
essential-patterns.md ──────────── docs/notes/architecture/cqrs/projection-patterns.md
implementation-roadmap.md ──────── [MISSING: cqrs-bootstrapping-guide.md]
reference.md ───────────────────── [MISSING: todo-aggregate-complete.md]

```

---

## Conclusion

Ironstar and northstar are not competing architectures—they are complementary.
Northstar proves ironstar's ideas work through concrete implementation.
Ironstar explains why northstar's code is designed the way it is.

The primary work required is:
1. Resolving the event bus documentation contradiction
2. Creating missing referenced documents
3. Filling 7 critical gaps from northstar's tracer bullet
4. Enhancing existing docs with 12 high-priority patterns

With these gaps filled, ironstar will provide a complete resource: theory (comprehensive documentation) + practice (northstar reference implementation) = production-ready template.

---

## Appendix: Full Gap Inventory

### A. Gaps from Northstar Missing in Ironstar (32 items)

**Critical (7)**: DatastarRequest extractor, concrete aggregate, event store transactions, query service pattern, progressive enhancement SSE, Zenoh key strategy, bootstrapping guide

**High (12)**: Signal contracts workflow, lazy HTML caching, event upcasting, projection rebuilding, DuckDB parameterization, SSE error recovery, command validation boundary, DuckDB projection schema, Zenoh embedded config, todo complete flow, testing strategy, session-scoped routing

**Medium (8)**: Module organization, feature checklists, error message UX, performance benchmarks, observability integration, rollback procedures, asset verification, local database setup

**Low (5)**: CLI reference, deployment checklist, multi-node scaling, i18n patterns, API documentation

### B. Gaps from Ironstar Missing in Northstar (16 critical items)

**Production patterns not in tracer bullet scope**: Performance tuning, advanced patterns (debouncing/batching), session security, error handling hierarchy, observability stack, OAuth authentication, analytics cache, event replay consistency (snapshot+delta), signal contracts (ts-rs), Zenoh deep dive, CSS architecture, integration patterns documentation, frontend build pipeline, crate architecture, session implementation, command write safety

---

*Review generated: 2025-12-25*
*Analyzed: 38 ironstar docs + 9 northstar docs*
*Methodology: Parallel subagent analysis with workflow DAG*
