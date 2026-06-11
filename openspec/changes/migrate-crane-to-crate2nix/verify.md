# Verification Report

> This file is produced by the `openspec-verify-change` skill after apply completes, to confirm that the
> implementation is consistent with the specs / design / tasks. Any failed check must be returned to its
> corresponding artifact for correction before re-running verify.

**Change**: `migrate-crane-to-crate2nix`
**Verified at**: `2026-06-11 15:30`
**Verifier**: `verification-report subagent (re-adjudication after reconciliation commit c8ba789)`

---

## Summary scorecard

| Dimension | Verdict | Critical | Warning | Suggestion |
|---|---|---|---|---|
| Completeness | PASS | 0 | 0 | 1 (carried) |
| Correctness | PASS | 0 | 0 | 0 |
| Coherence | PASS | 0 | 0 | 1 (carried) |
| Structural validation | PASS | — | — | — |
| Task completion | PASS (1 documented deferral) | — | — | — |

This is a re-adjudication of the prior checked-FAIL verify.md, not a rubber-stamp.
The reconciliation commit `c8ba789` ("docs(openspec): reconcile change artifacts to as-built state", 2026-06-11) amended the artifacts to the as-built state, and every prior finding was re-checked against the amended files with fresh reads and a live `nix eval` of the check surface.
The single prior CRITICAL (spec.md encoding the superseded monolithic-gate / 14-check / 911-test contract on four counts) is resolved: the delta spec now encodes the per-member `runTests=true` shape, the 27-check surface, and the 933-passed/5-ignored/938-defined envelope, with the contradicted MUST/MUST-NOT assertions removed.
All five prior WARNINGs are resolved; the two carried SUGGESTIONs (frozen `-c2n-member-src` token, single-platform build-graph baseline) now carry deliberate-freeze rationale in design.md and the baseline JSON, and do not block archive.
The implementation remains correct, committed, and CI-green on both platforms (PR 477).

---

## 1. Structural Validation (`openspec validate --all --json`)

- [x] All items report `"valid": true`

**Result**:

```text
openspec validate --all --json:
  items: 1, passed: 1, failed: 0
  byType: change { items: 1, passed: 1, failed: 0 }, spec { items: 0, passed: 0, failed: 0 }
openspec validate "migrate-crane-to-crate2nix":        "Change 'migrate-crane-to-crate2nix' is valid" (exit 0)
openspec validate "migrate-crane-to-crate2nix" --strict: "Change 'migrate-crane-to-crate2nix' is valid" (exit 0)
```

If any items fail, list their id and issues:

| Item | Type | Issues |
|---|---|---|
| — | — | — |

Structural validation passes, including `--strict`.
The amendments did not break the schema shape; every ADDED requirement in spec.md retains at least one `#### Scenario` block, so strict validation is clean.

---

## 2. Task Completion (`tasks.md`)

- [ ] All `- [ ]` have been changed to `- [x]`

Leaf-task accounting: 37 leaf checkboxes complete, 1 leaf checkbox (4.3c) documented-deferred, 1 parent checkbox (4.3) left unchecked solely because of its deferred sub-box.
Any-indent tally of `tasks.md`: 37 `- [x]`, 2 `- [ ]` (lines 62 and 65).
`git log --oneline merge-base..HEAD` returns 31 commits and `grep -c '^- \[x\]'` returns 35 top-level checked boxes, satisfying both implementation-evidence preconditions.

**Incomplete tasks** (if any):

| Task | Reason incomplete | Blocks archive? |
|---|---|---|
| 4.3 (parent, tasks.md:62) | Parent of 4.3c; unchecked only because the one deferred sub-box remains open. 4.3a (local check + RED demo) and 4.3b (actionlint + static workflow validation) are complete and evidenced in measurements.md. | No — tracking marker for the deferred sub-box. |
| 4.3c (tasks.md:65) DEFERRED | Live workflow dry-run (push a test branch with a trivial `Cargo.lock` change, observe auto-regen of `Cargo.nix` + auto-commit by github-actions[bot]) requires a pushed branch, which is user-gated in this WO session. The workflow IS implemented (regenerate-lock-files.yaml triggers on Cargo.lock/Cargo.toml/crates/**/Cargo.toml; regen+amend steps for Cargo.nix and build-graph-snapshot present; concurrency group generalized). The new step reuses the byte-identical github-actions[bot] auto-commit mechanism the flake.lock and bun.nix steps already use in production. Deferral rationale recorded in measurements.md. | No — documented deferral. Static + local-RED coverage (actionlint clean, cargo-nix-lock-sync RED/GREEN demonstrated) plus reuse of the proven auto-commit mechanism is adequate pre-archive coverage. The first post-push CI run touching Cargo.lock is the live confirmation. |

