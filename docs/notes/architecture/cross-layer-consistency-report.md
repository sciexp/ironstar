# Ironstar Cross-Layer Consistency Report

Generated: 2026-01-15

## Executive Summary

This audit validated consistency across five architectural layers: Idris specifications, Qlerify event model, EventCatalog documentation, architecture docs, and beads issue graph.
The layers demonstrate strong alignment on core event model semantics with actionable gaps in type refinement, invariant enforcement, and implementation coverage.

Key findings:
- **22/26 events** fully synchronized across all three specification layers (Idris, Qlerify, EventCatalog)
- **4 Todo events** intentionally excluded from Qlerify (example aggregate)
- **19 value objects** in Qlerify lack EventCatalog entity documentation
- **23% implementation progress** (57/242 issues closed)
- **154 ready-to-work tasks** with no blocking dependencies

## Discrepancy Matrix

| ID | Discrepancy | Layers | Severity | Resolution |
|----|-------------|--------|----------|------------|
| D1 | Todo aggregate missing from Qlerify | Idris, Qlerify | Info | Intentional - example aggregate |
| D2 | UiStateUpdated.value type: String vs Json | Idris, Qlerify | Warning | Align on Json type in Idris |
| D3 | Enum case convention: PascalCase vs lowercase | Idris, EventCatalog | Warning | Document serialization normalization |
| D4 | Timestamp: epoch ms vs ISO 8601 | Idris, EventCatalog | Warning | Document conversion at boundary |
| D5 | UserId: composite vs UUID surrogate | Idris, EventCatalog | ~~Blocking~~ Resolved | Adopted composite key (provider, externalId) - see ironstar-2it.23 |
| D6 | 19 value objects missing from EventCatalog | Qlerify, EventCatalog | Warning | Generate entity documentation |
| D7 | Flow linearization (siblings as sequential) | Qlerify, EventCatalog | Warning | Add branching notation to flows |
| D8 | GridSize allows 0 in Idris, requires 1 in EC | Idris, EventCatalog | Blocking | Add smart constructor in Idris |
| D9 | Missing userId on Todo mutation events | Idris, EventCatalog | Warning | Add userId field for audit |
| ~~D10~~ | ~~Missing Session/Query/Dashboard/Prefs event types in Idris~~ | ~~Idris, EventCatalog~~ | ~~Blocking~~ | Report error: All Idris specs exist and are complete |
| D11 | Session/Workspace aggregates not in Rust code | Architecture, Code | Blocking | Implement aggregates |
| D12 | 8-layer crate architecture planned but not implemented | Architecture, Code | Info | Single-crate acceptable for now |
| D13 | RevocationReason/QueryErrorCode enums missing | Idris, EventCatalog | Warning | Add to Idris Types.idr |
| D14 | String length constraints undocumented in Idris | Idris, EventCatalog | Warning | Add refined types or invariant comments |

## Layer-Specific Corrections

### Idris Specifications

Location: `/Users/crs58/projects/rust-workspace/ironstar/spec/`

- [x] **D5**: ~~Resolve UserId identity model~~ Resolved: composite key (provider, externalId) adopted
- [ ] **D8**: Add smart constructor for GridSize enforcing width >= 1 and height >= 1
- [x] ~~**D10**: Add event type definitions~~ Report error: All specs exist (Session.idr, Dashboard.idr, Preferences.idr, Catalog.idr, QuerySession.idr, SavedQuery.idr)
- [ ] **D13**: Add `RevocationReason` enum (UserLogout, AdminAction, SecurityConcern)
- [ ] **D13**: Add `QueryErrorCode` enum (SyntaxError, PermissionDenied, Timeout, ResourceExhausted, InternalError)
- [ ] **D13**: Add `SessionStatus` enum (Active, Expired, Revoked)
- [ ] **D13**: Add `QueryStatus` enum (Submitted, Running, Completed, Failed)
- [ ] **D9**: Add userId field to TodoCompleted, TodoUncompleted, TodoDeleted events
- [ ] **D2**: Change UiStateUpdated value field from String to JsonValue type
- [ ] **D14**: Add string length constraints via smart constructors (TodoTitle 1-500, DashboardTitle 1-200, TabTitle 1-100)

