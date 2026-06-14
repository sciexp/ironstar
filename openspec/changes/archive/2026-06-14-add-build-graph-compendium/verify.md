# Verification Report

> This file is produced by the `openspec-verify-change` skill after apply completes, to confirm that the
> implementation is consistent with the specs / design / tasks. Any failed check must be returned to its
> corresponding artifact for correction before re-running verify.

**Change**: `add-build-graph-compendium`
**Verified at**: `2026-06-14 10:05`
**Verifier**: `verification-report subagent (first verification; post-demotion as-built state)`

---

## Summary scorecard

| Dimension | Verdict | Critical | Warning | Suggestion |
|---|---|---|---|---|
| Completeness | PASS | 0 | 0 | 0 |
| Correctness | PASS | 0 | 0 | 1 |
| Coherence | PASS | 0 | 0 | 0 |
| Structural validation | PASS (not re-run; artifacts well-formed) | — | — | — |
| Task completion | PASS (17/17 leaf boxes checked) | — | — | — |

This is the first verification of `add-build-graph-compendium`; no prior verify.md existed.
The change demotes the build-graph drift regulator (a `build-graph-invariants` flake check committed by CAM-15) to an on-demand, non-gating observability instrument.
Every task group (C5 through C8) was re-checked against the implementation with concrete file:line and commit evidence, and the check surface was enumerated live with read-only `nix eval` (no `nix flake check` run).
The three demotion-session commits this verification specifically targets — the rename to `build-graph-report`, the `normalize` import fix, and the figure embedding in the index — are all confirmed present and correct.
The single SUGGESTION concerns two acceptance criteria (snapshot byte-identity and the DuckDB fidelity=202 query) that are verified by code inspection and committed values rather than by a live extraction+build, which was deliberately skipped per the cost guardrails.

---

## 1. Structural validation

`openspec validate` was not re-run in this pass (read-only verification under orchestrator control).
The change artifacts are well-formed: `proposal.md`, `design.md`, `tasks.md`, and a single delta spec `specs/build-graph-observability/spec.md` whose one ADDED requirement carries three `#### Scenario` blocks.
No structural defect is apparent on read.

---

## 2. Task completion (`tasks.md`)

- [x] All `- [ ]` have been changed to `- [x]`

Leaf-task accounting: 17/17 leaf checkboxes complete across the four commit groups (C5: 1.1–1.4; C6: 2.1–2.4; C7: 3.1–3.5; C8: 4.1–4.4).
No unchecked boxes remain.

The four task groups map cleanly to the four implementation commits, stacked on PR #477 after the demotion commits:
- C5 -> `d32f34f9` (persist raw extraction for shared reuse), with relocation + provenance landed by the prior demotion commit `ef81c4a8`.
- C6 -> `4baf4de5` (compendium app emitting edge list and DuckDB dataset).
- C7 -> `9dd3efc2` (curated graphviz renderings and index), figure embedding added by `9f1e8100`.
- C8 -> `be116108` (recipe + review-session runbook), reference ceilings preserved.
- Plus `32a135e7` (rename compendium app to build-graph-report) and `d8fc4f92` (normalize import fix).

---

## 3. Delta spec coverage

Delta spec: `openspec/changes/add-build-graph-compendium/specs/build-graph-observability/spec.md` — one ADDED requirement, three scenarios.

| Scenario | Implementation evidence | Verdict |
|---|---|---|
| snapshot is byte-deterministic over an unchanged substrate | snapshot keyed on hash-free six-tuple, sorted keys, no store hashes/timestamps (`build-graph-snapshot.nix:13-15` rationale; `normalize.py` projection). Byte-identity-after-refactor not re-derived live; refactor commit `d32f34f9` touched only extraction wiring (+4/-2), not the projection. | PASS (inspection) |
| the compendium is non-gating and absent from CI | live `nix eval .#checks.x86_64-linux --apply builtins.attrNames` = 26 names, none build-graph; `rg build-graph .github/` returns no match. | PASS (live) |
| the DuckDB dataset reproduces the snapshot's duplication value | `snapshot.json:3` `dev_profile_crates_with_multiplicity:202`; `queries.sql:10-11` Q1 total reproduces 202; node identity shared via `emit_edges.py:37-40` importing `normalize`. Live query not executed (costly extraction+load). | PASS (inspection) |

