---
title: Nix CI consolidation
cynefin: complicated
status: planning
---

# Nix CI consolidation

This document captures the design for consolidating ironstar's CI/CD pipeline into nix flake checks, reducing imperative GitHub Actions YAML to a minimal shell around `nix flake check`.
The work serves a dual purpose: improving ironstar's CI and experimentally validating the nix-first CI pattern for the sciexp monorepo architecture.

## Motivation

Ironstar's current CI (`ci.yaml`, ~800 lines) orchestrates validation through a mix of nix commands and imperative shell steps.
Several validation concerns (secrets scanning, JS/TS package tests, binary builds) run outside of nix's build graph, forfeiting the content-addressed caching that nix provides.
The result is redundant computation, brittle YAML wiring, and a pipeline that is difficult to maintain.

The target state is that `nix flake check` becomes the single validation command, exercising all checks through nix derivations.
This creates full optionality: the same `.#checks` attribute set works unchanged under GitHub Actions (single-runner `nix flake check`), a future buildbot-nix deployment (per-check fan-out with shared nix store), or local development (`nix flake check -L` or piped through nom).

This initiative also informs the sciexp monorepo architecture evaluation.
If ironstar demonstrates that a Rust + JS/TS project can express its entire validation pipeline as nix flake checks with acceptable performance, the pattern generalizes to the sciexp monorepo where buildbot-nix would evaluate `.#checks` across all constituent projects.

## Current state

### What `nix flake check` covers today

Six check derivations, all defined in `modules/rust.nix`, `modules/formatting.nix`, `modules/checks/nix-unit.nix`, and external flake-parts modules:

| Check | Source module | What it validates |
|---|---|---|
| `workspace-fmt` | `modules/rust.nix` | `cargo fmt --check` |
| `workspace-test` | `modules/rust.nix` | `cargo nextest run --workspace` (933 tests) |
| `workspace-clippy` | `modules/rust.nix` | `cargo clippy --all-targets -- -D warnings` |
| `treefmt` | `modules/formatting.nix` | nixfmt + rustfmt + biome |
| `pre-commit` | `modules/formatting.nix` | treefmt + gitleaks hooks |
| `nix-unit` | `modules/checks/nix-unit.nix` | Flake structure metadata assertions |

`workspace-test` and `workspace-clippy` share `cargoArtifacts` (the crane dependency layer).
`workspace-fmt` depends only on the raw filtered Rust source (`src`), not on `cargoArtifacts` or `combinedSrc`.
The remaining checks (`treefmt`, `pre-commit`, `nix-unit`) are independent of Rust compilation.
Running `nix flake check` on a single runner preserves shared `cargoArtifacts` across `workspace-test` and `workspace-clippy`.

### What CI does today

The `nix` job already runs `nix flake check --accept-flake-config --system x86_64-linux -L` via `just ci-build-category`, covering the six checks above.
The remaining jobs run outside the nix build graph:

| CI job | What it does | Path to consolidation |
|---|---|---|
| `secrets-scan` | `nix run nixpkgs#gitleaks -- detect` | Wrap as check derivation |
| `flake-validation` | `nix flake show --json`, `just --summary` | Evaluation-only; subsumable |
| `bootstrap-verification` | Verifies nix setup | CI plumbing; not needed when nix is the only tool |
| `binary-build` | `nix build .#ironstar-release` | Eliminate; dev binary sufficient for E2E |
| `test` (JS/TS) | `just <pkg>-test-coverage`, `just <pkg>-build`, `just <pkg>-test-e2e` | Build packages as nix derivations with checkPhase |
| `e2e` | Playwright against running ironstar server | Keep impure; server + browser not sandboxable yet |
| Deployment jobs | Cloudflare Workers, releases | Inherently effectful; remain in YAML |

### Nix derivation dependency graph for Rust builds

All crane derivations share `commonArgs` which uses `combinedSrc`:

