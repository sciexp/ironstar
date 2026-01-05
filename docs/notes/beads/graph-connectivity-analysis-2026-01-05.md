# Ironstar Beads Graph Connectivity Analysis

**Date:** 2026-01-05
**Working directory:** `~/projects/rust-workspace/ironstar/`

## Executive Summary

The ironstar beads issue graph shows **excellent connectivity** with 195 out of 199 active issues (98%) integrated into the main dependency graph. Only 4 reference documentation issues are intentionally orphaned with no dependencies or dependents.

## Graph Statistics

| Metric | Count |
|--------|-------|
| Total nodes | 217 |
| Active nodes (excluding tombstones) | 199 |
| Total edges | 417 |
| Connected components | 5 |
| **Main component size** | **195 issues** |
| Disconnected clusters (size > 1) | **0** |
| Orphaned issues (no deps, no blocks) | **4** |

## Analysis Results

### ✅ Main Dependency Graph: Highly Connected

The main dependency graph contains **195 out of 199 active issues (98%)**, organized under 12 epics:

| Epic ID | Priority | Title |
|---------|----------|-------|
| ironstar-2nt | P0 | Domain layer |
| ironstar-3gd | P0 | Scientific Data Integration |
| ironstar-753 | P0 | Third-party library integration |
| ironstar-f8b | P0 | Process compose integration |
| ironstar-ny3 | P0 | Frontend build pipeline |
| ironstar-nyp | P0 | Event sourcing infrastructure |
| ironstar-r62 | P0 | Presentation layer |
| ironstar-a9b | P1 | Implement fmodel-rust event sourcing foundation |
| ironstar-jqv | P1 | Authentication and authorization |
| ironstar-zuv | P2 | Testing and integration |
| ironstar-e6k | P3 | Example application (Todo) |
| ironstar-apx | P3 | Documentation and template |

### ✅ No Disconnected Clusters

**Zero multi-issue disconnected clusters were found.** All implementation work is properly wired into the main dependency graph.

### 📋 Orphaned Reference Issues: 4 Found

These are **intentionally standalone** reference documentation issues with no dependencies or dependents:

#### 1. ironstar-53t: Reference: Hoffman's Laws compliance mapping
- **Status:** open
- **Priority:** P2
- **Description:** Ironstar implements Kevin Hoffman's Ten Laws of Event Sourcing. Laws 1-7, 10 are explicit in implementation. Laws 8-9 (process managers) are deferred to v2.
- **Reference:** `docs/notes/architecture/cqrs/event-sourcing-core.md`
- **Recommendation:** **LINK to ironstar-nyp** (Event sourcing infrastructure epic)
- **Rationale:** The event sourcing epic (ironstar-nyp) description already includes "Hoffman Laws coverage" section. This reference issue provides cross-reference to the detailed compliance mapping document.

#### 2. ironstar-9dh: Reference: Bounded context patterns
- **Status:** open
- **Priority:** P2
- **Description:** Ironstar v1 operates as single bounded context with implicit internal boundaries (Session, Todo, Analytics). ACL patterns documented for future decomposition.
- **Reference:** `docs/notes/architecture/core/bounded-contexts.md`
- **Recommendation:** **LINK to ironstar-2nt** (Domain layer epic)
- **Rationale:** The domain layer epic (ironstar-2nt) description includes "Strategic classification" for QuerySession (Core) and Todo (Generic). This reference issue documents the bounded context structure.

#### 3. ironstar-k94: Reference: Strategic domain classification
- **Status:** open
- **Priority:** P2
- **Description:** Core: QuerySession (analytics). Supporting: Session, Auth. Generic: Todo (example), ES infrastructure.
- **Reference:** `docs/notes/architecture/core/architecture-decisions.md § Strategic domain classification`
- **Recommendation:** **LINK to ironstar-2nt** (Domain layer epic)
- **Rationale:** Same as ironstar-9dh — this reference issue provides the strategic domain classification (Core/Supporting/Generic) that structures the domain layer.

