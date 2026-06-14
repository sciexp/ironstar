# Verification Report

> This file is produced by the `openspec-verify-change` skill after apply completes, to confirm that the
> implementation is consistent with the specs / design / tasks. Any failed check must be returned to its
> corresponding artifact for correction before re-running verify.

**Change**: `migrate-crane-to-crate2nix`
**Verified at**: `2026-06-14`
**Verifier**: `verification-report subagent (re-round after the task-7 build-graph demotion; reconciling the check-surface delta 27 → 26)`

---

## Summary scorecard

| Dimension | Verdict | Critical | Warning | Suggestion |
|---|---|---|---|---|
| Completeness | PASS | 0 | 0 | 1 (carried) |
| Correctness | PASS | 0 | 0 | 0 |
| Coherence | PASS | 0 | 1 | 1 (carried) |
| Structural validation | PASS | — | — | — |
| Task completion | PASS (1 documented deferral) | — | — | — |

This is a re-round verification after the task-7 demotion, not a rubber-stamp of the prior PASS.
The prior verify.md / measurements.md encoded a 27-check surface and a `build-graph-invariants` graph-drift regulator.
Task 7 (lines 102-112 of tasks.md) demoted that gating to an on-demand, non-gating committed record (`modules/apps/build-graph-snapshot/snapshot.json`) and deleted the regulator and its baseline, returning the check surface to 26.
The current check surface was enumerated by read-only evaluation only (`nix flake show --json --all-systems` and `nix eval .#checks.<system> --apply builtins.attrNames`; flake referenced as bare `.`; no `nix flake check`).
The crate2nix migration itself (committed `Cargo.nix`, per-crate `buildRustCrate`, pinned toolchain, crane removal) was verified in the prior round and is confirmed here by artifact and code inspection only — it was not re-realized.

---

## 1. Structural Validation (`openspec validate --all --json`)

- [x] All items report `"valid": true`

The delta spec `specs/rust-build-substrate/spec.md` retains at least one `#### Scenario` block per ADDED requirement, including the rewritten non-gating-record requirement; the demotion did not break the schema shape.

| Item | Type | Issues |
|---|---|---|
| — | — | — |

---

## 2. Task Completion (`tasks.md`)

- [ ] All `- [ ]` have been changed to `- [x]`

Leaf-task accounting: 41 leaf checkboxes complete, 1 leaf checkbox (4.3c) documented-deferred, 1 parent checkbox (4.3, tasks.md:62) left unchecked solely because of its deferred sub-box.
Task 7 (the demotion, lines 102-112) is fully checked (7.1–7.4).

**Incomplete tasks** (if any):

| Task | Reason incomplete | Blocks archive? |
|---|---|---|
| 4.3 (parent, tasks.md:62) | Parent of 4.3c; unchecked only because the one deferred sub-box remains open. 4.3a (local `cargo-nix-lock-sync` RED/GREEN demo) and 4.3b (actionlint + static workflow validation) are complete and evidenced in measurements.md. | No — tracking marker for the deferred sub-box. |
| 4.3c (tasks.md:65) DEFERRED | Live workflow dry-run (push a test branch with a trivial `Cargo.lock` change, observe auto-regen + auto-commit of `Cargo.nix` by github-actions[bot]) requires a pushed branch, user-gated in this WO session. The workflow IS implemented: `regenerate-lock-files.yaml` triggers on `Cargo.lock`/`Cargo.toml`/`crates/**/Cargo.toml`, and the Cargo.nix regen+amend step pair (lines 75-87) reuses the byte-identical github-actions[bot] auto-commit mechanism already proven in production by the flake.lock and bun.nix steps. | No — documented deferral. Static + local-RED coverage plus reuse of the proven auto-commit mechanism is adequate pre-archive coverage; the first post-push CI run touching `Cargo.lock` is the live confirmation. |

Task completion is acceptable for archive at the human gate: the single open box is a justified, documented deferral, not undone work.

---

## 3. Delta Spec Sync State

Delta spec file: `openspec/changes/migrate-crane-to-crate2nix/specs/rust-build-substrate/spec.md`.
Main specs directory `openspec/specs/` has no `rust-build-substrate` capability yet; on archive the delta is absorbed into a newly-created `openspec/specs/rust-build-substrate/spec.md`.