Task completion is acceptable for archive at the human gate: the single open box is a justified, documented deferral, not undone work.

---

## 3. Delta Spec Sync State

For each delta spec file reported by the CLI
(`openspec status --change "migrate-crane-to-crate2nix" --json | jq -r '.artifactPaths.specs.existingOutputPaths[]'`),
compare against the corresponding main capability spec:

Delta spec file: `openspec/changes/migrate-crane-to-crate2nix/specs/rust-build-substrate/spec.md`.
Main specs directory `openspec/specs/` is empty (verified: `ls openspec/specs/` shows no capability subdirectories).
On archive the delta is absorbed into a newly-created `openspec/specs/rust-build-substrate/spec.md`.

| Capability | Sync status | Notes |
|---|---|---|
| rust-build-substrate | pending sync (new capability) | No prior main capability; archive will create the canonical spec from this delta. Re-verified that the delta no longer contradicts the merged code: it now asserts the per-member `runTests=true` test gate, the 27-check surface, and the 933/5/938 envelope (grep of spec.md for `911`, `stay 14`, `is 14`, `exactly 14` returns no match). The verbatim-enshrinement risk that drove the prior CRITICAL is eliminated. Sync at archive is the normal lifecycle step, not a defect vector. |

The delta spec is the only artifact synced to the canonical specs on archive (confirmed via `artifactPaths.specs`); proposal.md, design.md, plan.md, tasks.md, and brainstorm.md are not synced.
The residual historical `911` tokens in tasks.md (acceptance text, which itself notes the figure is stale), plan.md (micro-task plan), and brainstorm.md (rejected-option rationale) therefore do not reach the canonical spec.

---

## 4. Design / Specs Coherence Spot Check

Spot-check whether the decisions in `design.md` are reflected in the Requirements and
Scenarios of `specs/*.md`:

| Sampled item | design description | specs correspondence | Gap |
|---|---|---|---|
| Test gate shape | design.md Goals line 27 + D6 lines 146/155: 11 per-member `runTests` checks + zero-cost `workspace-test` linkFarm aggregate (no nextest). modules/rust.nix:358 (`runTests = true`), :365-370/:406 (linkFarm). | spec.md:79 mandates per-member `(cargoNix.workspaceMembers.<name>.build.override { runTests = true; ... }).passthru.test`; spec.md:80 mandates the `workspace-test` linkFarm aggregate. | None — spec now matches landed code and design. |
| Check surface count | design.md line 17/28/171: lands at 27 (15 prior + 11 per-member + build-graph-invariants); live `nix eval` = 27. | spec.md:99-102 scenario asserts total check count is 27 (15 + 11 + build-graph-invariants). | None — spec asserts 27; live surface is 27. |
| Test baseline | design.md line 27/82-equivalent: 933 passed / 5 ignored (938 defined). | spec.md:82/87 assert 933 passed / 5 ignored over 938 defined. | None — spec matches design and measurements.md:112. |
| Clippy gate | design.md line 192-equivalent: `cargo clippy --profile dev --locked --all-targets -- --deny warnings` over importCargoLock-vendored deps. modules/rust.nix:192, :159. | spec.md:81 + scenario :94-97 assert the same flags over `importCargoLock`-vendored deps, offline. | None — profile/locked/vendoring idiom now in the spec scenario. |
| Cargo.nix lockstep | design.md D7: cargo-nix-lock-sync content-addresses Cargo.lock + Cargo.nix. modules/checks/cargo-nix-lock-sync.nix. | spec.md:3-22 Cargo.nix-lockstep requirement + 3 scenarios. | None — synced and verified green. |
| No-IFD | design.md D1: committed Cargo.nix, no allow-import-from-derivation, no crate-hashes.json. | spec.md:24-38 no-IFD requirement + scenarios. | None — `rg` confirms no IFD/generatedCargoNix/crate-hashes.json residue in active code. |
| build-graph drift regulator | design.md D9: build-graph-invariants regulator + committed baseline/snapshot envelope (x86_64-linux canonical). modules/checks/build-graph-invariants.nix + build-graph-baseline.json + build-graph-snapshot.json. | spec.md:109-135 build-graph envelope-and-regulator requirement + 4 scenarios (lockstep snapshot, growth fails, planned shrinkage passes, 11-member floor). | None — the previously-uncovered regulator now has a first-class requirement. |

