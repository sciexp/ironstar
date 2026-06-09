<!--
Raw capture of the design exploration for migrate-crane-to-crate2nix.

This file is a decision-log record of the brainstorming that preceded design.md.
It preserves the exploration as it happened: the prior beads epic input, the user's refactor directive,
the two-design judge panel, the adversarial verdicts, and the empirically settled keystone facts.
design.md reorganizes these decisions into a structured design document; the two are complementary and do not overlap.
-->

# Brainstorm: migrate ironstar from crane to crate2nix

## Background

Ironstar builds its 11-crate Rust workspace through crane.
Crane vendors the whole workspace's dependencies once (`vendorCargoDeps`) and produces a single workspace-wide `cargoArtifacts` blob via `buildDepsOnly`, which every downstream derivation (the dev binary, the release binary, workspace-test, workspace-clippy, and twenty ad-hoc per-crate test/clippy packages) shares for cache efficiency.
The consequence is coarse cache granularity.
A Renovate bump of a single dependency invalidates the entire `cargoArtifacts` cone, so the niks3 cache at cache.scientistexperience.net re-stores a large blob rather than the one dependency crate that actually changed.

The exploration began from a prior planning session that produced the beads epic ironstar-8g3 with seven children encoding a crate2nix adoption plan.
The user's directive was to refactor that plan rather than adopt it wholesale, treating 8g3 as input to a fresh, adversarially-verified design rather than as a frozen blueprint.
The epic is now frozen and serves only as a traceability pointer.

The driving question is whether moving the Rust build substrate to crate2nix — a committed `Cargo.nix` with per-crate `buildRustCrate` derivations — buys per-dependency-crate cache granularity worth the operational cost of a generated, committed build description that must stay in lockstep with `Cargo.lock`.

## Method

Two competing designs were authored and put before a judge panel.
Design A led with cache value: maximize per-dependency-crate derivation granularity and treat source injection as a problem to solve.
Design B led with operational simplicity: minimize the port surface, reuse the existing `combinedSrc` overlay verbatim, and frame the migration as a near-identity translation.

Three adversarial verdicts then stress-tested the contested claims: a cargo-economics verdict on the cache payoff, a correctness verdict on the source model and feature unification, and an operations verdict on CI, drift detection, and rollback.
Every keystone fact was re-verified against the local crate2nix clone (tip b873ca5) and the ironstar tree before being admitted to the merged design.
The decisive fact was confirmed empirically rather than by reading source alone.

## Decision chain

### Q1 — does crate2nix's per-member src match crane's whole-tree src, or isolate each crate?

This is the keystone, and the two designs disagreed on it head-on.
Design B asserted that a classic committed `Cargo.nix` sets every member's `src = ./.` with `workspace_member = crates/<name>`, so the whole tree is present in every build sandbox and `combinedSrc` can be reused verbatim.
Design A asserted that each member gets an isolated `src` rooted at its own subdirectory, so cross-crate parent reads must be injected.

Resolved empirically: generating a `Cargo.nix` for crate2nix's own `workspace_with_nondefault_lib` sample (members `crates/main`, `crates/somelib` — the same `crates/<name>` layout as ironstar) produced `src = ./crates/main` and `src = ./crates/somelib` with `workspace_member = null`.
`src/resolve.rs:83` sets the package path to the manifest's parent directory, and `:575` returns `./.` only when the package path equals the `Cargo.nix` directory, which holds only for a single root crate.
Design A's framing is correct; Design B's keystone is false.

A refinement that both designs and all three verdicts missed: the two read classes behave differently under per-member src.
The `include_str!` migration reads from `crates/ironstar/src/...` resolve to `crates/ironstar/migrations`, which is inside the ironstar member tree, so they are already safe — `combinedSrc` injects migrations there and nothing more is needed.
Only the two rust-embed reads that reach the workspace root break: `static/dist` from `crates/ironstar` and `assets/ducklake-catalogs` from `crates/ironstar-analytics-infra`, both via `$CARGO_MANIFEST_DIR/../../`.

### Q2 — how do the two parent-reaching members get their assets?

Override `src` for exactly those two members to a `combinedSrc`-derived subtree that re-establishes the `../../` parent layout above the crate root.
For each, a `runCommand` copies the member's own subdir to `$out/crates/<name>` and re-materializes `$out/static/dist` and `$out/assets/ducklake-catalogs` two levels up, so the parent-relative rust-embed reads resolve exactly as they do under crane today.
The other nine members keep the generated `./crates/<name>` src.

This is adopted as a deliberate, documented choice, not an accident of the tooling.
It forgoes per-local-member source-cache granularity for the two affected crates, but the robust cache win — per-dependency-crate derivations — is unaffected.
Both designs' framings of this mechanism are rejected: Design A's hand-waved "materializes the parents" and Design B's false "reuse combinedSrc verbatim, near-identity port."

