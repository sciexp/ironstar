<!--
These checkboxes are the authoritative human-in-the-loop ledger for the apply phase.
The first checked box fires the board's Todo-to-In-Progress transition, so every box is authored unchecked.
The six tasks transcribe result.final.task_dag: additive and reversible until task 6, jj mode (anonymous chains, no worktrees).
-->

## 1. Bootstrap crate2nix input and committed Cargo.nix

Goal: add the crate2nix flake input (nixpkgs follows), add the `just regenerate-cargo-nix` recipe (`nix run github:nix-community/crate2nix -- generate`), and generate and commit `./Cargo.nix`.
Do not add `allow-import-from-derivation`.
Depends on: none.

- [x] 1.1 Add `crate2nix.url = "github:nix-community/crate2nix"` with `crate2nix.inputs.nixpkgs.follows = "nixpkgs"` to `flake.nix`.
- [x] 1.2 Add a `just regenerate-cargo-nix` recipe running `nix run --inputs-from . crate2nix -- generate` (then `treefmt Cargo.nix`) at the repo root. Refinement: `--inputs-from .` pins the generator to flake.lock's crate2nix revision instead of the floating `github:` ref, satisfying closure-operator determinism; verified to resolve (crate2nix 0.15.0).
- [x] 1.3 Generate `./Cargo.nix` with default features and commit it; confirm it references all 11 local members, each with `src = ./crates/<name>` (per-member, confirming the keystone in-repo), and that no `crate-hashes.json` is emitted and no IFD is added.
- [x] 1.4 Verify: `nix flake metadata | rg crate2nix`; `nix eval .#packages.aarch64-darwin --apply builtins.attrNames` still evaluates; the `rg -c '^\[\[package\]\]' Cargo.lock` count reconciles with the distinct `crateName/version` pairs in `Cargo.nix`. Reconciliation: 576 Cargo.lock packages == 576 distinct Cargo.nix `crateName@version` pairs (11 local + 565 third-party), bidirectional `comm` diff empty; crate2nix locked at rev `c994c83`.
- [x] 1.5 Capture crane's current eval time and a raw `Cargo.nix` instantiation timing in `measurements.md`; the `ironstar-c2n` eval timing for the ≤2x gate is deferred to task 2.5. Staging: `ironstar-c2n` does not exist until task 2 wires `Cargo.nix` into `modules/rust.nix`, so its eval cannot be timed at task 1.

## 2. Crate overrides, parallel dev/release packages, and source injection for the two parent-reaching members

Goal: add `ironstarCrateOverrides` (libduckdb-sys cc+HOME, aws-lc-sys cmake+perl, ring perl; rely on the nixpkgs default for libsqlite3-sys; no global pkg-config; drop the Linux openssl buildInput provisionally); build `ironstar-c2n`/`ironstar-release-c2n` from `workspaceMembers."ironstar".build` with the pinned 1.94.1 toolchain threaded in; override `src` for `ironstar` and `ironstar-analytics-infra` to a `combinedSrc`-derived subtree re-establishing `../../static/dist` and `../../assets/ducklake-catalogs` above each crate root, adding the ducklake-catalogs injection that is absent today.
This is the highest-risk task — the parent-relative asset injection is the load-bearing unknown.
Depends on: 1.