**Drift warnings** (non-blocking):

- none. The prior internal inconsistencies are resolved. The `14` and `26` tokens that remain in design.md are now explicitly progression-arithmetic ("entered this change at 14 and lands at 27"; "prior baseline was 14; ... raised it to 15 ... to 26 ... to the final 27"), not contradictory end-state assertions; a grep for bare end-state assertions (`stay 14`, `all 14 checks`, `surface is 14`) returns no match. No `911` token appears anywhere in design.md. D9's canonical roots are now post-swap names (`packages.ironstar`, `packages.ironstar-release`, design.md:231), matching the committed snapshot/baseline artifacts.

---

## 5. Implementation Signal

- [x] No unstaged files in the worktree
- [ ] All related commits have been pushed

`jj st`: the only working-copy change is this `verify.md` (status `A`), which is the verification artifact being (re)written, not implementation code.
Working copy `@` is `wzxxpxlk 6a18ffd4`; parent `@-` is the development-join commit `posqtkut 5b8569d4` ("wip: diamond development join — ejp observability + crate2nix migration").
The crate2nix-migration work and the reconciliation commit `c8ba789` are committed in the join lobe; the COMPLETENESS/CORRECTNESS evidence was gathered against the isolated migration tip.
Push state is user-gated in this WO session (the orchestrator routes commits/pushes); the push checkbox is left unchecked because push is not confirmed from within this verification.
PR 477 CI was reported green on both platforms (buildbot/nix-build, nix-eval, nix-effects all SUCCESS) by the upstream reviews.

**Commit range** (if known): merge-base(HEAD, origin/main)..HEAD = 31 commits, including reconciliation commit `c8ba789` and crane-removal commit `445da5b`; the migration lobe of the development join carries the substrate swap.

---

## 6. Front-Door Routing Leak Detector (warning, non-blocking)

Design output should not land in `docs/superpowers/specs/`.

Detect:

```bash
ls docs/superpowers/specs/*.md 2>/dev/null
```

- [x] No files, or any existing files are legitimate residue from before schema installation

`docs/superpowers/specs/*.md`: no matches (glob returns no files, exit 1). No front-door routing leak.

**Leak list** (if any):

| File | Content captured into change? | Recommended action |
|---|---|---|
| — | — | — |

---

## 7. Deferred Manual Dogfood vs Automated Test Equivalence

`plan.md` contains no rows marked `[~]` (verified: `grep -c '\[~\]' plan.md` = 0).
Per the template's interpretation rule, this section may be left blank (blank = PASS) for plan.md-tracked deferrals.

For completeness, the one deferral in this change is tracked in `tasks.md` (4.3c) and `measurements.md`, not in `plan.md`, so it does not trigger the §7 fill-in requirement; it is nevertheless surfaced here for the human gate:

| Deferred dogfood | Equivalent automated / static coverage | Coverage assessment | Real gap? |
|---|---|---|---|
| tasks.md 4.3c: live workflow dry-run (push test branch with trivial Cargo.lock change → observe auto-regen + auto-commit of Cargo.nix) | 4.3a local cargo-nix-lock-sync RED/GREEN demo + 4.3b actionlint clean static workflow validation; the auto-commit step reuses the byte-identical github-actions[bot] mechanism already proven in production by the flake.lock and bun.nix regen steps. | RED-state regression demonstrated locally; workflow YAML statically validated; auto-commit mechanism is a proven production pattern. The only uncovered slice is the live runtime exercise on a pushed branch. | no (equivalently covered for pre-archive; live confirmation arrives on the first post-push CI run touching Cargo.lock) |