A silent-empty footgun governs verification.
`assets.rs` embeds nothing if its folder is absent, so a wrong relative depth produces a binary that builds and runs but serves no UI or catalog.
Acceptance must assert the embedded manifest and catalog are non-empty, not merely that the crate compiles.

### Q3 — IFD or committed Cargo.nix?

Committed `Cargo.nix`, no import-from-derivation anywhere.
Ironstar deliberately avoids IFD: `modules/rust.nix:135` reads the workspace version via `fromTOML`/`readFile` of a plain file precisely so eval does not depend on a derivation build.
Crate2nix's `generatedCargoNix` path is IFD and is rejected; `nixConfig.allow-import-from-derivation` is explicitly not added.
Generation runs once on the dev host via `just regenerate-cargo-nix` (`nix run github:nix-community/crate2nix -- generate`) and commits `./Cargo.nix`.

This is safe and fully offline at build time.
`Cargo.lock` has 576 packages, 0 git sources, 0 alternate registries, and 565 crates.io checksums (the 11-package difference is the local path crates).
Every third-party crate's sha256 comes from `Cargo.lock` and bakes into `Cargo.nix` at generation time, so no `crate-hashes.json` is produced or needed.
`crate2nix generate` does not prefetch from the network for ironstar: `src/lock.rs:49-76` pulls hashes from `Cargo.lock`, and `src/sources.rs` prefetch fires only for sources lacking a lockfile checksum (git or alternate registry), of which there are none.
A single committed `Cargo.nix` encodes per-target cfg at nix-eval time, so it builds on both aarch64-darwin and x86_64-linux without per-target regeneration; do not commit per-target variants.

### Q4 — should per-crate test and clippy become first-class checks?

No.
Keep one workspace `cargo nextest` and one workspace `cargo clippy` as the single correctness gate, build no per-crate wrappers, and do not use `buildRustCrate runTests=true`.
All three verdicts and both designs converge here.

Per-crate nextest and clippy wrappers compile from source and do not consume `buildRustCrate`'s rlib artifacts, so they buy reporting granularity for zero cache value at a cost of roughly eighteen extra derivations.
`runTests=true` runs experimental cargo-test rather than nextest, which would lose the project's `--no-tests=pass`, the `[profile.ci]` retries and junit contract, and the 911-test baseline shape.
The genuine cache win — per-dependency-crate build derivations — accrues automatically from crate2nix regardless.

The twenty ad-hoc per-crate `*-test`/`*-clippy` items already live in `packages`, not `checks` (`modules/rust.nix:238-244`), so 8g3's "2 → 20 check expansion" conflated the two surfaces.
The ruling deletes those twenty packages, shrinking the package surface, and holds the check count invariant at 14.

### Q5 — drift detection between Cargo.lock and Cargo.nix?

Two layers.
Primary: extend `.github/workflows/regenerate-lock-files.yaml` to regenerate `Cargo.nix` when `Cargo.lock` or `Cargo.toml` change, running `crate2nix generate` (network allowed in GitHub Actions) and auto-committing.
This is real work: the workflow currently triggers only on `bun.lock`, `package.json`, `packages/**/package.json`, and `flake.nix`, with concurrency group `regenerate-bun-nix-*`; a new trigger path and a generate-and-amend step mirroring the existing flake.lock and bun.nix steps are required.
Secondary: a cheap no-network flake check `cargo-nix-lock-sync`, a pure derivation diffing the `[[package]] name@version` set in `Cargo.lock` against the `crateName/version` pairs in `Cargo.nix` and failing on a stale `Cargo.nix`.

A sandboxed full regenerate-and-diff check is rejected.
This corrects the cargo-economics and correctness verdicts, which claimed the rejection rested on network access: an in-sandbox regenerate is offline-feasible for ironstar because `Cargo.lock` carries all checksums.
The real reason to keep regeneration in CI is IFD plus buildbot nix-eval-jobs fanout, per the operations verdict.

### Q6 — feature unification risk under a default-features Cargo.nix?

Low but nonzero.
Crate2nix derives features from cargo-metadata's v2 resolver — the same unification cargo uses — so a single `Cargo.nix` generated with `rootFeatures = ["default"]` on the dev host is correct.
The top hotspot is `zenoh` with `default-features = false` selecting `transport_tcp`/`transport_tls`.
Both `ring` and `aws-lc-rs` are active on the resolved graph (both present in `Cargo.lock`), which makes the `aws-lc-sys` cmake/perl override mandatory rather than optional and elevates it above 8g3's enumerated override list.
The mitigation is the swap gate: a full build, the 911-test pass, and e2e on both aarch64-darwin and x86_64-linux before crane is removed.

### Q7 — overrides inventory?