| Capability | Sync status | Notes |
|---|---|---|
| rust-build-substrate | pending sync (new capability) | The delta now encodes the post-demotion contract: per-member `runTests=true` test gate, 26-check surface (spec.md:99-102), `build-graph-invariants` absent (spec.md:120-123), and the "build-graph snapshot is a committed, deterministic, non-gating record" requirement (spec.md:109-118). The 933-passed/5-ignored/938-defined envelope is preserved. Sync at archive is the normal lifecycle step. |

The delta spec is the only artifact synced to the canonical specs on archive; proposal.md, design.md, plan.md, tasks.md, measurements.md, and brainstorm.md are not synced, so residual historical figures in those files do not reach the canonical spec.

---

## 4. Design / Specs Coherence Spot Check

| Sampled item | design description | specs correspondence | Gap |
|---|---|---|---|
| Test gate shape | design.md D6: 11 per-member `runTests` checks + zero-cost `workspace-test` linkFarm aggregate. modules/rust.nix:357-358 (`runTests = true`), :366/:402 (linkFarm aggregate). | spec.md:79 mandates per-member `(cargoNix.workspaceMembers.<name>.build.override { runTests = true; testPreRun = "export HOME=/tmp"; }).passthru.test`; spec.md:80 mandates the `workspace-test` linkFarm aggregate. | None. |
| Check surface count | design.md:340 + D9 amendment (258-262): net surface 26; the task-4b regulator that briefly raised it to 27 was retracted by task 7. Live `nix eval` = 26 on all systems. | spec.md:99-102 asserts total check count 26 (15 prior + 11 per-member); spec.md:120-123 asserts `build-graph-invariants` absent. | None — spec asserts 26; live surface is 26. |
| Build-graph record | design.md D9 amendment (258-262): regulator demoted; snapshot survives as a non-gating committed record at `modules/apps/build-graph-snapshot/snapshot.json`, regenerated on demand via the `build-graph-snapshot` flake app. | spec.md:109-118 "build-graph snapshot is a committed, deterministic, non-gating record" requirement + scenarios (deterministic regeneration; no build-graph check gates the surface). | None — spec, design, and the deleted-check reality agree. |
| Test baseline | design.md:320: 933 passed / 5 ignored (938 defined). | spec.md:82/87 assert 933 passed / 5 ignored over 938 defined. | None. |
| Clippy gate | design.md: `cargo clippy --profile dev --locked --all-targets -- --deny warnings` over `importCargoLock`-vendored deps. | spec.md:81 + scenario :94-97 assert the same flags, offline. | None. |
| Cargo.nix lockstep | design.md D7: `cargo-nix-lock-sync` content-addresses Cargo.lock + Cargo.nix. modules/checks/cargo-nix-lock-sync.nix. | spec.md:3-22 Cargo.nix-lockstep requirement + 3 scenarios. | None. |
| No-IFD | design.md D1: committed Cargo.nix, no `allow-import-from-derivation`, no `crate-hashes.json`. | spec.md:24-38 no-IFD requirement + scenarios. | None — `rg` confirms no IFD residue in active code. |

**Drift warnings** (non-blocking):

- WARNING-1 (RESOLVED in this reconciliation commit): at verify time, proposal.md line 41 read "the check surface lands at 27 (15 prior + 11 per-member `*-test` + `build-graph-invariants`)" and line 47 read "12 → 27 checks", contradicting the as-built 26-check / no-regulator end state and the proposal's own Capabilities block (line 53). The same commit that regenerated measurements.md corrected proposal.md lines 41 and 47 to 26 and dropped the `build-graph-invariants` term; the proposal is now internally consistent. No residual action.

---

## 5. Implementation Signal

- [x] No unstaged files in the worktree
- [ ] All related commits have been pushed

`jj st`: the working copy has no changes (`@` is `myuvonzn`, parent `@-` is `posqtkut 66c1ff6d` "wip: diamond development join — ejp observability + crate2nix migration").
The crate2nix-migration work and the task-7 demotion are committed in the join lobe.
Push state is user-gated in this WO session; the push checkbox is left unchecked because push is not confirmed from within this verification.

---

## 6. Front-Door Routing Leak Detector (warning, non-blocking)

`docs/superpowers/specs/*.md`: no front-door routing leak detected.

