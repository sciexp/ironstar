## Context

CAM-15 (migrate-crane-to-crate2nix) landed a build-graph drift regulator under D9: a committed snapshot at `modules/checks/build-graph-snapshot.json`, committed ceilings at `modules/checks/build-graph-baseline.json`, and a pure `build-graph-invariants` flake check that fails CI when the realized snapshot's duplication structure exceeds the committed ceilings.
The snapshot is generated outside the build sandbox by an `apps.build-graph-snapshot` flake app (a `nix derivation show -r` over 16 canonical x86_64-linux roots, normalized by an embedded `python3` script to the hash-free key `(system, logical_pname, version, build_class, profile, member_scope)`), because a flake check cannot invoke recursive nix.

The demotion commits on PR #477 retire the gate.
A CI-red-on-drift regulator is the wrong instrument for build-graph data: build-graph shape is exploratory observability, not a crisp correctness invariant, so the gate forces ceiling bumps on every legitimate dependency change, never catches the churn it nominally guards, and never surfaces the data anywhere a human can explore it.
This change replaces the regulator with an on-demand compendium: the snapshot stays the operating envelope, but the means of inspecting it becomes a deterministic snapshot plus a queryable DuckDB dataset, curated renderings, and a review runbook — never a build failure, never a CI push.

The compendium stacks on PR #477 after the demotion commits relocate the snapshot out of `modules/checks/` and delete the gate.
The render code is authored fresh, informed by — not ported from — the gitignored seed scripts under `logs/graph-viz/` (machine-pinned, stale substrate from CAM-15's exploration).

## Goals / Non-Goals

**Goals:**

Replace the retired `build-graph-invariants` gate with an on-demand, non-gating observability instrument.
Keep the committed snapshot strictly byte-deterministic so two runs over an unchanged substrate produce an identical committed file.
Persist the raw extraction logs once so the snapshot, the edge list, and the DuckDB load share a single extraction rather than re-running `nix derivation show -r` three times.
Produce a queryable DuckDB dataset whose fidelity query reproduces the committed snapshot's `dev_profile_crates_with_multiplicity` value, so the dataset and the snapshot cannot silently diverge.
Produce curated, labeled, reproducible-enough renderings (a member DAG, a crate overview, and five rebuild cones) plus a generated index, separated from the machine-pinned seed substrate.
Provide a `just build-graph-report` recipe and a review runbook with canned queries and a gap checklist, and preserve the retired ceilings as non-gating `reference/ceilings.json` data.

**Non-Goals:**

Re-introducing any CI gate on build-graph shape is out of scope: the compendium never turns CI red and never pushes from CI.
The full observability follow-up program from CAM-15's D9 — per-commit snapshot upload via a herculesCI effect, a per-invocation cache-hit-rate metric on buildbot, a DuckDB time-series over per-commit invariants for trend queries, and fanout serialization to cure the nix-eval-jobs closure-edge over-count — remains deferred.
Reconciling the two edge-count regimes (the build-closure projection and the crate-DAG parse) is explicitly a non-goal: they are incommensurable by design (see edge-semantics labeling below).
Deterministic rendering geometry is a non-goal: rendering layout is non-deterministic and exploratory.

## Decisions

### D1: artifacts inventory (the approved specification, verbatim)

ARTIFACTS: (1) committed snapshot at modules/apps/build-graph-snapshot/snapshot.json (moved out of modules/checks/ by the demotion), byte-deterministic, with provenance.{cargo_nix_sha256, root_drv_paths_sha256}; (2) persisted raw extraction logs/build-graph/raw/<cat>__<name>.json for 16 roots (gitignored); (3) logs/build-graph/edges.ndjson — union edge list, one JSON line per {root, src six-tuple, dst six-tuple}, reusing normalize.py's inputs.drvs-based root_edges (the snapshot's own projection — NOT the env.dependencies crate-DAG parse); (4) logs/build-graph/build-graph.duckdb with nodes(node_id, system, pname, version, build_class, profile, member_scope, drv_hash_count), edges(root, src_node_id, dst_node_id), root_membership; (5) logs/build-graph/graph-viz/ renderings: member-dag (dot, role-colored), crate-overview (sfdp, fan-in-sized), five rebuild cones (adler2, zenoh, tokio, libduckdb-sys, sqlx-core; reverse-reachability over inputs.drvs, transitive-reduced, rankdir=BT, seed red / members green / binary blue) — SVG primary + PNG, PDF only if dot -Tpdf probe passes; plus generated index.md; (6) committed reference/ceilings.json — the retired baseline ceilings as non-gating reference data.

