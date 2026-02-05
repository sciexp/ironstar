# Decision: Web Components Build Tooling

## Status

Accepted

## Context

The ironstar project has a web-components directory that builds frontend assets (CSS, JavaScript via Rolldown) for embedding into the Rust binary via rust-embed.
Issue ny3.22 asks whether to migrate from the current pnpm setup to bun for web-components, potentially unifying it as a bun workspace package under packages/web-components.

The key questions are:
1. Does nixpkgs.bun support workspace package builds for static asset generation?
2. Can bun-built assets integrate with nix build via crane without breaking reproducibility?
3. Are there lockfile format issues between bun.lock and crane's Cargo.lock handling?
4. What are the tradeoffs vs keeping pnpm for web-components?

## Historical Background

The web-components directory was introduced in commit 3ec692e (2026-01-19) with the message "feat(frontend): add web-components package.json with rolldown deps".
No commit message or documentation explicitly records why pnpm was chosen over other package managers.
However, the architectural context reveals the reasoning.

The project has a dual package manager setup:
- Root project and packages/docs use bun (bun.lock at project root, approximately 2500 lines with workspace definitions for root and packages/docs)
- web-components uses pnpm (pnpm-lock.yaml in web-components directory)

This split exists because the nix build for frontend assets (modules/rust.nix lines 66-96) requires reproducible dependency fetching.
The nix build uses `pkgs.pnpmConfigHook`, `pkgs.pnpm`, and `pkgs.fetchPnpmDeps` to create a hermetic build of frontend assets before embedding them into the Rust binary.

The development shell (modules/dev-shell.nix) includes bun for general development tasks, while the nix build specifically uses pnpm for its reproducible fetcher support.

## Current State Assessment

### Nixpkgs bun status

Bun is available in nixpkgs (version 1.3.8 at evaluation time).
However, PR 376299 (opened 2025-01-24) titled "WIP: bun: init configHook and fetchDeps; bun: switch to finalAttrs from rec" remains open and unmerged.

The PR aims to add `bun.fetchDeps` and `bun.configHook` analogous to pnpm's `fetchPnpmDeps` and `pnpmConfigHook`.
Recent activity (February 2026) shows ongoing issues:
- Catalog dependencies fail to resolve in the configure phase
- Package conversion work is ongoing but incomplete
- The PR is labeled as WIP with changes requested

Without `bun.fetchDeps` and `bun.configHook`, there is no supported way to create reproducible nix derivations that fetch bun dependencies in a hermetic sandbox.

### Current lockfile situation

The project has no lockfile conflicts.
The dual-lockfile approach (bun.lock for root/docs, pnpm-lock.yaml for web-components) is intentional and functional:
- bun.lock tracks devDependencies for semantic-release and docs site dependencies
- pnpm-lock.yaml tracks web-components dependencies (echarts, lit, open-props, rolldown, etc.)
- No cross-workspace dependencies exist between these two dependency trees

### Migration cost assessment

Migrating web-components to bun would require:
1. Waiting for nixpkgs PR 376299 to merge
2. Converting modules/rust.nix from pnpmConfigHook/fetchPnpmDeps to bun.configHook/bun.fetchDeps
3. Regenerating lockfile (bun install in web-components)
4. Updating the hash in rust.nix for the new fetcher

Migration benefit would be tooling unification (single package manager), but the nix reproducibility story is the gating factor.

## Options Considered

### Option 1: Maintain pnpm (current)

Continue using pnpm for web-components with the established nixpkgs integration.

Advantages:
- Works today with stable nixpkgs support
- Reproducible nix builds are functional and tested
- No migration risk or effort required
- Hash (sha256-Mb6rwHxXIMNissTEd/VGEM4cCQ0Gy36fBYKODcRMFp8=) is already computed and verified

Disadvantages:
- Two package managers in the project
- Developers must remember pnpm for web-components vs bun for root

### Option 2: Migrate to bun

Move web-components under packages/ as a bun workspace member and use bun throughout.

Advantages:
- Single package manager for all JavaScript/TypeScript
- Bun's faster installation and build performance
- Unified workspace structure

Disadvantages:
- Blocked on nixpkgs PR 376299 (bun.fetchDeps/configHook)
- PR has been open for over a year with ongoing issues
- Migration introduces risk to working reproducible builds
- No clear timeline for PR completion

## Decision

Maintain pnpm for web-components.

## Rationale

The decision is driven by nixpkgs support status.
The `pnpmConfigHook` and `fetchPnpmDeps` integration is stable and working in production.
The equivalent bun support (PR 376299) has been in development for over a year and continues to face issues with catalog dependencies and package conversion.

Reproducible nix builds are a core requirement for ironstar's build pipeline.
The frontendAssets derivation must fetch dependencies hermetically and produce bit-identical outputs.
This requirement cannot be compromised for tooling unification.

The two-package-manager setup is not a significant burden:
- Developers rarely interact with web-components dependencies directly
- The nix build handles web-components automatically via crane
- The separation matches the architectural boundary (Rust binary vs frontend assets)

When PR 376299 merges and stabilizes, migration becomes a low-risk follow-up task.
At that point, the benefits of a unified bun workspace may justify the migration effort.

## Consequences

### Immediate

- No changes to existing build infrastructure
- web-components continues using pnpm-lock.yaml
- modules/rust.nix continues using pnpmConfigHook/fetchPnpmDeps

### Follow-up actions

- Monitor nixpkgs PR 376299 for merge and stabilization
- Create a follow-up issue to re-evaluate migration after bun nix support lands
- Document the decision rationale for future contributors who may ask about the dual-lockfile setup

### Monitoring criteria for reconsideration

Reconsider this decision when:
1. PR 376299 merges to nixpkgs-unstable
2. At least one release cycle passes with stable bun.fetchDeps usage
3. Examples of bun workspace builds in nix derivations emerge in the ecosystem
