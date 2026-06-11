## Context

Ironstar's 11-crate Rust workspace builds through crane.
Crane vendors the whole workspace's dependencies once and produces a single workspace-wide `cargoArtifacts` blob via `buildDepsOnly`, which every downstream derivation shares: the dev binary, the release binary, the `workspace-test` and `workspace-clippy` checks, and twenty ad-hoc per-crate `*-test`/`*-clippy` packages.
This yields coarse cache granularity.
A Renovate bump of one dependency invalidates the entire `cargoArtifacts` cone, so the niks3 cache at cache.scientistexperience.net re-stores a large blob rather than the single dependency crate that changed.

This change moves the Rust build substrate to crate2nix in its committed-`Cargo.nix` classic model, with no import-from-derivation anywhere.
Crate2nix emits one `buildRustCrate` derivation per crate, so a single-dependency bump invalidates only that dependency's cone in the cache.

Every keystone fact in this document was re-verified against the local crate2nix clone (tip b873ca5) and the ironstar tree.
The decisive per-member-src fact was confirmed empirically by generating a `Cargo.nix` for crate2nix's own `workspace_with_nondefault_lib` sample, whose `crates/<name>` layout is identical to ironstar's.

Constraints carried from the existing tree:
ironstar deliberately avoids IFD — `modules/rust.nix` reads the workspace version via `fromTOML`/`readFile` of a plain file precisely so eval does not depend on a derivation build.
The pinned Rust toolchain is 1.94.1 via rust-overlay, and this single-toolchain invariant must hold across the build, the devshell, and treefmt-driven fmt.
The check surface entered this change at 14 and lands at 27 (15 prior + 11 per-member `*-test` + `build-graph-invariants`); the devshell closure class stays stable because `devShell.inputsFrom = builtins.attrValues self'.checks` and the added checks carry lean closures (the per-member `crateWithTest` derivations are `stdenvNoCC` with empty `buildInputs`, the aggregate is a coreutils-only `linkFarm`, and `build-graph-invariants` is a pure `runCommand`).
`Cargo.lock` has 576 packages, 0 git sources, 0 alternate registries, and 565 crates.io checksums; the 11-package difference is the local path crates.

## Goals / Non-Goals

**Goals:**

Replace crane with crate2nix as the Rust build substrate, achieving per-dependency-crate nix cache granularity so a single Renovate dependency bump invalidates one crate's cone rather than the whole workspace blob.
Commit a `Cargo.nix` kept in lockstep with `Cargo.lock`, generated with no IFD.
Preserve the pinned 1.94.1 toolchain across all per-crate builds.
Preserve the test and clippy correctness gate at 933-passed/5-ignored parity, re-shaped as 11 per-member `runTests` checks plus a zero-cost `workspace-test` aggregate and a single workspace clippy gate.
Land the check surface at 27 (15 prior + 11 per-member `*-test` + `build-graph-invariants`) and preserve the embedded-assets behavior (non-empty static/dist manifest; ducklake-catalogs catalog).
Keep the migration additive and reversible until the substrate swap, removing crane and its substituter only at the end.

**Non-Goals:**

Crate decomposition changes (the v4y track), the bun/JS substrate, toolchain version bumps, and adoption of a `--profile ci` for the test gate are all out of scope.
Sourcing a real ducklake catalog is out of scope: today the `.db` is gitignored and filtered, so `ironstar-analytics-infra` silently embeds an empty catalog, and this design preserves that empty-embed behavior, recording catalog provenance as a discovered follow-up.
Per-local-member source-cache granularity for the two asset-reaching members is explicitly not a goal; the robust win is per-dependency-crate caching.

## Decisions

### D1: committed Cargo.nix, no IFD

**Choice.**
Add `crate2nix.url = "github:nix-community/crate2nix"` with `crate2nix.inputs.nixpkgs.follows = "nixpkgs"`.
Do not add `nixConfig.allow-import-from-derivation`.
Generate with default features (`rootFeatures = ["default"]`) via `just regenerate-cargo-nix` running `nix run github:nix-community/crate2nix -- generate` at the repo root, committing `./Cargo.nix`.

**Rationale.**
Ironstar avoids IFD by design, so committing `Cargo.nix` rather than using crate2nix's `generatedCargoNix` (which is IFD) keeps `nix flake check` and `nix build` fully offline.
Every third-party crate's sha256 comes from `Cargo.lock` and bakes into `Cargo.nix` at generation time, so no `crate-hashes.json` is produced or needed.
`crate2nix generate` does not prefetch from the network for ironstar: `src/lock.rs:49-76` pulls hashes from `Cargo.lock`, and `src/sources.rs` prefetch fires only for sources lacking a lockfile checksum (git or alternate registry), of which there are none.
Generate once on the dev host; a single committed `Cargo.nix` encodes per-target cfg at nix-eval time, so it builds on both aarch64-darwin and x86_64-linux without per-target regeneration.