### D2: edge-semantics labeling (the approved specification, verbatim)

EDGE SEMANTICS LABELING (mandatory): edges.ndjson and the DuckDB edges are the inputs.drvs build-closure projection (~3.6k-30k logical edges per root); the member-dag/crate-overview renderings use the env.dependencies/buildDependencies crate-DAG parse (~1371 edges). They are incommensurable by design; every artifact and the index must state which semantics it carries, and no count reconciliation between them is expected.

The two regimes are grounded in the realized snapshot: `modules/checks/build-graph-snapshot.json` records per-root `logical_edges` ranging from 3664 (`checks.cargo-nix-lock-sync`) to 30676 (`checks.workspace-test`), which is the `inputs.drvs` build-closure projection that `edges.ndjson` and the DuckDB `edges` table carry.
The crate-DAG parse (`env.dependencies`/`buildDependencies`, ~1371 edges) is a different relation entirely, used only by the member-dag and crate-overview renderings.
A reader who tries to reconcile the two counts is making a category error; the labeling requirement exists to prevent exactly that.

### D3: determinism stance (the approved specification, verbatim)

DETERMINISM STANCE: snapshot strictly byte-deterministic; raw/edges/duckdb content-deterministic but gitignored; rendering geometry non-deterministic (acceptable, exploratory).

The committed `snapshot.json` is the only strictly-deterministic artifact: sorted keys, no store hashes, no timestamps, so two runs over an unchanged substrate produce a byte-identical committed file (this is the mechanically verifiable determinism scenario in the spec).
The raw extraction logs, the edge list, and the DuckDB file are content-deterministic (the same substrate yields the same content) but gitignored, so byte-identity of the file-on-disk is not asserted for them.
Rendering geometry is non-deterministic: sfdp and dot layout coordinates vary run to run, which is acceptable because the renderings are exploratory review aids, not gated artifacts.

### D4: render code provenance (the approved specification, verbatim)

RENDER CODE PROVENANCE: authored fresh, informed by the gitignored seed scripts in logs/graph-viz/ (machine-pinned, stale substrate — not ported).

The seed scripts under `logs/graph-viz/` (`build_cones.py`, `gen_dot.py`, `gen_dot2.py`, `extract.py`) are CAM-15-era exploration: machine-pinned, stale, and gitignored.
The compendium's render code is authored fresh against the persisted `logs/build-graph/raw/` extraction and the DuckDB dataset, informed by the seed scripts' approach (reverse-reachability cones, transitive reduction, role-coloring) but not ported line-for-line.
This keeps the compendium's render code coherent with the new `logs/build-graph/` layout and the labeled edge semantics rather than inheriting the seed substrate's pinned assumptions.

### D5: shared raw extraction (C5)

The raw extraction is persisted once at `logs/build-graph/raw/<cat>__<name>.json` for the 16 canonical roots, so the snapshot projection, the edge list, and the DuckDB load all read the same extraction rather than each invoking `nix derivation show -r` independently.
The snapshot refactor that introduces shared reuse must leave the committed `snapshot.json` byte-identical to the pre-refactor file: the refactor changes where the extraction is read from, not what the projection computes.
The 16 roots are the canonical Rust-core set already pinned in `normalize.py`'s `CANONICAL_ROOTS` (the two `packages.ironstar*` roots plus `workspace-clippy`, `workspace-test`, `cargo-nix-lock-sync`, and the 11 per-member `*-test` checks).

### D6: DuckDB dataset shape (C6)

The compendium app emits `logs/build-graph/build-graph.duckdb` with three tables.
`nodes(node_id, system, pname, version, build_class, profile, member_scope, drv_hash_count)` is the hash-free node table keyed on the same six-tuple as `normalize.py`'s `logical_key`, with `drv_hash_count` being the distinct-store-hash multiplicity that the snapshot's duplication counts derive from.
`edges(root, src_node_id, dst_node_id)` is the `inputs.drvs` build-closure projection, one row per `(root, src, dst)`, the same relation `edges.ndjson` carries.
`root_membership` records which roots a node participates in.
The fidelity contract: a DuckDB query over `nodes` reproduces the committed snapshot's `dev_profile_crates_with_multiplicity` value (202 in the current realized snapshot — the count of dev-profile crate-compile logical keys with `drv_hash_count > 1`), so the dataset cannot silently diverge from the snapshot it is loaded from.

### D7: renderings and index (C7)