```
src (builtins.path, Rust + SQL filtered)
  |
  +---> workspace-fmt (cargoFmt, no compilation)
  |
  +---> frontendAssets (pnpm/Rolldown from web-components/)
  |        |
  +--------+---> combinedSrc = src + frontendAssets + migrationsSrc
                    |
                    +---> cargoArtifacts (buildDepsOnly, dev profile)
                    |        |
                    |        +---> workspace-test  (cargoNextest, all crates)
                    |        +---> workspace-clippy (cargoClippy --all-targets)
                    |        +---> ironstar        (buildPackage, dev binary)
                    |
                    +---> cargoArtifactsRelease (buildDepsOnly, release profile)
                              |
                              +---> ironstar-release (buildPackage, production binary)
```

`workspace-test` and `workspace-clippy` transitively compile all workspace crates in the dev profile.
The dev-profile `ironstar` package compiles the same code to produce an installable binary.
The release-profile `ironstar-release` is a completely separate compilation path (different `CARGO_PROFILE`, LTO, stripping, `opt-level = "z"`).

## Design decisions

### No `ironstar-release` in checks

The release build with LTO is expensive and provides negligible additional signal over dev-profile checks that already compile every crate.
The dev binary works for E2E testing (confirmed by experiment on the `ci-dev-profile-e2e` bookmark, already in the current jj chain).
Leave `ironstar-release` as a package derivation built only for actual releases, not on every PR.

### No nix-github-actions

`nix-github-actions` fans out one GitHub Actions runner per check attribute.
Under GHA, runners don't share a nix store, so `cargoArtifacts` would be substituted or rebuilt independently on each runner.
For ironstar's Rust workspace, this is a performance regression compared to running `nix flake check` on a single runner where all checks share `cargoArtifacts` through the nix store.

buildbot-nix achieves per-check fan-out without this problem because all builds share a single nix store on the worker.
The compatibility contract between `nix flake check` and buildbot-nix is simply: all validation is expressed as `.#checks` attributes.
No bridging library is needed.

### bun2nix for JS/TS package derivations

Both `packages/docs` (Astro Starlight) and `packages/eventcatalog` (EventCatalog) use bun (`"packageManager": "bun@1.3.4"`) with a root `bun.lock`.
`buildNpmPackage` (npm-only) and `pnpmConfigHook` (for the separate `web-components/` pnpm workspace) are not appropriate.

bun2nix (nix-community) provides:
- `fetchBunDeps` for offline dependency vendoring from `bun.lock`, with workspace support
- `bun2nix.hook` for `stdenv.mkDerivation` with custom `buildPhase`/`checkPhase`/`installPhase`
- Workspace lockfile parsing with `copyPathToStore` for workspace members

Maturity assessment: 486 commits, active development under nix-community, workspace support stabilized through multiple fixes (April-November 2025), working workspace template.
Supports bun v1.2+ (ironstar uses 1.3.4).

Setup: add `bun2nix` as a flake input, run `bun2nix` CLI to generate `bun.nix` from root `bun.lock`, create derivations using `stdenv.mkDerivation` + `bun2nix.hook`.

### pkgs-by-name-for-flake-parts for package derivation locations

Package derivations go in `pkgs/by-name/<name>/package.nix`, following the nixpkgs convention via `pkgs-by-name-for-flake-parts`.
This provides auto-discovery (drop a `package.nix` and it appears as `config.packages.<name>`), nixpkgs-compatible `callPackage` interface (with optional `inputs` access), and scales to many packages without manual registration.

The separation of concerns:
- `pkgs/by-name/<name>/package.nix`: pure derivation definition, no awareness of checks or devShells
- `modules/<concern>.nix`: flake-parts composition consuming packages via `config.packages.<name>`, wiring into checks, devShells, apps

The flow is strictly one-directional: modules reference packages, packages never reference modules.

**What goes where:**
- Crane-based Rust packages (`ironstar`, `ironstar-release`, `frontendAssets`) stay in `modules/rust.nix` because they depend on `crane-lib`, `cargoArtifacts`, and module-local bindings that are not `callPackage`-compatible
- bun2nix-based JS/TS packages (`ironstar-docs`, `ironstar-eventcatalog`) go in `pkgs/by-name/` because they use standard `callPackage` arguments plus `inputs` for bun2nix
- Check wiring for JS/TS packages goes in `modules/checks/` as separate module files discovered by import-tree

