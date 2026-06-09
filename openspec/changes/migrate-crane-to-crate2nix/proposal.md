---
linear_story_id: 2e4f94e9-7f7b-4a9f-8a83-112978ccfea1
linear_story_identifier: CAM-15
linear_story_title: "Migrate ironstar from crane to crate2nix"
linear_story_url: https://linear.app/cameronraysmith/issue/CAM-15/migrate-ironstar-from-crane-to-crate2nix
linear_story_state: Backlog
linear_team: CAM
linear_project: ironstar-build-performance
last_synced_state: Backlog
last_synced_at: null
review_round: 0
attempt_log: []
beads_epic: ironstar-8g3
---

## Why

Crane builds ironstar's Rust workspace through one workspace-wide `cargoArtifacts` blob shared by every derivation.
A Renovate bump of a single dependency invalidates that whole blob, so the niks3 cache re-stores a large artifact instead of the one dependency crate that changed.
This change adopts crate2nix's per-crate `buildRustCrate` derivations so a single-dependency bump invalidates only that dependency's cone, shrinking cache churn and CI wall-clock and tightening the Renovate-bump blast radius.

## What Changes

**Rust build substrate**
- From: crane with one workspace-wide `cargoArtifacts`/`buildDepsOnly` blob shared across all derivations.
- To: crate2nix with a committed `Cargo.nix` and one `buildRustCrate` derivation per crate.
- Reason: per-dependency-crate cache granularity in the niks3 cache.
- Impact: non-breaking through task 5 (additive `-c2n` packages coexist with crane); the substrate swap and crane removal land at the end.

**Build description and drift control**
- From: crane vendors dependencies at eval; no committed build description.
- To: a committed `Cargo.nix` (no IFD) plus a `just regenerate-cargo-nix` recipe, CI auto-regeneration on `Cargo.lock`/`Cargo.toml` changes, and a no-network `cargo-nix-lock-sync` staleness check.
- Reason: keep `Cargo.nix` in lockstep with `Cargo.lock` while staying fully offline at build.
- Impact: a new generation step in the lock-file workflow; one new flake check, balanced by the deletions below.

**Check and package surface**
- The 14-check surface is invariant; the workspace test and clippy gate is re-implemented crane-free at 911-test parity.
- The 20 ad-hoc per-crate `*-test`/`*-clippy` packages are deleted, shrinking the package surface.

**Removals (final task)**
- The crane flake input and its `cargoArtifacts`/`buildDepsOnly`/`vendorCargoDeps`/`cargoVendorDir` usage are deleted.
- The `crane.cachix.org` substituter is dropped.
- Stale docs are corrected (`.github/workflows/README.md` 12 → 14 checks) and the dead nix-unit input is removed once confirmed non-load-bearing.

## Capabilities

### New Capabilities

- `rust-build-substrate`: the durable requirements governing how the Rust workspace is built under nix — a committed `Cargo.nix` kept in lockstep with `Cargo.lock`, no IFD, the pinned toolchain threaded into per-crate builds, non-empty embedded-asset assertions, the workspace test and clippy gate at 911-test parity, and the per-dependency-crate cache-granularity property.

### Modified Capabilities

<!-- None: ironstar has no existing openspec/specs/ capabilities to modify. -->

## Impact

Build infrastructure: `flake.nix` (crane input and substituter), `modules/rust.nix` (the full crane-to-crate2nix substrate swap), a new committed `./Cargo.nix`, the `just regenerate-cargo-nix` recipe, `.github/workflows/regenerate-lock-files.yaml` (new trigger path and generate-and-amend step), and `.github/workflows/README.md` (check-count correction).
Dependencies: adds the `crate2nix` flake input; removes the `crane` input and the dead `nix-unit` input at the end.
Cache: the niks3 cache at cache.scientistexperience.net tracks roughly 576 per-dependency-crate paths instead of crane's two large blobs.
CI: buildbot nix-eval/nix-build umbrella check names are stable, so Mergify required_checks is invariant.

Scope boundaries — out of scope: the crate decomposition track (v4y), the bun/JS substrate, toolchain version bumps, `--profile ci` adoption for the test gate, and ducklake catalog provenance (the empty-embed status quo is preserved; sourcing a real catalog is a discovered follow-up).