### Qlerify Event Model

Location: `/Users/crs58/projects/rust-workspace/ironstar/data/ironstar-main-data-analysis-workflow.json`

- [ ] Add explicit enum value definitions to schema type descriptions (OAuthProvider, Theme, ChartType)
- [ ] Add field descriptions to event schemas (currently null on 84% of fields)
- [ ] Consider modeling Todo aggregate if needed for completeness

### EventCatalog

Location: `/Users/crs58/projects/rust-workspace/ironstar/packages/eventcatalog/`

- [ ] **D6**: Create entity documentation for 19 value objects:
  - SessionId, CatalogId, DashboardId, ChartId, TabId, QueryId, SavedQueryId, UserId
  - Timestamp, Duration, SqlStatement, DatasetRef
  - QueryResult, ErrorInfo, ChartConfig, Position, UiState, Theme, CatalogMetadata
- [ ] **D7**: Update flow definitions to represent branching:
  - SessionLifecycle: Split into logout/timeout paths or add parallel notation
  - SavedQueryManagement: Document mutations as independent operations
  - UserPreferencesSetup: Show theme/catalog/ui as parallel options
- [ ] **D3**: Add serialization notes documenting case convention (Idris PascalCase -> JSON lowercase)
- [ ] **D4**: Document timestamp format in schema descriptions (ISO 8601 for JSON, epoch ms internally)
- [ ] Add JSON Schema format annotations: uuid for IDs, date-time for timestamps, uri for refs

### Architecture Docs

Location: `/Users/crs58/projects/rust-workspace/ironstar/docs/notes/architecture/`

- [ ] Add reconciliation document noting planned vs actual crate architecture
- [ ] Document Session aggregate implementation status (planned vs in-progress)
- [ ] Document Workspace aggregates (Dashboard, SavedQuery, UserPreferences) implementation status
- [ ] Add fmodel-rust adoption decision (evaluated, deferred, or custom approach chosen)

### Beads Issues

Location: `.beads/issues.jsonl`

New issues needed:

- [x] ~~Create issue: Complete Idris event type definitions~~ Report error: All specs already exist
- [ ] Create issue: Generate EventCatalog value object entities (documentation)
- [x] ~~Create issue: Resolve UserId identity model~~ Created: ironstar-2it.23
- [ ] Create issue: Add Idris refined types for string constraints
- [ ] Update ironstar-r62.4 (AppState): Document as critical blocker for 7+ downstream tasks

Epic coverage gaps to address:

- [ ] No epic for EventCatalog value object documentation
- [x] ~~No epic for Idris specification completion~~ Not needed: all specs exist
- [ ] Consider consolidating orphaned issues (32 standalone tasks) into relevant epics

## Implementation Roadmap

### Phase 1: Upstream Corrections (Idris)

Priority: High - blocks downstream work

1. **~~Resolve UserId identity model (D5)~~** COMPLETED
   - Decision: composite key (provider, externalId) adopted
   - EventCatalog User entity updated (commit 29b6ddf)
   - Issue: ironstar-2it.23

2. **~~Complete event type definitions (D10)~~** REPORT ERROR
   - All Idris specifications already exist and are complete:
   - spec/Session/Session.idr (267 lines) - SessionCommand, SessionEvent, SessionState, sessionDecider
   - spec/Workspace/Dashboard.idr (357 lines) - DashboardCommand, DashboardEvent, DashboardState, dashboardDecider
   - spec/Workspace/Preferences.idr (205 lines) - PreferencesCommand, PreferencesEvent, PreferencesState, preferencesDecider
   - spec/Analytics/Catalog.idr (114 lines), spec/Analytics/QuerySession.idr (192 lines)
   - spec/Workspace/SavedQuery.idr (211 lines)

3. **Add missing enums (D13)**
   - RevocationReason, QueryErrorCode, SessionStatus, QueryStatus to Types.idr

4. **Add smart constructors (D8, D14)**
   - GridSize with positive dimension constraint
   - BoundedString for title/description constraints

### Phase 2: Model Sync (Qlerify)

Priority: Medium - improves documentation quality