#### 4. ironstar-sj6: Reference: DDD Starter Modelling Process integration
- **Status:** open
- **Priority:** P2
- **Description:** Ironstar follows the 8-step DDD Starter Modelling Process adapted for algebraic FDM. Mapping of EventStorming artifacts to type system.
- **Reference:** `docs/notes/architecture/core/discovery-and-specification.md`
- **Recommendation:** **LINK to ironstar-2nt** (Domain layer epic)
- **Rationale:** The domain layer epic (ironstar-2nt) description includes "Discovery grounding" section mapping EventStorming stickies (yellow/orange/blue) to aggregate modules, events, and commands. This reference issue documents the full DDD discovery process.

## Recommendations

### Option A: Link Reference Issues to Relevant Epics

Create dependency relationships to integrate reference issues into the main graph:

```bash
# Link Hoffman's Laws reference to Event sourcing epic
bd dep add ironstar-53t ironstar-nyp --type documents

# Link Bounded context patterns to Domain layer epic
bd dep add ironstar-9dh ironstar-2nt --type documents

# Link Strategic domain classification to Domain layer epic
bd dep add ironstar-k94 ironstar-2nt --type documents

# Link DDD Starter Modelling Process to Domain layer epic
bd dep add ironstar-sj6 ironstar-2nt --type documents
```

**Benefits:**
- Integrates reference documentation into main dependency graph (195 → 199 issues, 100% connectivity)
- Makes architectural references discoverable when viewing epic graphs
- Preserves reference issues as trackable work items

**Trade-offs:**
- Adds edges that represent "documents" rather than "depends on" or "blocks"
- Reference issues don't represent implementation work, so they don't naturally fit dependency semantics

### Option B: Keep Reference Issues as Orphans

Leave reference issues disconnected from the main graph.

**Benefits:**
- Avoids polluting dependency graph with non-implementation relationships
- Reference issues serve as index/navigation aids, not blockers
- Simpler graph semantics (only true dependencies)

**Trade-offs:**
- Reference issues remain invisible when viewing epic graphs
- Orphan detection tools will always report these 4 issues

### Option C: Convert to Labels or Close After Documentation

1. Add labels to relevant epics (e.g., `hoffman-laws`, `bounded-contexts`, `ddd-process`)
2. Close reference issues once documentation is stable
3. Use `bd list --label` to find issues by architectural concern

**Benefits:**
- Removes orphans entirely
- Shifts architectural cross-references to label-based navigation
- Reduces issue count

**Trade-offs:**
- Loses explicit tracking of architectural documentation status
- Labels are less discoverable than issues in graph views

## Recommended Action Plan

**Adopt Option A** for maximum traceability:

```bash
cd ~/projects/rust-workspace/ironstar

# Link architectural reference issues to their parent epics
bd dep add ironstar-53t ironstar-nyp --type documents
bd dep add ironstar-9dh ironstar-2nt --type documents
bd dep add ironstar-k94 ironstar-2nt --type documents
bd dep add ironstar-sj6 ironstar-2nt --type documents

# Commit beads changes
bd hooks run pre-commit
git add .beads/issues.jsonl
git commit -m "chore(beads): link reference issues to parent epics for graph integration"
```

This achieves **100% graph connectivity** while preserving reference issues as first-class trackable items.

## Additional Notes

### "task" Placeholder Issues

Four issues with title "task" were also identified:
- **ironstar-0ha**: Documents SSE projection function semantics (depends on ironstar-r62 Presentation layer)
- **ironstar-2vp**: Tests for bitemporal semantics and SSE reconnection (blocks ironstar-zuv Testing epic)
- **ironstar-72q**: Verify Zenoh key expression filtering (blocks ironstar-zuv Testing epic)
- **ironstar-a1s**: Verify catamorphism uniqueness using property-based tests (blocks ironstar-zuv Testing epic)

**Status:** These are **already integrated into the main dependency graph** via blocking relationships with ironstar-zuv (Testing and integration epic). No action required.

**Recommendation:** Consider renaming these issues from "task" to their first line of description for better discoverability:
- ironstar-0ha → "Document SSE projection function semantics"
- ironstar-2vp → "Test bitemporal semantics and SSE reconnection"
- ironstar-72q → "Verify Zenoh key expression filtering preserves monoid structure"
- ironstar-a1s → "Verify catamorphism uniqueness with property-based tests"

## Conclusion

The ironstar beads graph is **exceptionally well-structured** with only 4 intentionally orphaned reference issues. Linking these reference issues to their parent epics (Option A) would achieve 100% connectivity while maintaining clear architectural documentation trails.