| File | Content captured into change? | Recommended action |
|---|---|---|
| — | — | — |

---

## 7. Deferred Manual Dogfood vs Automated Test Equivalence

| Deferred dogfood | Equivalent automated / static coverage | Coverage assessment | Real gap? |
|---|---|---|---|
| tasks.md 4.3c: live workflow dry-run (push test branch with trivial Cargo.lock change → observe auto-regen + auto-commit of Cargo.nix) | 4.3a local `cargo-nix-lock-sync` RED/GREEN demo + 4.3b actionlint clean static workflow validation; the auto-commit step reuses the byte-identical github-actions[bot] mechanism already proven in production by the flake.lock and bun.nix regen steps. | RED-state regression demonstrated locally; workflow YAML statically validated; auto-commit mechanism is a proven production pattern. The only uncovered slice is the live runtime exercise on a pushed branch. | no (equivalently covered for pre-archive; live confirmation arrives on the first post-push CI run touching Cargo.lock) |

> Verdict for the human gate: this deferral does NOT block archive.

---

## Issues

### CRITICAL

None.

The prior round's CRITICAL was resolved before this re-round. This re-round adds no new CRITICAL: the check surface is the asserted 26, the regulator is deleted, and the spec matches the as-built non-gating-record contract.

### WARNING

**WARNING-1 (RESOLVED) — proposal.md asserted the retracted 27-check / build-graph-invariants surface (lines 41, 47) at verify time; reconciled in this commit.**

At verify time the proposal body contradicted the demotion and its own Capabilities block. The reconciliation commit corrected lines 41 and 47 to the 26-check non-gating-record end state and dropped the `build-graph-invariants` term. proposal.md is not synced to the canonical spec on archive; the record is now coherent. Did not block archive.

### SUGGESTION

**SUGGESTION-1 (carried) — Task 4.3 parent box remains unchecked solely due to the documented-deferred 4.3c sub-box (live workflow dry-run).**

tasks.md:62 (4.3 parent) and tasks.md:65 (4.3c) are the only unchecked boxes. 4.3a and 4.3b are complete and evidenced; 4.3c is a documented deferral. Recommendation: accept the deferral at the human gate; optionally leave 4.3/4.3c unchecked as a tracking marker until the live CI run is observed. Does not block archive.

---

## Open questions for the human gate

- measurements.md has been regenerated in this reconciliation commit: the Task 4b regulator sections were marked superseded and a Task 7 demotion record plus the 26-name surface were added, replacing the verbatim 27-name enumerations.
- proposal.md lines 41 and 47 were corrected to 26 (the `build-graph-invariants` term dropped) in this reconciliation commit; WARNING-1 is resolved.
- Push is user-gated; the orchestrator should confirm the migration lobe and the task-7 demotion are pushed before archive, and treat the first post-push CI run touching `Cargo.lock` as the live 4.3c confirmation.
- Cosmetic: `__pycache__` directories under `modules/apps/build-graph-snapshot/` and `modules/apps/build-graph-report/` are gitignored and untracked (only `normalize.py` and `snapshot.json` are tracked); no action required.

---

## Overall Decision

- [x] (pass) PASS — may proceed to finishing-a-development-branch and archive
- [ ] (warn) PASS WITH WARNINGS
- [ ] (fail) FAIL

**PASS** with 0 CRITICAL, 1 WARNING (proposal.md 27→26 reconciliation, non-blocking, non-synced artifact), 1 carried SUGGESTION (documented 4.3c deferral).

The check surface is the asserted 26 on the CI-relevant x86_64-linux system (and all exposed systems); no build-graph drift/gating check remains in `.#checks`; the snapshot survives as a non-gating on-demand committed record; the spec, design, tasks, and workflow README are reconciled to the demotion; and the crate2nix migration itself is confirmed by artifact and code inspection to still build the workspace under the c2n substrate with crane fully removed.

**Next step**:

Regenerate verify.md and measurements.md to this post-demotion reality under orchestrator control, correct proposal.md lines 41/47 to 26, then proceed to finishing-a-development-branch and archive. On archive, the delta `specs/rust-build-substrate/spec.md` (which already encodes the 26-check non-gating-record contract) becomes the canonical `openspec/specs/rust-build-substrate/spec.md`.
