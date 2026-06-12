<!--
These checkboxes are the authoritative human-in-the-loop ledger for the apply phase.
The first checked box fires the board's Todo-to-In-Progress transition, so every box is authored unchecked.
The four tasks map to commits C5 through C8, stacked on PR #477 after the demotion commits; jj mode (anonymous chains, no worktrees).
-->

## 1. C5 — persist raw extraction for shared reuse

Goal: relocate the committed snapshot to `modules/apps/build-graph-snapshot/snapshot.json` (where the PR #477 demotion moves it) and refactor the `build-graph-snapshot` app so the `nix derivation show -r` extraction is persisted once to `logs/build-graph/raw/<cat>__<name>.json` for the 16 canonical roots, shared by the snapshot projection, the edge list, and the DuckDB load.
The refactor changes where the extraction is read from, not what the projection computes.
Depends on: the PR #477 demotion commits.

- [ ] 1.1 Refactor the app to extract once into `logs/build-graph/raw/<cat>__<name>.json` for the 16 canonical roots, then project the committed snapshot from the persisted raw rather than re-extracting.
- [ ] 1.2 Add `provenance.{cargo_nix_sha256, root_drv_paths_sha256}` to the committed snapshot.
- [ ] 1.3 Gitignore `logs/build-graph/raw/`.
- [ ] 1.4 Acceptance: the committed `snapshot.json` is byte-identical after the refactor (sha256 evidence before/after); all 16 raw files persist under `logs/build-graph/raw/`.

## 2. C6 — compendium app, edge list, and DuckDB dataset

Goal: extend the app to emit `logs/build-graph/edges.ndjson` (the union edge list, one JSON line per `{root, src six-tuple, dst six-tuple}`, reusing `normalize.py`'s `inputs.drvs`-based `root_edges` projection — the build-closure projection, not the crate-DAG parse) and `logs/build-graph/build-graph.duckdb` with `nodes(node_id, system, pname, version, build_class, profile, member_scope, drv_hash_count)`, `edges(root, src_node_id, dst_node_id)`, and `root_membership`.
Depends on: 1.

- [ ] 2.1 Emit `logs/build-graph/edges.ndjson` from the persisted raw via the `inputs.drvs`-based `root_edges` projection; label it as the build-closure projection.
- [ ] 2.2 Load `logs/build-graph/build-graph.duckdb` with the `nodes`, `edges`, and `root_membership` tables.
- [ ] 2.3 Gitignore `logs/build-graph/edges.ndjson` and `logs/build-graph/build-graph.duckdb`.
- [ ] 2.4 Acceptance: the app exits 0; the DuckDB fidelity query over `nodes` reproduces the committed snapshot's `dev_profile_crates_with_multiplicity=202`.

## 3. C7 — renderings and index

Goal: emit the curated renderings under `logs/build-graph/graph-viz/` — a member-dag (`dot`, role-colored, crate-DAG parse), a crate-overview (`sfdp`, fan-in-sized, crate-DAG parse), and five rebuild cones (adler2, zenoh, tokio, libduckdb-sys, sqlx-core; reverse-reachability over the `inputs.drvs` build-closure projection, transitive-reduced, `rankdir=BT`, seed red / members green / binary blue) — SVG primary + PNG, PDF only if a `dot -Tpdf` probe passes; plus a generated `index.md`.
Render code is authored fresh, informed by (not ported from) the gitignored seed scripts in `logs/graph-viz/`.
Depends on: 2.

- [ ] 3.1 Render the member-dag and crate-overview from the crate-DAG parse.
- [ ] 3.2 Render the five rebuild cones from the build-closure projection (reverse-reachability, transitive-reduced, `rankdir=BT`, role-colored), SVG + PNG, PDF gated on the `dot -Tpdf` probe.
- [ ] 3.3 Generate `index.md` stating, per artifact, which edge semantics it carries (build-closure projection vs crate-DAG parse).
- [ ] 3.4 Gitignore `logs/build-graph/graph-viz/`.
- [ ] 3.5 Acceptance: the member-dag, the crate-overview, and the five cones render with a generated index; each artifact and the index state their edge semantics.

## 4. C8 — justfile recipe, runbook, and reference ceilings

Goal: add a `just build-graph-report` recipe dispatching the full compendium; write a review runbook with at least four canned DuckDB queries and a four-item gap checklist; preserve the retired `modules/checks/build-graph-baseline.json` ceilings verbatim as non-gating `reference/ceilings.json`.
Depends on: 3.

- [ ] 4.1 Add the `just build-graph-report` recipe wrapping the compendium app.
- [ ] 4.2 Write the review runbook: at least four canned DuckDB queries plus a four-item gap checklist.
- [ ] 4.3 Preserve the retired baseline ceilings as `reference/ceilings.json` (non-gating reference data).
- [ ] 4.4 Acceptance: `just build-graph-report` dispatches; the runbook has at least four canned queries and a four-item gap checklist; the ceilings are preserved as non-gating reference, with no build-graph check on the check surface and no CI workflow invoking the compendium.