**Alternatives considered.**
IFD via `appliedCargoNix`/`generatedCargoNix` — rejected for contradicting the deliberate IFD-avoidance pattern.
Committing per-target `Cargo.nix` variants — rejected because a single `Cargo.nix` encodes per-target cfg at eval time; Design A's "generate on build target" wording risked this and is not adopted.

### D2: rust.nix consumption with the pinned toolchain threaded in

**Choice.**
Import `./Cargo.nix` with `buildRustCrateForPkgs` overriding `rustc` and `cargo` to the pinned 1.94.1 `rustToolchain`, and merging `ironstarCrateOverrides` onto `pkgs.defaultCrateOverrides`:

```
cargoNix = import ./Cargo.nix {
  inherit pkgs;
  buildRustCrateForPkgs = p: p.buildRustCrate.override {
    rustc = rustToolchain; cargo = rustToolchain;
    defaultCrateOverrides = p.defaultCrateOverrides // ironstarCrateOverrides p;
  };
};
```

`packages.ironstar` is `cargoNix.workspaceMembers."ironstar".build` with `.override { release = false; }` (release is crate2nix's default, so dev needs the explicit false), and `packages.ironstar-release` is the same with `release = true`.

**Rationale.**
Threading the toolchain through `buildRustCrateForPkgs` preserves the single-toolchain invariant so per-crate builds match fmt and the devshell.
A pure workspace has no `rootCrate`; crate2nix exposes `workspaceMembers` and `allWorkspaceMembers`, so member builds use `workspaceMembers.<name>.build`.

**Alternatives considered.**
Using a `rootCrate` — not applicable; verified crate2nix exposes no `rootCrate` for a pure workspace.

### D3: per-member src and the source-injection mechanism

**Choice.**
For a `crates/<name>` workspace, the generated `Cargo.nix` sets each local member's `src = lib.cleanSourceWith { filter = sourceFilter; src = ./crates/<name>; }` with `workspace_member = null` — the member's own subdirectory, not `./.`.
The `include_str!` migration reads from `crates/ironstar/src/...` resolve to `crates/ironstar/migrations`, inside the ironstar member tree, so they are safe under per-member src with no extra work.
The two rust-embed reads reach the workspace root and break unless src is overridden: `#[folder = "$CARGO_MANIFEST_DIR/../../static/dist"]` in `crates/ironstar/src/infrastructure/assets.rs:25` and `#[folder = "$CARGO_MANIFEST_DIR/../../assets/ducklake-catalogs"]` in `crates/ironstar-analytics-infra/src/embedded_catalogs.rs:40`.

For exactly those two members, override `src` to a `combinedSrc`-derived subtree that re-establishes the `../../` parent layout above the crate root.
Concretely, a per-member derived tree `memberSrc.<name> = runCommand ... '' mkdir -p $out/crates; cp -r ${combinedSrc}/crates/<name> $out/crates/<name>; mkdir -p $out/static $out/assets; cp -r ${combinedSrc}/static/dist $out/static/dist; cp -r ${ducklakeCatalogsSrc} $out/assets/ducklake-catalogs ''`, set as that member's `src` via the override.
The other nine members keep the generated `./crates/<name>` src.

**Rationale.**
This was confirmed empirically: generating `Cargo.nix` for `workspace_with_nondefault_lib` (members `crates/main`, `crates/somelib`) produced `src = ./crates/main` and `src = ./crates/somelib` with `workspace_member = null`.
`src/resolve.rs:83` sets the package path to the manifest's parent directory, `:575` returns `./.` only when the package path equals the `Cargo.nix` directory (a single root crate), and `templates/Cargo.nix.tera:161/163` corroborate.
`buildRustCrate` unpacks only the member subdir, with no cd-into-member-from-workspace-root behavior.
The override re-establishes the parent layout, making the parent-relative reads work exactly as crane does today.
This is a deliberate, documented choice: it forgoes per-local-member source-cache granularity for the two affected crates, but the per-dependency-crate cache win is unaffected.
Crate2nix's `sourceFilter` is a denylist (it excludes only `.git`/`target`/IDE/`Cargo.nix`/editor-backups), so `.sql` files inside member dirs are retained automatically, and crane's bespoke `.sql` allowlist special-case becomes unnecessary.

**Alternatives considered.**
Design A's "materializes the parents" hand-wave and Design B's "reuse combinedSrc verbatim, near-identity port" framing — both rejected.
Design B's keystone (classic `Cargo.nix` sets every member `src=./.` with `workspace_member=crates/<name>`) is false per the empirical generation.

### D4: crate overrides inventory

**Choice.**
A single `ironstarCrateOverrides = pkgs: { ... }` attrset merged onto `pkgs.defaultCrateOverrides`:
`libduckdb-sys` gets `nativeBuildInputs ++ [ pkgs.stdenv.cc ]` and `HOME = "/tmp"` (bundled C++ amalgamation; HOME guards any build-time DuckDB write);
`aws-lc-sys` gets `nativeBuildInputs ++ [ pkgs.cmake pkgs.perl ]`, mandatory;
`ring` gets `nativeBuildInputs ++ [ pkgs.perl ]` (asm build; cheap insurance);
`libsqlite3-sys` relies on the nixpkgs default (pkg-config plus sqlite) and gets nothing added.
Do not add a workspace-wide pkg-config.
Provisionally drop the Linux `openssl` buildInput; add an `openssl-sys` override only if a transitive surfaces.
Darwin frameworks rely on the modern nixpkgs apple-sdk, with a per-crate buildInput added only if an aarch64-darwin build error surfaces.

**Rationale.**
Verified `-sys` inventory from `Cargo.lock`: `libduckdb-sys`, `libsqlite3-sys`, `aws-lc-rs` plus `aws-lc-sys`, and `ring` are present; `openssl-probe` is present but there is no `openssl-sys`.
Both `ring` and `aws-lc-rs` are active on the resolved graph, so `aws-lc-sys` is on the critical path and its override is mandatory, not optional — this elevates it above 8g3's enumerated override list.
Crane carried a workspace-wide pkg-config; under crate2nix only `libsqlite3-sys` needs it, and the default covers it.
The TLS stack is rustls via ring/aws-lc-rs and no `openssl-sys` is in the lock, so the Linux openssl buildInput is dropped provisionally; the test/clippy gate and the swap-gate builds catch any transitive surfacing.

### D5: packages — transitional and post-swap

**Choice.**
Transitional packages (additive, crane untouched): `ironstar-c2n` (dev), `ironstar-release-c2n` (release), and optionally `allWorkspaceMembers-c2n = cargoNix.allWorkspaceMembers` for a cheap compile-everything signal.
Post-swap, rename to `ironstar`/`ironstar-release` and delete the crane attrs.
Delete the twenty ad-hoc per-crate `*-test`/`*-clippy` packages, reducing the package surface.

**Rationale.**
The twenty per-crate items live in `packages`, not `checks` (`modules/rust.nix:238-244`), generated by `genAttrs` over ten `libCrates`; deleting them shrinks the package surface.
The package-set-invariant already excludes `*-test`/`*-clippy` suffixes and lists `ironstar-release` and `frontendAssets` as excluded; during the transition, add `ironstar-c2n`/`ironstar-release-c2n` to `excluded` and remove them at the swap when the names revert.

### D6: test and clippy strategy

The original choice (a single monolithic `cargo nextest` test gate plus a single `cargo clippy` gate) is superseded for the test side by the gate-shape revision below; the clippy side is unchanged.
The original reasoning is retained for provenance, then amended.

**Original choice (superseded for the test gate).**
Keep one workspace `cargo nextest` and one workspace `cargo clippy` as the single correctness gate.
Build no per-crate wrappers and do not use `buildRustCrate runTests=true`.
After crane removal, the gate is a crane-free derivation running `cargo nextest run --workspace --no-tests=pass` (default nextest profile for parity) and `cargo clippy --workspace --all-targets -- -D warnings` against `combinedSrc`, with the pinned toolchain plus native deps (cc/cmake/perl/sqlite, pkg-config) in `nativeBuildInputs` and `HOME=/tmp`.
Doctest stays disabled.

**Original rationale (superseded for the test gate).**
Per-crate nextest/clippy wrappers compile from source and do not consume `buildRustCrate`'s rlib artifacts, buying reporting granularity for zero cache value at roughly eighteen extra derivations.
This rationale held against *nextest wrappers* specifically; it did not anticipate the crate2nix `runTests` path, which reuses each crate's existing `buildRustCrate` build rather than compiling from source (one build of each crate serves the binary and its tests), so the per-member test checks do gain cache value, contradicting the original "zero cache value" finding for that mechanism.
`runTests=true` was rejected for running "experimental cargo-test rather than nextest, losing `--no-tests=pass`, the `[profile.ci]` retries/junit contract, and the 933-passed/5-ignored (938 defined) baseline shape"; the parity experiment (measurements.md, "Per-crate runTests parity experiment") found nextest's process-per-test isolation empirically unnecessary for parity — the cargo-test harness reproduces the gate's 933 passed / 5 ignored envelope exactly, including the `#[ignore]` network skips, and the pinned nixpkgs `build-rust-crate` builds both lib unit tests and `tests/` integration targets, so `--no-tests=pass` (a guard against empty test sets) is unneeded.

**Gate-shape revision (the current test-side end state).**
The monolithic nextest test gate is replaced by 11 per-member `runTests` checks — one per workspace member, named `ironstar-core-test` ... `ironstar-test` — plus a zero-build-cost aggregate that preserves the `workspace-test` check name.
Each per-member check is `(cargoNixDev.workspaceMembers.<name>.build.override { runTests = true; testPreRun = "export HOME=/tmp"; }).passthru.test`, the run-log derivation crate2nix's `crateWithTest` produces; a test failure fails that derivation by construction (the runner's `set -e` buildPhase propagates any non-zero test-binary exit), so every per-member check can fail.
The aggregate `workspace-test` is a `linkFarm` over the 11 per-member outputs: it forces each to build (their run logs are realized) while adding no compiler closure of its own, so the devshell `inputsFrom` and CI ergonomics that referenced `workspace-test` persist, and the buildbot/Mergify umbrella names are untouched.
`workspace-clippy` is unchanged — it remains the sole monolithic gate, one workspace-wide `cargo clippy --profile dev --locked --all-targets -- --deny warnings` over `combinedSrc`.