1. Add field descriptions to event schemas
2. Add explicit enum value documentation
3. Verify all schemas have proper type annotations

### Phase 3: Documentation Update (EventCatalog)

Priority: Medium - improves documentation quality

1. **Generate value object entities (D6)**
   - Create domains/*/entities/{ValueObject}/index.mdx for 19 types
   - Include properties table and invariants

2. **Update flow definitions (D7)**
   - Add branching documentation or split into path-specific flows

3. **Add format annotations**
   - uuid, date-time, uri formats to schema.json files

### Phase 4: Issue Graph Update (Beads)

Priority: Low - project management

1. Create blocking dependency for AppState struct (ironstar-r62.4)
2. Create new epic: Idris Specification Completion
3. Create new epic: EventCatalog Value Object Documentation
4. Wire orphaned issues to relevant epics

## Verification Checklist

After implementing corrections:

- [ ] All Idris types have corresponding Qlerify schemas
- [ ] All Qlerify events have corresponding EventCatalog events
- [ ] All EventCatalog schema.json files use explicit format annotations
- [ ] All EventCatalog flows document branching behavior correctly
- [ ] All value objects have EventCatalog entity documentation
- [ ] All aggregates have implementation issues in beads
- [x] UserId identity model consistent across all layers (composite key adopted 2026-01-15)
- [ ] Timestamp format documented at all boundaries
- [ ] Enum case conventions documented

## Appendix: Layer Inventories

### A. Idris Specifications (7 aggregates)

| Aggregate | Context | Events | Commands | Status |
|-----------|---------|--------|----------|--------|
| Session | Session | 4 | 3 | Complete |
| Catalog | Analytics | 2 | 2 | Complete |
| QuerySession | Analytics | 4 | 2 | Complete |
| Dashboard | Workspace | 6 | 6 | Complete |
| SavedQuery | Workspace | 5 | 5 | Complete |
| UserPreferences | Workspace | 5 | 5 | Complete |
| Todo | (Example) | 4 | 4 | Complete |

### B. Qlerify Event Model (6 aggregates, 26 events)

| Aggregate | Events | Schemas | Type Coverage |
|-----------|--------|---------|---------------|
| Session | 4 | 8 | 16% |
| Catalog | 2 | 4 | 16% |
| QuerySession | 4 | 8 | 16% |
| Dashboard | 6 | 12 | 16% |
| SavedQuery | 5 | 10 | 16% |
| UserPreferences | 5 | 10 | 16% |

### C. EventCatalog (3 domains, 25 events)

| Domain | Services | Events | Entities | Flows |
|--------|----------|--------|----------|-------|
| Session | 1 | 4 | 2 | 1 |
| Analytics | 2 | 6 | 2 | 2 |
| Workspace | 3 | 17 | 4 | 3 |

### D. Beads Issue Graph (15 epics, 208 tasks)

| Epic | Tasks | Closed | Progress |
|------|-------|--------|----------|
| ironstar-2it (Event Modeling) | 22 | 21 | 95% |
| ironstar-2nt (Domain Layer) | 17 | 8 | 47% |
| ironstar-nyp (Event Sourcing) | 40 | 8 | 20% |
| ironstar-6lq (Rust Workspace) | 9 | 9 | 100% |
| ironstar-cxe (Template) | 5 | 5 | 100% |
| ironstar-r62 (Presentation) | 17 | 0 | 0% |
| ironstar-ny3 (Frontend Build) | 18 | 0 | 0% |
| ironstar-jqv (Auth) | 12 | 0 | 0% |
| Other epics | 68 | 6 | 9% |

### E. Critical Blockers

| Issue | Blocks | Status | Description |
|-------|--------|--------|-------------|
| ironstar-r62.4 | 7 | Open | Define AppState struct |
| ironstar-09r | 6 | Open | ds-echarts web component |
| ironstar-e6k.8 | 5 | Open | Todo route mounting |
| ironstar-753 | 4 | Open | Third-party integration |
| ironstar-jqv.1 | 4 | Open | GitHub OAuth provider |
| ironstar-nyp.27 | 4 | Open | ZenohEventBus struct |