> Verdict for the human gate: this deferral does NOT block archive. Static + local-RED evidence plus reuse of a proven auto-commit mechanism is adequate pre-archive coverage.
> Surface to the human: the very first push touching Cargo.lock will be the live confirmation of 4.3c.

---

## Issues

### CRITICAL

None.

The prior CRITICAL-1 is RESOLVED by reconciliation commit `c8ba789`.
Re-checked all four sub-counts against the amended `specs/rust-build-substrate/spec.md`, `modules/rust.nix`, and the live `nix eval .#checks.aarch64-darwin --apply builtins.attrNames`:

1. Test command: spec.md:79 now mandates per-member `(cargoNix.workspaceMembers.<name>.build.override { runTests = true; testPreRun = "export HOME=/tmp"; }).passthru.test`. The superseded `cargo nextest run --workspace --no-tests=pass` language is gone. modules/rust.nix has no active `nextest` / `--no-tests=pass` / `--run-ignored` (only comments describing the rejected approach).
2. `runTests=true` mechanism: spec.md:79 mandates `runTests = true`; the prior MUST-NOT is removed. modules/rust.nix:358 sets `runTests = true`. Agreement.
3. Per-crate test promotion: spec.md:99-102 + :104-107 affirm the 11 per-member `*-test` checks are intentionally present and the 20 ad-hoc per-crate `*-test`/`*-clippy` packages are removed. Live `nix eval` shows exactly 11 per-member `*-test` checks (`ironstar-core-test` ... `ironstar-test`) plus the `workspace-test` aggregate.
4. Check-surface count: spec.md:99-102 scenario asserts the total is 27; live `nix eval` = 27 (verified attribute list and length).

The baseline is now stated as 933 passed / 5 ignored over 938 defined (spec.md:82/87), matching measurements.md:112.
Because the delta spec is the only artifact synced to the empty `openspec/specs/` on archive, and it no longer carries any contradicted assertion, the verbatim-enshrinement risk is eliminated.

### WARNING

None.

Disposition of the five prior WARNINGs:

- WARNING-1 (build-graph-invariants / cargo-nix-lock-sync uncovered): RESOLVED. spec.md:109-135 adds a first-class "build-graph envelope and drift regulator" requirement with 4 scenarios (lockstep snapshot, duplication growth fails, planned shrinkage passes, 11-member floor); cargo-nix-lock-sync remains covered by spec.md:3-22.
- WARNING-2 (proposal.md 14/911 framing + wrong README figure): RESOLVED. proposal.md:39 now reads "the check surface lands at 27 ... at 933-passed/5-ignored parity"; :45 reads "12 → 27 checks"; :51 restates "933-passed/5-ignored parity". No `14`/`911` survives in the proposal body.
- WARNING-3 (design.md 14/26/27 + 911 inconsistency): RESOLVED. All end-state figures in design.md are 27 and 933/5/938. The surviving `14`/`26` tokens are explicit progression-arithmetic; no bare end-state `stay 14` assertion remains; no `911` token appears in design.md.
- WARNING-4 (D9 `-c2n` canonical roots): RESOLVED. design.md:231 names post-swap `packages.ironstar` / `packages.ironstar-release`, matching modules/apps/build-graph-snapshot.nix and the committed baseline/snapshot JSON required_roots.

### SUGGESTION

**SUGGESTION-1 (carried) — Task 4.3 parent box remains unchecked solely due to the documented-deferred 4.3c sub-box (live workflow dry-run).**

tasks.md:62 (4.3 parent) and tasks.md:65 (4.3c) are the only unchecked boxes.
4.3a and 4.3b are complete and evidenced; 4.3c is a documented deferral (rationale in measurements.md), not undone work.
Recommendation: accept the deferral at the human gate; the static + local-RED evidence plus byte-identical reuse of the proven flake.lock/bun.nix auto-commit mechanism is adequate pre-archive coverage. Optionally leave 4.3/4.3c unchecked as a tracking marker until the live CI run is observed.
Does not block archive.