**Rationale (gate-shape revision).**
The parity experiment is the evidence: one dev-profile build of each crate is reused by both the binary and its tests; a leaf edit (a comment in `crates/ironstar-todo/src/lib.rs`) reruns exactly 2 of the 11 member test suites (`ironstar-todo` and `ironstar`, its reverse-dep) instead of recompiling and rerunning the whole 933-test workspace; the per-member sum reconstructs the baseline envelope exactly (933 passed / 5 ignored).
This is the per-crate test cache granularity that mirrors the per-dependency-crate build granularity the migration targets — the test side now shares the same finer regulator shape rather than being a single coarse one.
The `ironstar` binary member required the dSYM fix (step 3 below); once applied, its `tests/` integration targets (`chart_integration.rs`, `duckdb_integration.rs`, `layout_integration.rs`, `todo_feed.rs`) execute and pass, including the `multi_thread` tokio SSE tests in `todo_feed.rs` that were the latent threading-model risk flagged in the experiment.

**dSYM fix (step 3).**
On aarch64-darwin the dev profile (`-C debuginfo=2`) emits a `bin/ironstar.dSYM` directory beside the executable, and crate2nix's test runner stages the real binary with a non-recursive `cp ${crate}/bin/*` (`templates/nix/crate2nix/default.nix:191-193`) that aborts under `set -e` on that directory, blocking the ironstar member's tests.
The fix is a `postInstall` hook in `ironstarCrateOverrides.ironstar` that runs `rm -rf "$out"/bin/*.dSYM` after the binary is installed.
This is verified-supported in the pinned nixpkgs `build-rust-crate`: `postInstall` set in a crate override lands in `extraDerivationAttrs` (`default.nix:350`, since `postInstall` is not in `processedAttrs`) and wins the final `// extraDerivationAttrs` merge (`default.nix:579`) over the function-argument default, and `install-crate.nix` calls `runHook postInstall` after `cp -rP target/bin/* $out/bin`.
It requires no crate2nix fork-pin, no `Cargo.toml` profile change (the dev-profile debugging experience for local `cargo` users is unchanged), and no `Cargo.nix` hand-edit.
It is parity, not loss: crane's `buildPackage` installs the bare `ironstar` binary without a dSYM (verified — crane's `bin/` holds only `ironstar`, the c2n crate's `bin/` holds `ironstar` + `ironstar.dSYM`), so stripping the dSYM makes the c2n binary output match crane's, and it applies harmlessly to `packages.ironstar-c2n` (the same crate override feeds it).