Five rendering families plus a generated `index.md` land under `logs/build-graph/graph-viz/`.
The member-dag is a `dot` rendering over the crate-DAG parse, role-colored.
The crate-overview is an `sfdp` rendering over the crate-DAG parse, fan-in-sized.
The five rebuild cones (adler2, zenoh, tokio, libduckdb-sys, sqlx-core) are reverse-reachability over the `inputs.drvs` build-closure projection, transitive-reduced, `rankdir=BT`, with the seed crate colored red, workspace members green, and the binary blue.
Each cone and the two crate-DAG renderings carry SVG (primary) and PNG; PDF is emitted only if a `dot -Tpdf` probe passes.
The generated `index.md` states, per artifact, which edge semantics it carries (build-closure projection vs crate-DAG parse), satisfying the mandatory labeling requirement.

### D8: runbook, recipe, and reference ceilings (C8)

A `just build-graph-report` recipe dispatches the compendium (extraction, snapshot, edge list, DuckDB load, renderings, index).
A review runbook carries at least four canned DuckDB queries (e.g. heaviest distinct-compile crates, dev/release duplication split, per-member test-variant uniques, per-root node/edge counts) and a four-item gap checklist for the human reviewer.
The retired ceilings from `modules/checks/build-graph-baseline.json` are preserved verbatim as `reference/ceilings.json`: non-gating reference data that records the accepted-as-of-CAM-15 duplication levels for a reviewer to compare against, without failing any build.

## Risks / Trade-offs

[Risk] Snapshot drift during the C5 refactor (highest).
Moving the extraction to a shared `logs/build-graph/raw/` source and relocating the snapshot to `modules/apps/build-graph-snapshot/snapshot.json` must not change the committed snapshot's bytes.
→ Mitigation: C5 acceptance asserts the snapshot is byte-identical after the refactor (sha256 evidence) and that all 16 raw files persist.

[Risk] DuckDB/snapshot divergence.
The DuckDB dataset is loaded from the same extraction as the snapshot, but a load bug could let the dataset and the snapshot disagree on duplication.
→ Mitigation: C6 acceptance runs the fidelity query and asserts it reproduces `dev_profile_crates_with_multiplicity=202`, the committed snapshot's value.

[Risk] Edge-semantics confusion.
A reviewer or downstream consumer could try to reconcile the ~3.6k-30k build-closure edge counts with the ~1371 crate-DAG edge count and conclude something is wrong.
→ Mitigation: the mandatory labeling requirement (D2) is enforced per artifact and in the index; no reconciliation is expected or attempted.

[Risk] Rendering non-determinism mistaken for a regression.
sfdp/dot geometry varies run to run; a reviewer could mistake layout churn for a graph change.
→ Mitigation: the determinism stance (D3) documents that rendering geometry is non-deterministic and exploratory; only the snapshot is byte-deterministic.

[Trade-off] Losing the automatic gate.
The retired regulator failed CI on duplication regrowth; the compendium does not.
→ Accepted: the gate never caught real churn and forced ceiling bumps on legitimate changes; the runbook plus the preserved `reference/ceilings.json` give a human the same comparison without a brittle build failure.

## Migration Plan

The compendium stacks on PR #477 after the demotion commits relocate the snapshot and delete the gate.
The work is four commits.
C5 persists the raw extraction for shared reuse (snapshot byte-identical after the refactor; 16 raw files persist).
C6 adds the compendium app, the edge list, and the DuckDB dataset (app exits 0; DuckDB fidelity query reproduces `dev_profile_crates_with_multiplicity=202`).
C7 adds the renderings and the index (member-dag + crate-overview + five cones rendered with an index stating edge semantics).
C8 adds the `just build-graph-report` recipe, the runbook, and the reference ceilings (recipe dispatches; runbook has at least four canned queries and a four-item gap checklist; ceilings preserved as non-gating reference).

## Open Questions

These are recorded for the human go/no-go.

Snapshot relocation ownership: the demotion commits on PR #477 are assumed to move `snapshot.json` to `modules/apps/build-graph-snapshot/snapshot.json` and delete `build-graph-invariants.{nix,py}` plus `build-graph-baseline.json` from `modules/checks/`.
If the demotion lands the snapshot elsewhere, C5 must target that path.

Reference ceilings fidelity: `reference/ceilings.json` preserves the retired `build-graph-baseline.json` verbatim.
If the baseline's ceiling values are stale relative to the post-demotion realized graph, the reference is still recorded as-retired (non-gating), and refreshing it is a follow-up.
