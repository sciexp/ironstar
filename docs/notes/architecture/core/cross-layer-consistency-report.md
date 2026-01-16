# Ironstar Cross-Layer Consistency Report

Generated: 2026-01-16

## Executive Summary

This audit reviewed five interconnected documentation/specification layers for the ironstar project: Idris2 specifications (formal types), Qlerify Event Model (visual modeling), EventCatalog (generated documentation), Architecture Docs (design decisions), and Beads Issues (implementation roadmap).
The analysis identified 18 discrepancies across layers, with 3 blocking issues requiring immediate attention before implementation can proceed.
The project demonstrates strong architectural alignment overall, with primary gaps in type representation consistency and value object documentation completeness.

## Discrepancy Matrix

| ID | Discrepancy | Layers | Severity | Resolution |
|----|-------------|--------|----------|------------|
| D1 | Timestamp type mismatch: Idris Integer ms vs EventCatalog ISO 8601 string | Idris, EventCatalog | **Blocking** | Align on ISO 8601 at JSON boundary; document internal conversion |
| D2 | Todo aggregate missing from EventCatalog and Qlerify | Idris, EventCatalog, Qlerify | **Blocking** | Add Todo domain to EventCatalog or remove from Idris if out of scope |
| D3 | SessionState enum incomplete: missing NoSession initial state | Idris, EventCatalog | **Blocking** | Document NoSession as pre-creation state in Session entity |
| D4 | Nat constraints not enforced in JSON Schema (missing minimum: 0) | Idris, EventCatalog | Warning | Add minimum: 0 to 12+ integer fields |
| D5 | Naming convention mismatch: Qlerify space-separated vs EventCatalog PascalCase | Qlerify, EventCatalog | Warning | Document as serialization boundary convention |
| D6 | 19 value objects missing EventCatalog entity documentation | Qlerify, EventCatalog | Warning | Generate entity MDX files for Phase 3 |
| D7 | ChartType, CatalogState, ChartOptions lack schema.json files | Idris, EventCatalog | Warning | Create JSON Schema definitions or document as internal-only |
| D8 | GridSize allows 0 in Idris but EventCatalog implies minimum 1 | Idris, EventCatalog | Warning | Add smart constructor in Idris; update schema constraints |
| D9 | OAuthProvider not standalone entity in EventCatalog | Idris, EventCatalog | Info | Optional: create dedicated entity for clarity |
| D10 | ChartPlacement decomposition undocumented in event schema | Idris, EventCatalog | Info | Add field mapping documentation to DashboardChartAdded |
| D11 | EventId/Version envelope types not in JSON representation | Idris, EventCatalog | Info | Document as metadata fields outside payload |
| D12 | Qlerify schema type coverage: 84% fields untyped | Qlerify | Warning | Enrich Qlerify export with dataType annotations |
| D13 | Missing enums: RevocationReason, QueryErrorCode, SessionStatus | Idris | Warning | Add to Idris Types.idr if needed |
| D14 | String length constraints undocumented | Idris, EventCatalog | Info | Add maxLength annotations where appropriate |
| D15 | Flow branching notation incomplete in SessionLifecycle | EventCatalog | Info | Add split/parallel paths for logout vs expiry |
| D16 | Beads epic ironstar-2it referenced but not found | Beads | Info | Verify epic ID or update reference |
| D17 | Composite key constraints (User.provider+externalId) not in schema | Idris, EventCatalog | Info | Document pattern for consuming code |
| D18 | CatalogMetadata nested structure missing from event schema | Idris, EventCatalog | Warning | Add nested object to CatalogMetadataRefreshed |

## Layer-Specific Corrections

### Idris Specifications

Priority corrections for `/Users/crs58/projects/rust-workspace/ironstar/spec/`:

- [ ] **D1**: Update Timestamp documentation to clarify JSON boundary uses ISO 8601 strings; internal representation remains Integer ms
- [ ] **D3**: Add explicit documentation that NoSession is the initial state before any SessionCreated event
- [ ] **D8**: Add smart constructor for GridSize ensuring width >= 1 and height >= 1
- [ ] **D13**: Evaluate whether RevocationReason, QueryErrorCode, SessionStatus enums are needed
- [ ] **D14**: Document string length constraints for Name, SqlQuery, UiState fields

### Qlerify Event Model

Priority corrections for `/Users/crs58/projects/rust-workspace/ironstar/data/ironstar-main-data-analysis-workflow.json`:

- [ ] **D2**: Add Todo aggregate with TodoCreated, TodoCompleted, TodoUncompleted, TodoDeleted events (if in scope)
- [ ] **D5**: Document that Qlerify uses space-separated names for visual modeling; transformation to PascalCase occurs during EventCatalog generation
- [ ] **D12**: Enrich 210 fields with dataType annotations (26 Command schemas, 26 Query schemas)

### EventCatalog

Priority corrections for `/Users/crs58/projects/rust-workspace/ironstar/packages/eventcatalog/`:

- [ ] **D1**: Add documentation note to all timestamp fields explaining ISO 8601 format at JSON boundary
- [ ] **D2**: Create Todo domain with TodoService, TodoCreated/Completed/Uncompleted/Deleted events, Todo entity
- [ ] **D3**: Update Session entity MDX to document NoSession as pre-creation initial state
- [ ] **D4**: Add `minimum: 0` to all Nat-derived integer fields in schema.json files:
  - QuerySessionCompleted: rowCount, durationMs
  - DatasetInfo: tableCount
  - DashboardChartAdded: gridX, gridY, gridWidth, gridHeight
  - CatalogMetadataRefreshed: datasetCount
- [ ] **D6**: Generate entity MDX files for 19 value objects:
  - SessionId, CatalogId, DashboardId, ChartId, TabId, QueryId, SavedQueryId, UserId
  - Timestamp, Duration, SqlStatement, DatasetRef
  - QueryResult, ErrorInfo, ChartConfig, Position, UiState, Theme, CatalogMetadata
- [ ] **D7**: Create schema.json files for ChartType, CatalogState, ChartOptions entities
- [ ] **D10**: Add ChartPlacement field mapping to DashboardChartAdded event documentation
- [ ] **D15**: Update SessionLifecycle flow with branching notation for logout vs expiry paths
- [ ] **D18**: Add CatalogMetadata nested object definition to CatalogMetadataRefreshed schema

### Architecture Docs

Priority corrections for `/Users/crs58/projects/rust-workspace/ironstar/docs/notes/architecture/`:

- [ ] **D1**: Add section to event-sourcing-core.md documenting timestamp serialization boundary (ISO 8601 JSON ↔ Integer ms internal)
- [ ] **D2**: Clarify Todo aggregate scope in bounded-contexts.md (Generic Canary vs. deferred feature)
- [ ] Update documentation-index.md with reference to this consistency report

### Beads Issues

Priority corrections for `.beads/`:

- [ ] **D16**: Verify ironstar-2it epic exists or update cross-references
- [ ] Create new issue: "D1: Align timestamp type representation across layers"
- [ ] Create new issue: "D2: Add Todo aggregate to EventCatalog (if in scope)"
- [ ] Create new issue: "D3: Document NoSession initial state in Session entity"
- [ ] Create new issue: "D4: Add Nat constraints to JSON Schema integer fields"
- [ ] Create new issue: "D6: Generate 19 value object entity MDX files"

## Implementation Roadmap

### Phase 1: Blocking Corrections (Upstream First)

These must be resolved before implementation can proceed safely.

**1.1 Timestamp Type Alignment (D1)**

Decision required: Idris Integer ms vs ISO 8601 string at JSON boundary.

Recommended resolution:
- Keep ISO 8601 in JSON Schema (industry standard for REST/SSE)
- Document that Idris deserialization parses ISO 8601 and converts to internal ms representation
- Add boundary layer conversion functions to Idris spec

Files affected:
- `spec/Core/Event.idr`: Update Timestamp documentation
- `packages/eventcatalog/domains/*/services/*/events/*/schema.json`: Add format notes
- `docs/notes/architecture/cqrs/event-sourcing-core.md`: Add serialization boundary section

**1.2 Todo Aggregate Scope Decision (D2)**

Decision required: Is Todo in MVP scope or a future feature?

Option A (In scope): Add to all layers
- Create `spec/Todo/` with full Decider specification (already exists)
- Add Todo domain to Qlerify export
- Generate EventCatalog artifacts for Todo domain

Option B (Deferred): Document exclusion
- Add comment to `spec/Todo/` marking as "template example, not in MVP"
- Update bounded-contexts.md to clarify Generic Canary role

**1.3 SessionState Initial State (D3)**

Resolution: Document NoSession as pre-materialized state.

Files affected:
- `packages/eventcatalog/domains/Session/entities/Session/index.mdx`: Add "Initial State" section
- `spec/Session/Aggregate.idr`: Add documentation comment

### Phase 2: Type Constraint Corrections (Qlerify → EventCatalog)

**2.1 Nat Constraint Enforcement (D4)**

Add `minimum: 0` to 12 integer fields across 6 event schemas:

```json
{
  "rowCount": {
    "type": "integer",
    "minimum": 0,
    "description": "Number of rows returned by query"
  }
}
```

Affected schemas:
- QuerySessionCompleted/schema.json (rowCount, durationMs)
- DashboardChartAdded/schema.json (gridX, gridY, gridWidth, gridHeight)
- CatalogMetadataRefreshed/schema.json (datasetCount, tableCount)

**2.2 Qlerify Type Enrichment (D12)**

Batch update 210 fields with dataType annotations in Qlerify export.
This enables automated schema generation and type validation.

### Phase 3: Documentation Completeness (EventCatalog)

**3.1 Value Object Entity Generation (D6)**

Generate 19 entity MDX files with property definitions:

Priority 1 (Core identifiers):
- SessionId, UserId, QueryId, DashboardId, SavedQueryId, ChartId, TabId, CatalogId

Priority 2 (Domain types):
- Timestamp, Duration, SqlStatement, DatasetRef, QueryResult, ErrorInfo

Priority 3 (Configuration types):
- ChartConfig, Position, UiState, Theme, CatalogMetadata

**3.2 Missing Schema Files (D7)**

Create schema.json for entities used in event payloads:
- ChartType (enum with 8 variants)
- CatalogState (enum with 2 variants)
- ChartOptions (object with nested properties)

**3.3 Flow Refinements (D15)**

Update SessionLifecycle flow with proper branching notation for:
- Explicit logout (SessionInvalidated)
- TTL expiry (SessionExpired)
- Refresh success (SessionRefreshed)

### Phase 4: Issue Graph Synchronization (Beads)

**4.1 Create Tracking Issues**

```bash
bd create "Align timestamp type representation across layers" --epic ironstar-nyp --priority p0
bd create "Add Todo aggregate to EventCatalog" --epic ironstar-e6k --priority p1
bd create "Document NoSession initial state" --epic ironstar-jqv --priority p0
bd create "Add Nat constraints to JSON Schema" --epic ironstar-nyp --priority p1
bd create "Generate 19 value object entity MDX files" --epic ironstar-apx --priority p2
```

**4.2 Update Dependencies**

Wire new issues into dependency graph:
- Timestamp alignment blocks all event handler implementation
- NoSession documentation blocks Session aggregate implementation
- Nat constraints block schema validation implementation

## Verification Checklist

Post-remediation verification criteria:

- [ ] All Idris enum types have corresponding EventCatalog enum definitions with matching variants
- [ ] All EventCatalog schema.json files validate against JSON Schema Draft 2020-12
- [ ] All timestamp fields document ISO 8601 format in schema.json
- [ ] All integer fields derived from Idris Nat have minimum: 0 constraint
- [ ] All EventCatalog flows match Qlerify parent-child relationships
- [ ] All 6 aggregates (Session, Catalog, QuerySession, Dashboard, SavedQuery, UserPreferences) have complete coverage:
  - Idris Decider specification
  - Qlerify visual model
  - EventCatalog service with events, commands, queries
  - Beads implementation issue
- [ ] Todo aggregate scope is documented (in scope with full coverage OR explicitly deferred)
- [ ] 19 value objects have EventCatalog entity documentation
- [ ] Cross-layer-consistency-report.md added to documentation-index.md

## Appendix: Layer Extraction Summaries

### A. Idris2 Specifications Summary

4 bounded contexts with complete Decider pattern implementations:

| Context | Aggregates | Commands | Events | State Types |
|---------|-----------|----------|--------|-------------|
| Todo | 1 | 4 | 4 | 4 |
| Session | 1 | 3 | 4 | 4 |
| Analytics | 2 | 4 | 6 | 4 |
| Workspace | 3 | 16 | 16 | 3 |

Core abstractions: Decider, View, Saga, Effect patterns with algebraic composition.

### B. Qlerify Event Model Summary

6 aggregates, 26 events, 60 schemas (26 Commands, 26 Queries, 8 Entities).
Type coverage: 16% (84% of fields lack dataType).

### C. EventCatalog Summary

4 domains, 6 services, 31 events, 22 commands, 14 queries, 26 entities, 6 flows, 3 Zenoh channels.
All events have schema.json with typed properties.

### D. Architecture Documentation Summary

59 markdown files across 6 categories.
Complete design-phase specification with bounded context definitions, technology decisions, and critical invariants.
Implementation not yet started.

### E. Beads Issue Graph Summary

6 strategic epics, 44 implementation tasks (228 total including P2/P3).
All tasks in open status.
Critical path: Error types → fmodel traits → EventRepository → Deciders → Handlers.