**Warming cost (one-time).**
The first per-member `-test` build path threads `buildTests`/test-mode flags not previously cached for this override combination, so it recompiles a large dependency slice (≈180 crates, including the zenoh link/transport tree) once before any member test runs; thereafter member test derivations are individually cached and rerun only on reverse-dep change.
On warm CI caches the steady-state cost is the per-member figure, not the full-warming figure.

**Check-surface arithmetic note.**
The flake check surface lands at 27: the 15 prior checks are unchanged except that `workspace-test` becomes the zero-cost aggregate (no net name change), 11 per-member `*-test` checks are added (15 + 11 = 26), and task 4b's `build-graph-invariants` regulator (D9) adds the 27th (26 + 1 = 27).
The prior baseline was 14; task 4's `cargo-nix-lock-sync` raised it to 15 (flagged in measurements.md); the per-member revision raised it to 26; `build-graph-invariants` raises it to the final 27.
The per-member checks are a separate namespace from packages, so the package-set-invariant (`modules/checks/package-set-invariant.nix`) is unaffected — it operates over `self.packages` and already excludes `*-test` suffixes; no edit there is needed.

**Alternatives considered.**
8g3's 2 → 20 per-crate *nextest wrapper* check expansion — rejected (reporting-only, no cache value, compiles from source).
The crane-`buildDepsOnly`-sliver fallback (retain one crane derivation feeding the gate) is removed from the design entirely: per the orchestrator ruling there is no crane in the end state, and the per-member `runTests` shape needs no deps-provider sliver because it reuses the crate2nix per-crate builds directly.
A residual monolith remains only as `workspace-clippy`; a crane-free deps-prebuild that would speed its cold-cache compile is a possible follow-up, out of scope here.

### D7: drift detection