Setup: add `pkgs-by-name-for-flake-parts` as a flake input, import its `flakeModule`, set `pkgsDirectory = ../../pkgs/by-name` in perSystem.
Handle `legacyPackages` override if needed (vanixiets uses `lib.mkForce pkgs` to prevent pkgs-by-name from overriding the full nixpkgs set).

### Playwright E2E in check derivations via playwright-web-flake

The doc package E2E tests (Playwright smoke tests against locally-served content) can run inside nix derivations.
Precedent: clan-core's `clan-app` runs storybook snapshot tests via Playwright in `passthru.tests` derivations.

Requirements for the derivation:
- `playwright-web-flake` browsers with `chromium-headless-shell` only (lighter than full Chromium)
- `PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true"`
- `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE = "nixos"`
- `--no-sandbox` and `--disable-dev-shm-usage` for Chromium in nix sandbox
- Pre-built static site served locally (not Astro dev mode which may attempt network access)
- The eventcatalog already has `preview:ci` (`bunx serve dist`) for this pattern; docs needs equivalent

The ironstar application E2E (SSE lifecycle, Datastar behavior) remains impure in CI because it requires a running ironstar server with database initialization, which is not sandboxable without a NixOS VM test.
NixOS VM tests for the application E2E were assessed and rejected for now: 1.5-2.5GB closure, 2-4 minute runtime, SSE timing sensitivity in QEMU.
A lightweight curl-based service smoke test derivation (health endpoint verification) could be added as a simpler alternative.

### Dendritic module composition via import-tree

No central checks aggregator.
Each concern owns its check in its own module file.
Import-tree discovers all `.nix` files under `modules/` (including subdirectories) and flake-parts merges `perSystem.checks` from every module automatically.
This is consistent with the existing `modules/checks/nix-unit.nix` pattern.

### CI YAML design

The `nix-based-ci.yaml` must use the existing `.github/actions/setup-nix` composite action, which handles:
- nothing-but-nix disk clearing (`cleave` protocol, 4GB mnt-safe-haven)
- `cachix/install-nix-action` with pinned nix version (2.32.4)
- magic-nix-cache (FlakeHub disabled)
- cachix push+pull

All nix commands require `--accept-flake-config` because `flake.nix` declares `nixConfig` with extra substituters (nix-community, crane, pyproject-nix, sciexp caches).

The minimal pipeline:

```
nix-check (single runner, full installer):
  setup-nix (nothing-but-nix + cachix)
  nix flake check --accept-flake-config -L

e2e (depends on nix-check):
  setup-nix (quick installer + cachix)
  nix build --accept-flake-config .#ironstar   # cargoArtifacts from cachix
  playwright against dev binary

deployment jobs (unchanged, effectful)
```

For local development, nom is available via home-manager (not in devshell or CI).
The justfile already has a conditional guard:
```bash
if command -v nom &> /dev/null; then
    nix flake check --impure --log-format internal-json -v |& nom --json
else
    nix flake check --impure
fi
```

## Architecture

### New files and directories

```
pkgs/
  by-name/
    ironstar-docs/
      package.nix              # bun2nix-based Astro build
    ironstar-eventcatalog/
      package.nix              # bun2nix-based EventCatalog build
bun.nix                        # generated by bun2nix CLI from bun.lock
modules/
  packages/
    by-name.nix                # imports pkgs-by-name-for-flake-parts, sets pkgsDirectory
  checks/
    nix-unit.nix               # (existing)
    gitleaks.nix               # runCommand wrapping gitleaks against src
    docs.nix                   # wires ironstar-docs passthru.tests into checks
    eventcatalog.nix           # wires ironstar-eventcatalog passthru.tests into checks
.github/
  workflows/
    nix-based-ci.yaml          # minimal nix-first pipeline
```

### Modified files