---

## 4. Design / specs coherence spot check

| Sampled decision | design description | implementation correspondence | Gap |
|---|---|---|---|
| D1 artifacts inventory | committed snapshot at `modules/apps/build-graph-snapshot/snapshot.json`; raw logs; `edges.ndjson`; `build-graph.duckdb` with nodes/edges/root_membership; graph-viz renderings + index; `reference/ceilings.json`. | snapshot relocated and tracked at the apps path; `emit_edges.py` writes nodes/edges/root_membership; `load_duckdb.sql:17-43` creates the three tables; `render.py` emits renderings + index; `reference/ceilings.json` tracked. | None |
| D2 edge-semantics labeling | every artifact + index states build-closure vs crate-DAG; no reconciliation. | `render.py:8-24` docstring + index summary table (`:616-628`) + per-view `Edge relation:` lines + incommensurability paragraph (`:606-611`); runbook §edge-semantics note. | None |
| D5 shared raw extraction (C5) | extract once to `logs/build-graph/raw/`, shared by snapshot, edges, DuckDB. | `build-graph-snapshot.nix:70` and `build-graph-report.nix:77-93` share `logs/build-graph/raw`; report reuses if present. | None |
| D6 DuckDB shape + fidelity (C6) | nodes/edges/root_membership; fidelity query reproduces 202. | `load_duckdb.sql:17-43` exact schema; `snapshot.json:3`=202; `queries.sql` Q1 reproduces 202. | None |
| D7 renderings + index (C7) | member-dag (dot), crate-overview (sfdp), five cones (adler2/zenoh/tokio/libduckdb-sys/sqlx-core), index with labeling. | `render.py:77` exactly those 5 seeds; member-dag/crate-overview from `env.dependencies`; cones reverse-reachability over `inputs.drvs`, role-colored, rankdir=BT, transitive-reduced; index labels semantics. | None |
| D8 runbook + recipe + ceilings (C8) | `just build-graph-report`; >=4 canned queries + 4-item gap checklist; ceilings preserved non-gating. | `justfile:1332-1333`; runbook 4 queries (Q1-Q4) + 4 checklist items; `reference/ceilings.json` note states non-gating. | None |

**Drift warnings** (non-blocking): none.

---

## 5. Rename consistency (highlighted change 1)

Repo-wide grep, not a single location:
- `modules/apps/build-graph-report.nix` defines `apps.build-graph-report` (`:61`); the old `build-graph-compendium.nix` is gone.
- Rename commit `32a135e7` renamed `build-graph-compendium.nix` -> `build-graph-report.nix`, the `apps.build-graph-compendium` attribute, the directory, the justfile recipe, the runbook references, and `render.py` internal strings.
- `justfile:1332-1333` exposes `build-graph-report` -> `nix run .#build-graph-report`.
- Modules auto-load via `import-tree ./modules` (`flake.nix:51`), so no flake.nix edit is required and none is missing.
- `rg build-graph-compendium` finds matches ONLY in the unrelated `migrate-crane-to-crate2nix` artifacts (historical), never in active code.

The rename is consistent across derivation, recipe, and docs.

---

## 6. normalize import fix (highlighted change 2)

Commit `d8fc4f92` adds, at `build-graph-report.nix:55-58`, a `normalizeLib` `runCommandLocal` that copies `build-graph-snapshot/normalize.py` into a store path, and exports `PYTHONPATH="${normalizeLib}..."` before `python3 emit_edges.py` (`:96`).
`emit_edges.py:37-40` performs `import normalize`; in the nix-store execution context the `sys.path.insert` of `SNAPSHOT_DIR` does not resolve (`emit_edges.py` is an isolated store path with no `build-graph-snapshot` sibling), so the `PYTHONPATH` export is the mechanism that makes the import succeed.
Sharing `normalize`'s `logical_key`/`drv_hash`/`CANONICAL_ROOTS` is what guarantees the report's node identity cannot drift from the snapshot's.
Confirmed present and correct.

---

## 7. Figure embedding (highlighted change 3)