**Choice.**
Primary: extend `.github/workflows/regenerate-lock-files.yaml` to regenerate `Cargo.nix` when `Cargo.lock` or `Cargo.toml` change (new trigger path plus a generate-and-amend step mirroring the existing flake.lock/bun.nix steps; reuse or adapt the concurrency group), running `crate2nix generate` with network allowed in GitHub Actions and auto-committing.
Secondary: a cheap no-network flake check `cargo-nix-lock-sync`, a pure derivation diffing the `[[package]] name@version` set in `Cargo.lock` against the `crateName/version` pairs in `Cargo.nix` and failing on a stale `Cargo.nix`.

**Rationale.**
The workflow currently triggers only on `bun.lock`, `package.json`, `packages/**/package.json`, and `flake.nix`, with concurrency group `regenerate-bun-nix-*`, so adding the trigger path and the generate-and-amend step is real work.
The corrected hermeticity reasoning: an in-sandbox regenerate is offline-feasible for ironstar — `src/lock.rs:49-76` reads sha256s from `Cargo.lock` and `src/sources.rs` prefetch fires only for non-lockfile sources, of which there are none (565 crates.io checksums present).
So the reason to keep regeneration in CI is IFD plus buildbot nix-eval-jobs fanout, not network — correcting the cargo-economics and correctness verdicts' network claim, per the operations verdict.

**Alternatives considered.**
A sandboxed full regenerate-and-diff check — rejected because the rejection rests on IFD plus nix-eval-jobs fanout, and the cheap no-network staleness check covers staleness more cheaply.

### D8: devshell and auxiliary surfaces

**Choice.**
Near-zero port.
Thread the pinned 1.94.1 toolchain into `buildRustCrateForPkgs` so per-crate builds match fmt and the devshell.
Keep the devshell closure class stable across the check-surface growth to 27 by adding only lean-closure checks.

**Rationale.**
There is no `craneLib.devShell` (`modules/dev-shell.nix` uses plain `pkgs.mkShell`), rustfmt runs via treefmt (`modules/formatting.nix`) not crane, and there is no crane `cargoDoc` (`cargoDocTest` is commented out; doctest is false).
The only coupling is `devShell.inputsFrom = builtins.attrValues self'.checks`; the 27 checks keep the closure class stable because the added per-member test derivations are `stdenvNoCC` with empty `buildInputs`, the `workspace-test` aggregate is a coreutils-only `linkFarm`, and `build-graph-invariants` is a pure `runCommand`, while deleting the twenty per-crate packages reduces fan-out.
`frontendAssets` (pnpm/Rolldown), gitleaks, docs/eventcatalog (bun2nix), e2e (Playwright), treefmt, and the structure-package-set-invariant are all non-crane and unchanged.
The e2e check consumes the dev binary (`self'.packages.ironstar`); post-swap that becomes the crate2nix dev build, so the `/bin/ironstar` path identity must be verified.

### D9: build-graph observability — baseline-zero and the graph-drift regulator

This is the task-4b minimal slice: lock the current build-graph topology as baseline-zero and add a graph-drift regulator, so the substrate swap (task 5) and crane removal (task 6) produce measured, gated deltas rather than unobserved churn.
The full observability program (per-commit CI effects, a cache-hit metric, a time-series store, and fanout serialization) is a separate follow-up change and out of scope here.

**Choice.**
Three committed and wired artifacts plus a justfile recipe and a workflow lockstep extension.

The snapshot generator is a flake app, `apps.build-graph-snapshot`, modeled on `apps.regenerate-cargo-nix` in `modules/apps/regenerate.nix`.
It runs `nix derivation show -r` over the canonical roots (pinned to system `x86_64-linux`), normalizes the output to a hash-free key with an embedded `python3` script, and writes a small deterministic committed JSON at `modules/checks/build-graph-snapshot.json`.
The committed snapshot has sorted keys, no store hashes, and no timestamps, so running the app twice produces byte-identical output.

The committed baseline is `modules/checks/build-graph-baseline.json`: the accepted ceilings the regulator gates against.
The graph-drift regulator is `modules/checks/build-graph-invariants.nix`: a pure `runCommand` content-addressed via `builtins.path` on exactly the snapshot and the baseline (the same discipline as `cargo-nix-lock-sync`), with no recursive nix and no IFD.

**Hermeticity constraint (the load-bearing correction).**
A flake check derivation runs inside the nix build sandbox, which cannot invoke `nix eval` or `nix derivation show` (no recursive nix).
So the graph cannot be re-derived inside the regulator the way `cargo-nix-lock-sync` re-derives the `Cargo.lock`/`Cargo.nix` package sets from plain files.
The design splits the work accordingly: the app re-derives the graph projection outside the sandbox (where recursive nix is available) and commits the snapshot; the regulator is a pure derivation that validates the committed snapshot against the committed baseline.
The snapshot is the envelope, the regulator is the gate; the app is the means of refreshing the envelope.