```
flake.nix                      # add bun2nix and pkgs-by-name-for-flake-parts inputs
.gitignore                     # ensure bun.nix is tracked (not ignored)
```

### Unchanged files

```
modules/rust.nix               # crane-based Rust packages and checks stay as-is
modules/formatting.nix         # treefmt + pre-commit stay as-is
modules/dev-shell.nix          # devShell stays as-is
modules/process-compose.nix    # dev orchestration stays as-is
.github/workflows/ci.yaml      # retained during parallel validation, archived later
.github/actions/setup-nix/     # composite action used by both ci.yaml and nix-based-ci.yaml
```

## Open questions

### patch-package compatibility with bun2nix

ironstar uses `patch-package` (npm ecosystem) for `@eventcatalog/core`, triggered via a `postinstall` script.
bun2nix has built-in support for bun's native `patchedDependencies` format via `patchedDependenciesToOverrides`, but ironstar uses the npm-style `patches/` directory.
Options: convert the patch to bun2nix's `overrides` API, convert to bun's native `patchedDependencies`, or verify that `patch-package` runs correctly in the bun2nix sandbox via lifecycle scripts.

### eventcatalog generate source requirements

The `eventcatalog build` recipe calls `eventcatalog generate` before building, which may read event definitions from the Rust crates.
The derivation's `src` filtering must include any files that `eventcatalog generate` reads.
This needs investigation during implementation.

### Hoisted linker flag

Astro and EventCatalog may expect hoisted `node_modules` layout.
Use `bunInstallFlags = ["--linker=hoisted"]` (plus `"--backend=copyfile"` on Darwin) as documented in bun2nix's hook troubleshooting.
Verify during implementation.

### legacyPackages override

pkgs-by-name-for-flake-parts sets `legacyPackages` to its package scope.
If ironstar uses `legacyPackages` for anything (e.g., clan-core integration, evalCheck sweep), it will need `legacyPackages = lib.mkForce pkgs` as vanixiets does.
Assess during implementation.

## Epic structure

The issues are ordered by dependency.
Issues within the same tier can be worked in parallel.

**Tier 0 — infrastructure (no dependencies):**
- Add `bun2nix` and `pkgs-by-name-for-flake-parts` as flake inputs; create `modules/packages/by-name.nix`
- Add gitleaks check derivation (`modules/checks/gitleaks.nix`)

**Tier 1 — package derivations (depends on tier 0):**
- Create `ironstar-docs` package derivation in `pkgs/by-name/`
- Create `ironstar-eventcatalog` package derivation in `pkgs/by-name/`

**Tier 2 — check wiring (depends on tier 1):**
- Wire `ironstar-docs` tests (unit + E2E) into checks via `modules/checks/docs.nix`
- Wire `ironstar-eventcatalog` tests (unit + E2E) into checks via `modules/checks/eventcatalog.nix`

**Tier 3 — CI pipeline (depends on tiers 0-2):**
- Update E2E job to use `.#ironstar` instead of `.#ironstar-release`
- Create `nix-based-ci.yaml` using setup-nix composite action

**Tier 4 — validation and promotion:**
- Run both `ci.yaml` and `nix-based-ci.yaml` in parallel on PRs; compare results
- Promote `nix-based-ci.yaml` to `ci.yaml`; archive old pipeline

**Cross-reference:**
- sciexp/planning issue documenting evaluation findings for sciexp monorepo architecture

## Relationship to sciexp monorepo architecture

This initiative validates whether the clan-core pattern (all validation as nix flake checks, buildbot-nix for CI) works for a Rust + JS/TS project.
Key evaluation criteria:

- Does crane's `cargoArtifacts` sharing make single-runner `nix flake check` performant enough for CI?
- Does bun2nix reliably handle workspace lockfiles for JS/TS package builds?
- Is `pkgs-by-name-for-flake-parts` a viable convention for monorepo-scale package management?
- What is the developer experience of `nix flake check` as the primary local validation command?

Findings feed into the sciexp monorepo architecture document at `~/projects/sciexp/planning/docs/notes/research/sciexp-monorepo-architecture.md`.