A single `ironstarCrateOverrides` attrset merged onto `pkgs.defaultCrateOverrides`.
The verified `-sys` inventory from `Cargo.lock`: `libduckdb-sys`, `libsqlite3-sys`, `aws-lc-rs` plus `aws-lc-sys`, and `ring` are present; `openssl-probe` is present but there is no `openssl-sys`.
`libduckdb-sys` needs `stdenv.cc` and `HOME=/tmp` for its bundled C++ amalgamation.
`aws-lc-sys` needs cmake and perl and is mandatory.
`ring` gets perl as cheap insurance for its asm build.
`libsqlite3-sys` relies on the nixpkgs default override (pkg-config plus sqlite) and gets nothing added.
The workspace-wide pkg-config that crane carried is dropped, since only `libsqlite3-sys` needs it and the default covers it.
The Linux `openssl` buildInput is provisionally dropped because the TLS stack is rustls via ring/aws-lc-rs and no `openssl-sys` is in the lock; an `openssl-sys` override is added only if a transitive surfaces.
Darwin frameworks rely on the modern nixpkgs apple-sdk, with a per-crate buildInput added only if an aarch64-darwin build error surfaces.

### Q8 — devshell and auxiliary surfaces?

Near-zero port.
There is no `craneLib.devShell` (`modules/dev-shell.nix` uses plain `pkgs.mkShell`), rustfmt runs via treefmt (`modules/formatting.nix`) not crane, and there is no crane doc check (`cargoDocTest` is commented out and doctest is disabled).
The only coupling is `devShell.inputsFrom = builtins.attrValues self'.checks`; holding the check count at 14 keeps the closure class stable, and deleting the twenty per-crate packages reduces fan-out.
The pinned 1.94.1 rust-overlay toolchain must be threaded into `buildRustCrateForPkgs` (rustc and cargo override) so per-crate builds match fmt and devshell.
The e2e check consumes the dev binary (`self'.packages.ironstar`), which post-swap becomes the crate2nix dev build; the `/bin/ironstar` path identity must be verified.

### Q9 — coexistence and rollback?

Staged parallel `-c2n` packages with a single final swap commit.
Crane is a self-contained input with its own `crane.cachix.org` substituter (`flake.nix:29`, `:63`, `:69`), so it stays fully functional until deletion.
Through tasks 1 to 5 the migration is additive: `ironstar-c2n` and `ironstar-release-c2n` build alongside the untouched crane packages, added to the package-set-invariant `excluded` list for the transition.
Pre-swap rollback drops the `-c2n` packages; post-swap rollback reverts the single swap commit, with crane code still in git history and `Cargo.nix` inert.
`crane.cachix.org` is kept transitionally and removed only in the final cleanup.
Mergify required_checks is invariant: the buildbot umbrella names `buildbot/nix-build` and `buildbot/nix-eval` are stable under check-matrix expansion, so no mergify edits are needed.

### Q10 — eval cost?

Tolerable, gated empirically.
The committed ~576-crate `Cargo.nix` is parsed rather than generated at eval, which is cheaper than crane's `vendorCargoDeps` hashing, but derivation fanout grows by roughly 576 dependency-crate derivations that nix-eval-jobs must instantiate and that the niks3 cache tracks as many small paths instead of crane's two large deps blobs.
The swap is gated on measured eval time within roughly 2x of crane, with cold-cache fetch latency captured in the task 1 and task 2 acceptance.
This is recorded evidence feeding a human go/no-go before the substrate swap, not an automatic abort.

## Rejected alternatives

IFD via `appliedCargoNix`/`generatedCargoNix` — rejected: contradicts ironstar's deliberate IFD-avoidance pattern.

The 2 → 20 per-crate check expansion from 8g3 — rejected: per-crate wrappers compile from source and reuse no `buildRustCrate` artifacts, so they add reporting granularity with zero cache value.

`buildRustCrate runTests=true` — rejected: runs experimental cargo-test, not nextest, losing `--no-tests=pass`, the `[profile.ci]` contract, and the 911-test baseline shape.

A sandboxed full regenerate-and-diff drift check — rejected: an in-sandbox regenerate is offline-feasible for ironstar, so the rejection rests on IFD plus nix-eval-jobs fanout, not network; a cheap no-network package-set staleness check plus CI auto-regen covers drift more cheaply.

The hybrid (crane builds everything, crate2nix only for dependency caching) — rejected by orchestrator ruling as incoherent: nothing would consume the per-dependency derivations.
The legitimate fallback is a single retained crane `buildDepsOnly` feeding only the test/clippy gate, taken only if the gate's dependency-recompile cost proves unacceptable.

## Settled scope and open questions

Three orchestrator rulings are recorded and not reopened.
Scope is full crane replacement.
The ducklake-catalogs embed preserves today's empty-embed behavior (the `.db` is gitignored and filtered, so `ironstar-analytics-infra` silently embeds an empty catalog today); sourcing a real catalog is out of scope and recorded as a discovered follow-up.
The caching-payoff measurement is recorded evidence in the task 1 and task 2 acceptance, informing a human go/no-go before the swap rather than an automatic abort.