**SUGGESTION-2 (carried) — Single-platform build-graph baseline; frozen `-c2n-member-src` derivation-name token.**

modules/checks/build-graph-baseline.json pins the baseline to `x86_64-linux`, which is what buildbot CI evaluates; a darwin-specific graph regression would not be caught by the regulator.
This is now a deliberate, documented choice: design.md:229-230 states the baseline is "pinned to a single canonical system, x86_64-linux, which is what buildbot CI evaluates and which evals purely from the committed Cargo.nix with no IFD", and the baseline JSON `note` field records the same rationale.
The two member-source derivation names retain the frozen `-c2n-member-src` token; design.md:233 carries the deliberate-freeze rationale ("content-addressed names whose stability the build relies on, and renaming them would force member rebuilds").
Recommendation: accept as documented intentional freezes; optionally add a per-system baseline map in a future change if a darwin-specific graph regression is ever observed.
Does not block archive.

Disposition of the prior SUGGESTIONs:

- prior SUGGESTION-2 (residual nix-unit comments in justfile / mk-structural-check.nix / package-set-invariant.nix): RESOLVED. `grep -nE 'nix-unit'` returns no match in any of the three files, and a repo-wide sweep of active `*.nix`/`justfile` finds no residue.
- prior SUGGESTION-3 (frozen `-c2n` token, single-platform baseline, clippy scenario detail): the clippy-scenario detail is now folded into spec.md:94-97; the remaining two items are carried as SUGGESTION-2 above with deliberate-freeze rationale now present.

---

## Open questions for the human gate

- Push is user-gated in this WO session, so the §5 push checkbox is unchecked. The orchestrator should confirm the migration lobe and reconciliation commit are pushed (PR 477) before archive, and that the first post-push CI run touching Cargo.lock provides the live 4.3c confirmation.
- CLAUDE.md project status still cites the stale 911-test figure. CLAUDE.md is outside this change's diff and is a symlink to sciexp/planning per project memory. Recommend raising the 911 staleness as a discovered follow-up against the planning repo rather than blocking this archive.
- The historical `911` tokens that remain in tasks.md:40 (acceptance text), plan.md:48 (micro-task plan), and brainstorm.md (rejected-option rationale) are in artifacts that are NOT synced to the canonical spec on archive. They are left as-is as accurate historical record (tasks.md:40 itself notes the figure is stale). Confirm this convention is acceptable, or fold a cosmetic pass into a later commit if uniform figures are preferred.

---

## Overall Decision

- [x] (pass) PASS — may proceed to finishing-a-development-branch and archive
- [ ] (warn) PASS WITH WARNINGS — may proceed to subsequent steps but note: `<explanation>`
- [ ] (fail) FAIL — return to the failed artifact, correct it, then re-run verify

**PASS** with 0 CRITICAL, 0 WARNING, 2 SUGGESTION (both carried with rationale; neither blocks archive).

This is a re-adjudication, not a rubber-stamp: each prior finding was independently re-checked against the amended artifacts, the live 27-check surface, and modules/rust.nix, and a new-drift sweep was run.
The prior CRITICAL-1 and all five prior WARNINGs are resolved by reconciliation commit `c8ba789`; the only archive-synced artifact (the delta spec) now matches the as-built implementation on every previously-contradicted count, and `openspec validate --strict` is clean.
The two carried SUGGESTIONs are documented intentional freezes (single-platform baseline, `-c2n-member-src` token) and the documented 4.3c deferral, none of which block archive at the human gate.

**Next step**:

Proceed to finishing-a-development-branch and archive.
On archive, the delta `specs/rust-build-substrate/spec.md` will be created as the canonical `openspec/specs/rust-build-substrate/spec.md`; this now correctly enshrines the as-built contract.
Surface to the human: confirm push/PR 477 status, treat the first post-push CI run touching Cargo.lock as the live 4.3c confirmation, and consider raising the CLAUDE.md 911 staleness as a follow-up against the planning repo.