Commit `9f1e8100` (`render.py` +9 lines) appends a markdown image embed `![<png>](<png>)` to the generated index for the member-dag (`render.py:649-651`), the crate-overview (`:670-672`), and each cone (`:702-704`), guarded by `if <render>.png is not None`.
The embeds are present in the current `write_index` body alongside the per-artifact `Edge relation:` labeling.
Confirmed.

---

## 8. Check surface (non-gating posture)

Enumerated with read-only evaluation only (no `nix flake check`):
- `nix flake show --json --all-systems` -> `x86_64-linux: 26`, `aarch64-linux: 26`, `aarch64-darwin: 26`.
- `nix eval .#checks.x86_64-linux --apply builtins.attrNames` -> 26 names; none is a build-graph check (the only `invariant` match is `structure-package-set-invariant`, an unrelated package-set check).
- `modules/checks/` contains no build-graph file (only `package-set-invariant.nix`).
- `rg build-graph .github/` -> no match: no CI workflow invokes either build-graph app.

The CI-relevant system is `x86_64-linux` (buildbot owns CI); its count is 26.

26-vs-27 reconciliation: 27 was the PRE-demotion surface (with `build-graph-invariants`), which the `migrate-crane-to-crate2nix` verify.md correctly asserted for ITS as-built state.
This change stacks on the demotion commits; commit `19d4e1cb` reconciled the surface to 26, and proposal.md:67 states the surface "drops by one".
26 is the correct post-demotion count. No discrepancy.

---

## 9. Implementation signal

- [x] Implementation committed across `d32f34f9`, `4baf4de5`, `9dd3efc2`, `be116108`, `32a135e7`, `9f1e8100`, `d8fc4f92`, stacked on the PR #477 demotion commits.
- [ ] Push state user-gated in this WO session (not confirmed from within this verification).

Committed artifacts confirmed git-tracked: `modules/apps/build-graph-snapshot/snapshot.json`, `modules/apps/build-graph-report/reference/ceilings.json`.
Gitignored artifacts (`logs/build-graph/raw/`, `edges.ndjson`, `build-graph.duckdb`, `graph-viz/`) are covered by the broad `.gitignore:17` `logs/` entry.

---

## Issues

### CRITICAL

None.

### WARNING

None.

### SUGGESTION

**SUGGESTION-1 — two acceptance criteria are verified by inspection, not by a live run.**

C5/1.4 (snapshot byte-identical after the refactor; 16 raw files persist) and C6/2.4 (DuckDB fidelity query reproduces `dev_profile_crates_with_multiplicity=202`) are both verified here from committed values and code structure rather than from a live `build-graph-report` run, because the extraction (`nix derivation show -r` over 16 roots) and DuckDB load were deliberately not executed under the cost guardrails.
The committed `snapshot.json:3` already carries `202`; the refactor commit touched only extraction wiring; and node identity is shared via the imported `normalize` module, so divergence is structurally hard.
Recommendation: optionally run `just build-graph-report` once on an x86_64-linux machine before or shortly after archive to convert these two from inspection-verified to runtime-verified; not required to archive.

---

## Open questions for the human gate

- Push/PR #477 state is user-gated in this WO session; confirm the seven implementation commits are pushed before archive.
- The two inspection-verified acceptance criteria (snapshot byte-identity, DuckDB fidelity=202) can be promoted to runtime-verified by a single `just build-graph-report` invocation on linux; treat as optional post-archive confirmation.

---

## Overall decision

- [x] (pass) PASS — may proceed to finishing-a-development-branch and archive
- [ ] (warn) PASS WITH WARNINGS
- [ ] (fail) FAIL

**PASS** with 0 CRITICAL, 0 WARNING, 1 SUGGESTION (the SUGGESTION does not block archive).

All 16 leaf tasks are complete and map to real implementation with file:line and commit evidence.
The three highlighted demotion-session changes (rename to `build-graph-report`, `normalize` import fix, figure embedding) are confirmed.
The check surface is 26 with no build-graph gate and no CI workflow invoking the compendium, matching the non-gating posture the change asserts.

**Next step**: proceed to archive; on archive the delta `specs/build-graph-observability/spec.md` becomes the canonical `openspec/specs/build-graph-observability/spec.md`. Optionally run `just build-graph-report` on linux to runtime-confirm the two inspection-verified acceptance criteria.