**Canonical system and root decisions.**
The committed baseline is pinned to a single canonical system, `x86_64-linux`, which is what buildbot CI evaluates and which evals purely from the committed `Cargo.nix` with no IFD.
Cross-system eval of every canonical root from a darwin host was confirmed: `nix derivation show -r` over each x86_64-linux root succeeds without a linux builder because the crate2nix roots carry no import-from-derivation.
The canonical roots are Rust-core only: post-swap (task 5) these are `packages.ironstar`, `packages.ironstar-release`, `checks.workspace-clippy`, `checks.workspace-test`, `checks.cargo-nix-lock-sync`, and the 11 per-member `*-test` checks (during the additive transition tasks 1 through 4 these two package roots carried the `-c2n` suffix, renamed at the swap).
`checks.ironstar-e2e` is excluded: it is IFD-bound on linux (it requires building the bun2nix frontend asset during eval, which fails with no linux builder).
The two member-source derivation names retain the frozen historical `-c2n-member-src` token deliberately (`ironstar-c2n-member-src` and `ironstar-analytics-infra-c2n-member-src`): they are content-addressed names whose stability the build relies on, and renaming them would force member rebuilds, so the token persists past crane removal as a fixed-name string with no remaining crane reference.

**Normalized key and what a ceiling means.**
Each node is keyed on the hash-free tuple `(system, logical_pname, version, build_class, profile, member_scope)`, dropping the store hash entirely.
`logical_pname`/`version` come from the crate2nix `crateName`/`crateVersion` env for crate-compile nodes and from the derivation name otherwise; `build_class` is one of `crate-compile`, `crate-test-compile`, `test-runner`, `vendor-blob`, `other`, derived from the `crateName`/`buildTests` env and the name; `profile` is `dev`/`release`/`test`/`na` parsed from the `release` and `buildTests` env booleans, not from the hash; `member_scope` is the workspace member whose `buildTests=1` variant produced a node, else `shared`.
Duplication is measured as the count of distinct store hashes that collapse onto one logical key — the distinct-compile multiplicity of that crate at that profile.
The gated duplication ceilings are scoped to `build_class ∈ {crate-compile, crate-test-compile}` only; nixpkgs stdenv-stage bootstrap variance (clang-wrapper, perl, source-fetch drvs appearing 5-9 times across bootstrap stages) is non-actionable, not ironstar-specific, and is recorded report-only rather than gated, because it drifts on any nixpkgs bump.

