---
linear_story_id: 84355c11-3118-424d-8867-3afb27536f96
linear_story_identifier: CAM-16
linear_story_title: "Add an on-demand build-graph compendium replacing the build-graph drift regulator"
linear_story_url: https://linear.app/cameronraysmith/issue/CAM-16/add-an-on-demand-build-graph-compendium-replacing-the-build-graph
linear_story_state: Done
linear_team: CAM
linear_project: ironstar-build-performance
last_synced_state: Done
last_synced_at: 2026-06-14T19:37:26Z
review_round: 0
attempt_log:
  - { at: "2026-06-14T19:37:00Z", transition: "Backlog(bind)", outcome: "posted", note: "T4 archive gate: issue CAM-16 created retroactively, description seeded from proposal.md Why+What Changes" }
  - { at: "2026-06-14T19:37:26Z", transition: "Backlog->Done", outcome: "posted", note: "T4 archive gate: retroactive catch-up straight to Done; work complete and merged in PR #477" }
beads_epic: TBD
---

## Why

The crate2nix migration (CAM-15) committed a build-graph drift regulator: a `build-graph-invariants` flake check that gates a committed snapshot against committed ceilings, failing CI on duplication regrowth.
The demotion commits on PR #477 retire that regulator because a CI-red-on-build-graph-drift gate is the wrong instrument for build-graph data.
Build-graph shape is exploratory observability, not a correctness property: a check that fails the build because a dependency bump added a compile derivation forces ceilings to be bumped on every legitimate change, never catches the churn it nominally guards, and never feeds the data anywhere a human can explore it.
The gate is also asymmetric with its sibling `cargo-nix-lock-sync`, which gates a genuine equality (every `Cargo.lock` package appears in `Cargo.nix`); duplication structure has no such crisp invariant.

This change replaces the removed regulator with an on-demand observability instrument.
The compendium is run by a human (or a non-gating effect) when build-graph data is wanted: it never turns CI red, never pushes anything from CI, and produces a committed deterministic snapshot plus a queryable DuckDB dataset, curated renderings, and a review runbook.
The CCV stance shifts from envelope-plus-regulator (gate) to envelope-plus-instrument (observe): the snapshot remains the operating envelope, but the means of inspecting drift becomes an explorable dataset and a runbook rather than a build failure.

## What Changes

**Build-graph compendium app**
- From: a `build-graph-invariants` flake check gating a committed snapshot against committed ceilings (CI-red on duplication regrowth).
- To: an on-demand `build-graph-snapshot` app that emits a byte-deterministic committed snapshot, persists raw extraction logs for shared reuse, derives a union edge list, and loads a queryable DuckDB dataset.
- Reason: build-graph shape is exploratory observability, not a gating correctness property.
- Impact: the check surface drops the `build-graph-invariants` check; no CI workflow invokes the compendium; nothing is pushed from CI.

**Edge list and DuckDB dataset**
- From: in-memory normalization with derived counts committed to a JSON snapshot only.
- To: a persisted union edge list at `logs/build-graph/edges.ndjson` (one JSON line per root/src/dst, reusing `normalize.py`'s `inputs.drvs`-based `root_edges` projection) and a `logs/build-graph/build-graph.duckdb` with `nodes`, `edges`, and `root_membership` tables.
- Reason: a queryable dataset is the explorable substrate the gate never provided.
- Impact: new gitignored artifacts under `logs/build-graph/`; the committed snapshot stays byte-deterministic.

**Curated renderings and index**
- From: gitignored, machine-pinned seed renderings under `logs/graph-viz/` with no generated index.
- To: a curated `logs/build-graph/graph-viz/` set — a member DAG, a crate overview, and five rebuild cones — plus a generated `index.md`, authored fresh and informed by (not ported from) the stale seed scripts.
- Reason: stable, labeled, reproducible-enough renderings for review, separated from the machine-pinned seed substrate.
- Impact: new gitignored renderings; render geometry is non-deterministic and exploratory by design.

**Review runbook and reference ceilings**
- From: ceilings encoded as a gating baseline (`build-graph-baseline.json`) that fails CI on growth.
- To: a `just build-graph-report` recipe and a review runbook with canned DuckDB queries and a gap checklist, plus the retired ceilings preserved as non-gating `reference/ceilings.json` data.
- Reason: a human-driven review loop replaces the automatic gate; the old ceilings remain useful as reference, not as a gate.
- Impact: the runbook is the review entry point; the ceilings are reference-only.

## Capabilities

### New Capabilities

- `build-graph-observability`: the durable requirements governing the build-graph compendium as an on-demand, non-gating instrument — a byte-deterministic committed snapshot, a queryable DuckDB dataset whose fidelity reproduces the snapshot's duplication value, a non-gating posture (no build-graph check on the check surface, no CI workflow invoking the compendium), and the explicit edge-semantics labeling that distinguishes the `inputs.drvs` build-closure projection from the crate-DAG parse.

### Modified Capabilities

<!-- None: the build-graph regulator was never an openspec/specs/ capability; it lived in CAM-15's rust-build-substrate delta and is retired by the PR #477 demotion commits this change stacks on. -->

## Impact

Build infrastructure: the `build-graph-snapshot` app under `modules/apps/`, the committed snapshot relocated to `modules/apps/build-graph-snapshot/snapshot.json` (moved out of `modules/checks/` by the demotion), a new `just build-graph-report` recipe, and the removal of the `build-graph-invariants` check, its `.nix` and `.py` validator, and the `build-graph-baseline.json` gate from `modules/checks/`.
Artifacts: gitignored `logs/build-graph/raw/`, `logs/build-graph/edges.ndjson`, `logs/build-graph/build-graph.duckdb`, and `logs/build-graph/graph-viz/`; a committed `reference/ceilings.json` carrying the retired baseline as reference data.
Check surface: drops by one (the `build-graph-invariants` check is removed); buildbot nix-eval/nix-build umbrella names are stable, so Mergify required_checks is invariant.
CI: never red on build-graph data and never pushes build-graph artifacts; the compendium is invoked on demand only.

Scope boundaries — this change stacks on PR #477 after the demotion commits land (C5 through C8 on top of the snapshot relocation).
Out of scope: per-commit CI upload of the snapshot via an effect, a per-invocation cache-hit metric, a time-series store over per-commit invariants, and fanout serialization to cure the nix-eval-jobs closure-edge over-count; these remain the deferred observability follow-up program from CAM-15's D9.