- [x] 2.1 Add the single `ironstarCrateOverrides` attrset merged onto `pkgs.defaultCrateOverrides` with the verified `-sys` overrides (aws-lc-sys mandatory).
- [x] 2.2 Build `ironstar-c2n` (dev, `release = false`) and `ironstar-release-c2n` (release) from `workspaceMembers."ironstar".build` with the pinned toolchain threaded into `buildRustCrateForPkgs`. Refinement: `release` is a top-level `import ./Cargo.nix { release = …; }` argument baked into each crate config (crate2nix threads it into `buildByPackageIdImpl`), not a `.build.override` argument; so dev and release use two `import` instances (`mkCargoNix false` / `mkCargoNix true`) rather than `.override { release = …; }`.
- [x] 2.3 Override `src` for `ironstar` and `ironstar-analytics-infra` to per-member derived subtrees with fixed `name`s, re-establishing the `../../` parent layout for `static/dist` and `assets/ducklake-catalogs`. Refinement: the `src` override sets the full derived tree (member at `crates/<name>`, asset dirs two levels up) plus `workspace_member = "crates/<name>"`, so `buildRustCrate` cd's into the member subdir (`configure-crate.nix` lines 60-61, 160) and `CARGO_MANIFEST_DIR/../../<asset>` resolves to the re-materialized siblings. `ducklake-catalogs` is injected as an empty tree, preserving the empty-embed status quo (crane's combinedSrc never injected it; `.db` is gitignored).
- [x] 2.4 Add `ironstar-c2n`/`ironstar-release-c2n` to the package-set-invariant `excluded` list for the transition.
- [x] 2.5 Acceptance and verify. Correction (a) PLATFORM STAGING: build and verify on aarch64-darwin locally; x86_64-linux verification is deferred to buildbot CI on push (the task-5 swap gate requires both green), so local-only is expected here — reason: this WO session runs on aarch64-darwin and cross-building the libduckdb-sys C++ amalgamation + aws-lc-sys for x86_64-linux is not available without a linux remote builder. Correction (b) EMBED PARITY, not non-empty-catalog: first obtain the CRANE binary (`nix build .#ironstar`) as baseline and extract its embedded-asset evidence; then assert the c2n binary's `AssetManifest` is NON-EMPTY and MATCHES the crane baseline's asset set, and the ducklake-catalog embed matches the crane baseline (expected: empty embed preserved) — reason: a wrong relative depth yields a binary that builds and runs but embeds nothing, so "compiles" is insufficient; parity against crane is the load-bearing assertion. Steps: `nix build .#ironstar-c2n .#ironstar-release-c2n` on aarch64-darwin; `result/bin/ironstar` runs (boots; the axum app has no clap `--help`/`--version`, so it starts the server — no missing-library/linker failure); the `assets.rs` AssetManifest is NON-EMPTY and equals the crane set (`bundle.BtE3nzmj.css`, `bundle.lI_X820P.js`, `datastar.CFmrOwYN.js`, `manifest.json`; `.map` excluded); the ducklake-catalog embed is empty in both (parity); migrations resolve from the member tree; eval ratio 1.23x ≤ 2x. See `measurements.md` Task 2.5 (a)-(d).

## 3. Migrate workspace-test and workspace-clippy off crane and delete the 20 per-crate packages

Goal: re-implement `workspace-test` (`cargo nextest run --workspace --no-tests=pass`, default profile for parity) and `workspace-clippy` (`cargo clippy --workspace --all-targets -- -D warnings`) as crane-free derivations over `combinedSrc` with the pinned toolchain, native deps (cc/cmake/perl/sqlite, pkg-config), and `HOME=/tmp`; delete the 20 per-crate `*-test`/`*-clippy` packages.
Independent of task 2; may proceed in parallel via a development join.
Depends on: 1.

- [ ] 3.1 Re-implement `workspace-test` as a crane-free nextest derivation over `combinedSrc` with the pinned toolchain, native deps, and `HOME=/tmp`.
- [ ] 3.2 Re-implement `workspace-clippy` as a crane-free clippy derivation with the same inputs, denying warnings.
- [ ] 3.3 Delete the 20 per-crate `*-test`/`*-clippy` packages generated by `genAttrs` over `libCrates`.
- [ ] 3.4 Acceptance and verify: `nix build .#checks.<sys>.workspace-test .#checks.<sys>.workspace-clippy`; compare the test count to the 911-test baseline (ignored network tests skipped); `nix eval .#checks.<sys> --apply builtins.attrNames` still shows 14 names; `nix eval .#packages.<sys> --apply builtins.attrNames` shows the 20 per-crate attrs gone.
- [ ] 3.5 Fallback flag: if the gate's dependency-recompile cost is unacceptable, fall back to one retained crane `buildDepsOnly` feeding only these two derivations, and surface the decision to the maintainer.