**Pinned-as-baseline policy for persistent duplication.**
Two duplication classes are intrinsic to crate2nix and survive crane removal: the dev/release profile split (a release artifact is a full second compile of the dependency closure, sharing essentially nothing with the dev compile) and per-member test-variant feature unification (each member's `buildTests=1` variant unifies dev-deps differently, producing distinct compile derivations of shared dependency crates).
These are encoded as the accepted baseline level, not as reduction targets: the regulator fails on growth above the committed ceiling, not on the duplication's existence.
The heavy-crate distinct-compile table (zenoh, tokio, sqlx and its sub-crates, the DuckDB/SQLite C-binding crates, arrow, moka, rkyv) is gated the same way — a ceiling per crate that fails on growth.
The `vendor_monolith_count` field records the distinct vendor-blob derivations the snapshot actually sees, which under the canonical roots is only what `workspace-clippy`'s `importCargoLock` vendor contributes; it is gated against growth rather than required to drop, because `workspace-clippy`'s vendoring is independent of crane and persists past task 6.

**CCV framing.**
The snapshot is the operating envelope (the realized build-graph shape the system must stay within); the regulator is the regulator (it fails when the realized snapshot leaves the committed envelope).
Together they form one envelope-plus-regulator pair that composes into ironstar's existing closure operator alongside `cargo-nix-lock-sync` and `structure-package-set-invariant`.
The graph-drift regulator is a sibling of `cargo-nix-lock-sync`, not a replacement: `cargo-nix-lock-sync` gates the crate *set* (every `Cargo.lock` package appears in `Cargo.nix`), while this regulator gates the crate set's *duplication structure* and workspace-member presence, which the set-level check cannot see.
Regulator integrity requires a red demonstration: a deliberate ceiling perturbation must make the check fail with an actionable message before the green state is trusted.

**Follow-up program (out of scope).**
Per-commit snapshot upload via a herculesCI effect, a per-invocation cache-hit-rate metric on buildbot, a DuckDB time-series over per-commit invariants for trend queries, and fanout serialization to cure the nix-eval-jobs closure-edge over-count are all deferred to a separate observability change.
This slice locks baseline-zero and gates against duplication regrowth; the time-series and CI-effect machinery build on top of it.

## Risks / Trade-offs

[Risk] Parent-relative asset injection (highest).
The per-member derived src that re-establishes `../../static/dist` and `../../assets/ducklake-catalogs` above `crates/ironstar` and `crates/ironstar-analytics-infra` is the load-bearing unknown; `assets.rs` silently embeds nothing if the folder is absent, so a wrong relative depth produces a binary that builds and runs but serves no UI or catalog.
→ Mitigation: task-2 acceptance asserts the embedded manifest and catalog are non-empty (smoke run or explicit assertion), not merely "compiles"; empirical depth verification before declaring task 2 done; e2e in task 5 catches end-to-end.

[Risk] Ducklake-catalogs provenance.
The catalog embeds empty today (gitignored `.db`, filtered out); if a real catalog is expected, the migration would need to source it, and a gitignored `.db` cannot come from the flake source.
→ Mitigation: default to preserving the empty-embed status quo; surfaced as an open question.

[Risk] Feature divergence (zenoh `default-features=false`; ring-vs-aws-lc-rs).
A default-features `Cargo.nix` could resolve `transport_tls` or the crypto path differently from cargo's workspace unification, breaking SSE/Zenoh at runtime.
→ Mitigation: a full 933-passed/5-ignored (938 defined) test pass plus e2e on both aarch64-darwin and x86_64-linux is the swap gate; do not swap until both platforms are green.

[Risk] Eval and cache fanout.
Roughly 576 dependency-crate derivations grow nix-eval-jobs instantiation cost and the niks3 object count (many small paths vs two large blobs), raising cold-cache CI fetch latency.
→ Mitigation: record eval time and cold-cache fetch time in task-1/2 acceptance; gate the swap on eval ≤ ~2x crane; if exceeded, reconsider the hybrid fallback.

[Risk] Test-gate dependency recompile.
The original monolithic crane-free test/clippy derivation recompiled all deps on any change (no per-crate cache in the gate).
→ Resolution (test side): the gate-shape revision (D6) replaces the monolithic test gate with 11 per-member `runTests` checks that reuse the crate2nix per-crate builds, so a leaf edit reruns only the touched member plus its reverse-dep cone (measured: 2 of 11 suites on an `ironstar-todo` edit), not all deps.
The crane-`buildDepsOnly` fallback is removed (no crane in the end state); the only residual monolith is `workspace-clippy`, whose cold-cache compile is the remaining recompile exposure, mitigated on warm CI caches and addressable by a future crane-free deps-prebuild (out of scope).
A one-time warming cost remains: the first per-member `-test` build recompiles a ≈180-crate dependency slice not previously cached for the test-mode override, after which member tests are individually cached.

[Risk] Darwin framework gaps.
`security-framework-sys`/`core-foundation-sys` framework needs on aarch64-darwin are unverified.
→ Mitigation: detected by the task-2 darwin build; add a single per-crate buildInput override on the offending `-sys` crate.

[Risk] Cargo.nix staleness on non-Renovate PRs.
A developer bumping `Cargo.lock` locally without regenerating, on a PR the CI trigger paths do not catch.
→ Mitigation: the `cargo-nix-lock-sync` staleness check (task 4) is the belt-and-suspenders; CI auto-regen handles Renovate.

[Risk] Content-addressed name stability.
The `combinedSrc`-derived per-member srcs must keep fixed `name`s or unrelated commits trigger full rebuilds (crane uses `name="ironstar-src"` for this).
→ Mitigation: preserve fixed names in the derived-src runCommands; a no-op commit should not rebuild.

[Risk] nix-unit input removal.
Removing the input could break a transitive follows (`treefmt-nix.follows`).
→ Mitigation: gate removal on confirming no load-bearing consumer (verified only comment-level references today); `nix flake check` after removal.

[Trade-off] Per-local-member source-cache granularity is forgone for the two asset-reaching members, which share a derived `combinedSrc` subtree.
→ Accepted because the robust win is per-dependency-crate caching, which is unaffected, and the two members must re-establish the parent layout anyway.

## Migration Plan

The migration is staged across six tasks, additive and reversible until the substrate swap.

Tasks 1 through 5 are additive: parallel `ironstar-c2n` and `ironstar-release-c2n` packages build alongside the untouched crane packages, with crane (a self-contained input carrying its own `crane.cachix.org` substituter) fully functional throughout.
Task 1 bootstraps the crate2nix input and the committed `Cargo.nix`.
Task 2 adds crate overrides, the parallel packages, and the source injection for the two parent-reaching members.
Task 3 migrates the workspace test and clippy gate off crane and deletes the twenty per-crate packages.
Task 4 adds drift detection.
Task 5 is the substrate swap: rename the `-c2n` packages, point `default`, the e2e check binary, and CD at the crate2nix builds, and remove the transition exclusions.
Task 6 removes crane, the `crane.cachix.org` substituter, and stale docs, and demonstrates the cache-granularity payoff.

The swap (task 5) is gated on all 27 checks green and the 933-passed/5-ignored (938 defined) test pass plus e2e on both aarch64-darwin and x86_64-linux, with a human go/no-go informed by the eval-time and cold-cache measurements captured in tasks 1 and 2.

Rollback.
Pre-swap rollback drops the `-c2n` packages.
Post-swap rollback reverts the single swap commit; crane code remains in git history and `Cargo.nix` is inert.
`crane.cachix.org` is kept transitionally and removed only in the final cleanup.
Mergify required_checks is invariant: the buildbot umbrella names `buildbot/nix-build` and `buildbot/nix-eval` are stable under check-matrix expansion, so no mergify edits are needed.

## Deviations from beads epic ironstar-8g3

The prior planning session's epic ironstar-8g3 (seven children) is the input; the user's directive was to refactor it, and the epic is now frozen.
The following deviations are deliberate.

1. Source model.
8g3's single combinedSrc overlay injecting frontend assets and SQL migrations into the binary crate is retained in spirit but re-scoped: under crate2nix's per-member src, the `include_str!` migration reads are already safe (inside `crates/ironstar/migrations`), so only the two rust-embed reads reaching the workspace root need a per-member derived-src override.
The final design uses a targeted per-member src override for exactly two crates and lets the other nine use the generated subdir src.

2. Check surface.
8g3 planned per-crate test and clippy *nextest-wrapper* derivations promoted to first-class checks; those wrappers compile from source and reuse no `buildRustCrate` artifacts, so the design rejected them and deleted the twenty per-crate packages.
The gate-shape revision (D6) instead promotes 11 per-member *`runTests`* test checks — distinct from 8g3's wrappers in that they reuse each crate's existing `buildRustCrate` build and thus carry real cache value — plus a zero-cost aggregate preserving the `workspace-test` name; clippy stays a single workspace gate.
Net surface after this revision is 27 (15 prior, of which `workspace-test` becomes the aggregate, plus 11 per-member `*-test`, plus the task-4b `build-graph-invariants` regulator), not the 14 the original design and task-5 acceptance assert.
8g3's "2 → 20" still correctly noted that the twenty *packages* it conflated with checks already existed as packages and were deleted; the new per-member checks are a separate namespace and add no packages.

3. Nextest config-file flag dropped.
8g3 specified `cargo nextest run -p <crate> --config-file .config/nextest.toml --profile ci`.
The final design uses the default profile for the workspace gate, since `--profile ci` is used nowhere today, so default equals parity with the current crane check; selecting `--profile ci` for buildbot junit is deferred.

4. Drift.
8g3's "both sandboxed cargo-nix-fresh and CI auto-regen" becomes CI auto-regen plus a cheap no-network staleness check; the sandboxed full regenerate is dropped because the rejection rests on IFD plus nix-eval-jobs fanout, not network.

5. pkg-config workspace-wide dropped.
8g3 carried crane's global pkg-config; the final design adds it per-crate only where needed (`libsqlite3-sys`, covered by the nixpkgs default).

6. openssl buildInput on Linux provisionally dropped.
8g3 kept crane's `optionals isLinux [openssl]`; the TLS stack is rustls via ring/aws-lc-rs with no `openssl-sys` in the lock.

7. aws-lc-sys override elevated to mandatory (absent from 8g3's enumerated list), since ring and aws-lc-rs are both active.

8. Ducklake-catalogs injection added.
8g3 and combinedSrc today do not inject `assets/ducklake-catalogs`; the per-member src override for `ironstar-analytics-infra` must inject it, preserving the empty-embed status quo.

9. IFD posture made explicit.
8g3 said "committed Cargo.nix (no IFD)"; the final design grounds this in the verified existing IFD-avoidance pattern and explicitly forbids `allow-import-from-derivation` and `generatedCargoNix`.

10. Docs cleanup (workflows/README.md 12 → 27, nix-unit input removal) folded into task 6, beyond 8g3's scope; nix-unit removal gated on confirming no load-bearing follows.

## Open Questions

These are recorded for the human go/no-go and were settled by orchestrator rulings where noted.

Migration scope: full crane replacement versus hybrid.
Per-member source isolation means the per-local-member cache benefit of crate2nix for ironstar is near-zero (the two asset-reaching members share a derived subtree anyway); the robust win is purely per-dependency-crate caching.
A hybrid where crane keeps building the binary and a single `buildDepsOnly` feeds the test/clippy gate, with crate2nix added only for dependency-crate granularity, is materially lower-risk on the asset-injection hazard.
Orchestrator ruling: full replacement; the hybrid as a whole is rejected as incoherent because nothing would consume the per-dependency derivations, and the only legitimate fallback is the single retained `buildDepsOnly` feeding the gate (D6).

Ducklake-catalogs intent: whether `ironstar-analytics-infra` is supposed to embed a real catalog at build time (silently shipping nothing today) or whether empty-embed is intended.
Orchestrator ruling: preserve the empty-embed status quo; sourcing a real catalog is out of scope and recorded as a discovered follow-up.

Caching-payoff measurement gate: whether to require a measured demonstration that a single Renovate dependency bump under crane today rebuilds a large `cargoArtifacts` blob versus crate2nix's single-cone rebuild.
Orchestrator ruling: capture the delta as recorded evidence in tasks 1 and 2 (eval time vs ~2x crane; synthetic one-dep bump rebuild demo), informing a human go/no-go before task 5, not an automatic abort.