## 4. Drift detection — CI auto-regen and a no-network staleness check

Goal: extend `.github/workflows/regenerate-lock-files.yaml` to regenerate `Cargo.nix` on `Cargo.lock`/`Cargo.toml` changes (new trigger path plus a generate-and-amend step mirroring the existing flake.lock/bun.nix steps; reuse or adapt the concurrency group); add a pure flake check `cargo-nix-lock-sync` diffing the `Cargo.lock` `[[package]]` set against the `Cargo.nix` `crateName/version` set.
Depends on: 1.

- [ ] 4.1 Add the new trigger path (`Cargo.lock`/`Cargo.toml`) and the `crate2nix generate` plus amend step to `regenerate-lock-files.yaml`, reusing or adapting the concurrency group.
- [ ] 4.2 Add the pure no-network `cargo-nix-lock-sync` flake check diffing the two package sets and failing on a stale `Cargo.nix`.
- [ ] 4.3 Acceptance and verify: dry-run the workflow on a test branch with a trivial `Cargo.lock` change and confirm it regenerates and auto-commits `Cargo.nix`; `nix build .#checks.<sys>.cargo-nix-lock-sync` is green on a clean tree and red on a deliberate mismatch.

## 5. Substrate swap

Goal: rename `ironstar-c2n`→`ironstar` and `ironstar-release-c2n`→`ironstar-release`; point `default`, the e2e check binary, and CD at the crate2nix builds; remove the transition exclusions.
Depends on: 2, 3, 4.

- [ ] 5.1 Rename the `-c2n` packages to `ironstar`/`ironstar-release`, point `default`, the e2e check binary, and CD at them, and remove the transition `excluded` entries.
- [ ] 5.2 Acceptance and verify: `nix flake check` — all 14 checks green on both systems; `just e2e-test` passes against the crate2nix dev binary with the `/bin/ironstar` path identical; `nix build .#ironstar .#ironstar-release`.
- [ ] 5.3 Demonstrate via `nix build --dry-run` before and after a synthetic one-crate `Cargo.lock` bump that only that dependency-crate's cone is in the rebuild set on the niks3 cache.

## 6. Crane removal, cache-effectiveness verification, and docs cleanup

Goal: delete the crane input, `cargoArtifacts`/`buildDepsOnly`/`vendorCargoDeps`/`cargoVendorDir`, and the `crane.cachix.org` substituter; fix `.github/workflows/README.md` (it claims 12 checks including nonexistent workspace-fmt/pre-commit/nix-unit — verified stale; the live count is 14); remove the nix-unit flake input if confirmed dead (referenced only in comments; confirm the `treefmt-nix.follows` is not load-bearing before removal).
Depends on: 5.

- [ ] 6.1 Delete the crane flake input, all crane usage (`cargoArtifacts`, `buildDepsOnly`, `vendorCargoDeps`, `cargoVendorDir`), and the `crane.cachix.org` substituter.
- [ ] 6.2 Fix `.github/workflows/README.md` from 12 to 14 checks, removing the stale nonexistent names.
- [ ] 6.3 Remove the dead nix-unit flake input after confirming no load-bearing `follows` consumer.
- [ ] 6.4 Acceptance and verify: `rg -n 'crane|cargoArtifacts|buildDepsOnly|vendorCargoDeps|crane.cachix' modules/ flake.nix` is empty; `nix flake check` passes; the cache-granularity demo via `nix build --dry-run` before and after a synthetic lock bump shows a single-dep Renovate-style bump invalidating only the affected dependency-crate derivation — the migration's raison d'être, demonstrated.
