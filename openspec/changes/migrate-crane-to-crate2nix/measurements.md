# Migration measurements

This file records timing and size measurements captured during the crane-to-crate2nix migration, factual and reproducible.

## CCV no-leak note

The `cargo-nix-lock-sync` lockstep regulator that holds `Cargo.nix` in sync with `Cargo.lock` lands in task 4.
Until task 4 ships, the manual reconciliation performed in task 1.4 is the stand-in regulator for `Cargo.nix`/`Cargo.lock` drift.

## Task 1.5 — crane baseline and raw Cargo.nix instantiation

Host: aarch64-darwin (Darwin 25.2.0); crate2nix locked at rev `c994c83` (crate2nix 0.15.0).

### (a) Crane baseline drvPath eval

Command: `time nix eval .#packages.aarch64-darwin.ironstar.drvPath`.
Run 1 (cold-ish): 1.833s total (0.68s user, 0.27s system).
Run 2 (warm): 1.282s total (0.68s user, 0.19s system).
Resolved drvPath both runs: `/nix/store/bwkkkpq5vwyb9v1hj1zb76cyhws4ljn5-ironstar-0.1.0.drv`.

### (b) Raw Cargo.nix instantiation (read + force)

Command: `time nix eval --impure --expr 'builtins.seq (builtins.readFile ./Cargo.nix) "parsed"'`.
Run 1: 0.077s total (0.03s user, 0.02s system).
Run 2: 0.070s total (0.03s user, 0.02s system).
This measures only file read and force, not the per-crate derivation instantiation that requires the package wiring added in task 2.

### Cargo.nix size

Bytes: 851635 (832K after treefmt formatting; the pre-format generated file was 768748 bytes).
Lines: 29457.

## Task 2.5 staging note

The `ironstar-c2n` eval timing that feeds the ≤2x gate is deferred to task 2.5 because the `ironstar-c2n` package does not exist until task 2 wires `Cargo.nix` into `modules/rust.nix`.
At task 1 only `Cargo.nix` exists; nothing imports it, so `nix eval .#packages.<sys>.ironstar-c2n` has no attribute to time yet.

## Task 2.5 — c2n eval timing, build wall-clock, and embed parity

Host: aarch64-darwin (Darwin 25.2.0); crate2nix locked at rev `c994c83`; nixpkgs `addf7cf5`.

### (a) Eval timing against the crane baseline (≤2x gate)

Command: `time nix eval .#packages.aarch64-darwin.ironstar-c2n.drvPath`.
Warm run 1: 1.745s. Warm run 2: 1.579s. Warm run 3: 1.577s. Steady warm ≈ 1.58s.
Crane baseline warm (re-measured this session): 1.319s, 1.256s; steady warm ≈ 1.28s.
Ratio: 1.58 / 1.28 ≈ **1.23x** (well under the 2x gate; original task-1 crane baseline range 1.282–1.833s also bounds it).
Resolved c2n drvPath: `/nix/store/rggikm7l6jf35zvsdvazqzn5n73di41j-rust_ironstar-0.1.0.drv`.

### (b) Build wall-clock and derivations built (aarch64-darwin)

`nix build .#ironstar-c2n` (dev): the third-party dependency cone (~565 crate derivations + the libduckdb-sys amalgamation) was served from the niks3/local store; the 11 local-member derivations plus the `ironstar` binary compiled locally.
Final `ironstar` binary buildPhase: 34s (per build log); total wall-clock for the dev package from a warm dependency store: ~33 min elapsed in this session, dominated by the libduckdb-sys C++ amalgamation and aws-lc-sys/ring compiles that were not yet cached for the pinned-toolchain override (these are the override-touched crates whose hashes diverge from any pre-existing cache entry).
Derivations built locally (not substituted): the override-touched `-sys` crates (libduckdb-sys, aws-lc-sys, ring) and their dependents up through the 11 local members and the binary. The remaining ~560 third-party crate derivations were store-resident.
Dev binary out: `/nix/store/yyx73zq73hpxkzlh2yic1r2fpzz9kq1s-rust_ironstar-0.1.0` (273 MB, debuginfo dev profile).

### (c) Embed parity — crane baseline vs c2n (method: embedded byte-pattern inspection)

`frontendAssets` (`/nix/store/4jhs6ys3zl6qnvykssli738ns5p5cb17-ironstar-frontend-0.1.0`) static/dist set:
`bundle.BtE3nzmj.css`, `bundle.lI_X820P.js`, `datastar.CFmrOwYN.js`, `manifest.json` (the rust-embed-eligible set: `*.css|*.js|*.json`, `*.map` excluded).
manifest.json: `{ "bundle.css": "bundle.BtE3nzmj.css", "bundle.js": "bundle.lI_X820P.js", "datastar.js": "datastar.CFmrOwYN.js" }`.

c2n dev binary (`/nix/store/yyx73zq…-rust_ironstar-0.1.0`, 273 MB) via `strings -a … | rg -F`: all four asset filenames FOUND; the manifest body strings (`bundle.css`, `datastar.js`) present.
c2n release binary (`/nix/store/59dqlaad…-rust_ironstar-0.1.0`, 66 MB): all four asset filenames FOUND; identical embedded-content layout (`…bundle.BtE3nzmj.css.map*/bundle.lI_X820P.js…manifest.json{`), confirming the rust-embed include order css→js→datastar.js→manifest.json.

`.map` exclusion note: the single `.css.map`/`.js.map` token that appears in both dev and release binaries is the `sourceMappingURL` comment *inside* the embedded `.css`/`.js` bundle content (`/*# sourceMappingURL=bundle.BtE3nzmj.css.map*/`), not an embedded `.map` file. No `.map` file is a rust-embed entry; the `#[exclude = "*.map"]` is honored. Both binaries show the identical pattern.

Both binaries consume the identical `frontendAssets` derivation (`/nix/store/4jhs6ys3zl6qnvykssli738ns5p5cb17-ironstar-frontend-0.1.0`), so the asset set is identical to crane's by construction.

Crane-binary cross-check (the load-bearing parity assertion). Crane baseline `nix build .#ironstar` -> `/nix/store/75m99xskra1n833x0cpkqdsxrzz24zkp-ironstar-0.1.0` (188 MB). Extracting the embedded asset-token set (`strings -a | rg -o '(bundle\.…\.(css|js)|datastar\.…\.js|manifest\.json)' | sort -u`) from crane, c2n-dev, and c2n-release and diffing:
crane vs c2n-dev: IDENTICAL. crane vs c2n-release: IDENTICAL.
Token set in all three: `bundle.BtE3nzmj.css`, `bundle.lI_X820P.js`, `datastar.CFmrOwYN.js`, `manifest.json` (plus the manifest-body boundary token `bundle.cssdatastar.js`). The c2n `AssetManifest` is NON-EMPTY and MATCHES the crane baseline exactly. Ducklake catalog: crane also shows `No embedded catalogs` and the same `ducklake:hf://…space.db` network URI -> empty-embed parity confirmed on both sides.

Injected member tree (`/nix/store/ym9jknj…-ironstar-c2n-member-src`) layout verified:
`crates/ironstar/migrations/{001_events.sql,002_sessions.sql}` present (so `include_str!` migrations resolve from the member tree), and `static/dist/{bundle.BtE3nzmj.css,bundle.lI_X820P.js,datastar.CFmrOwYN.js,manifest.json,+.map siblings}` two levels above the crate root (so `#[folder = "$CARGO_MANIFEST_DIR/../../static/dist"]` resolves; `.map` siblings present in source but excluded by the rust-embed attribute).

### (d) Ducklake-catalog embed (empty-embed parity)

`ironstar-analytics-infra` member src injects an empty `assets/ducklake-catalogs` tree (the `.db` is gitignored and crane's combinedSrc never injected this path), preserving the empty-embed status quo (design D5 / Open Questions orchestrator ruling). The `DuckLakeCatalogs` rust-embed struct contains no entries; `attach_all` is a no-op, identical to crane.

Verification on the c2n release binary: no `.db` file is embedded. The `space.db` string that appears in the binary is the hardcoded runtime network-ATTACH fallback URI `ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db` from `main.rs`, accompanied by the log strings `No embedded catalogs, trying network ATTACH` and `Attached DuckLake catalog via network` — confirming the embed is empty and the runtime falls back to network, exactly as under crane.

## Task 3 — crane-free workspace-test/clippy gate

Host: aarch64-darwin (Darwin 25.2.0 arm64); nixpkgs `addf7cf5`; pinned rust 1.94.1.

### Gate derivation shapes

Both gates are plain `pkgs.stdenv.mkDerivation` (no crane) built by a shared `mkWorkspaceGate` helper in `modules/rust.nix`:

- src = `combinedSrc` (the same runCommand the crane checks consume).
- Offline vendored deps: `cargoDeps = rustPlatform.importCargoLock { lockFile = self + "/Cargo.lock"; }` wired via `rustPlatform.cargoSetupHook`. This is the canonical nixpkgs vendoring idiom, fully crane-free, reading the same `Cargo.lock` crane vendors. Fully offline: the lock has 565 crates.io checksums and 0 git sources, so `importCargoLock` needs no `outputHashes` and prefetches nothing. No IFD: the lockfile is a plain file read at eval time.
- Toolchain: the pinned 1.94.1 `rustToolchain` (carries cargo/rustc/clippy) in `nativeBuildInputs`; build log confirms `cargo 1.94.1 (29ea6fb6a 2026-03-24)`.
- Native deps: `stdenv.cc` + `cmake` + `perl` + `pkg-config` (the libduckdb-sys/aws-lc-sys/ring `-sys` crates this single-derivation workspace compile needs), plus `sqlite` buildInput on Linux for libsqlite3-sys. `HOME=/tmp` as crane. `dontUseCmakeConfigure=true` neutralizes cmake's setup-hook configurePhase so cargoSetupHook's vendoring stands.

### Exact commands (replicate the crane checks)

- `workspace-test`: `cargo nextest run --cargo-profile dev --no-tests=pass` (crane's `commonArgs` set `CARGO_PROFILE=dev`, so the crane nextest ran with `--cargo-profile dev`; default nextest test profile, no `--profile ci`; no `--run-ignored`, so the network `#[ignore]` tests are skipped exactly as the crane check skips them).
- `workspace-clippy`: `cargo clippy --profile dev --locked --all-targets -- --deny warnings` (crane's `cargoClippy` defaulted `cargoExtraArgs=--locked` and ran via `cargoWithProfile`, which with `CARGO_PROFILE=dev` injects `--profile dev`; `--deny warnings` is `-D warnings`).

### Build wall-clock (aarch64-darwin, dependency cone substituted, members + binary compiled locally)

Both gates built concurrently from a warm dependency store. Each gate is its own derivation and compiles the full workspace + all deps from source within it (no shared cargo `target` cache between the two, by design — the genuine cache win is per-dependency-crate `buildRustCrate` derivations on the build/release packages, not on these gates).

- `workspace-clippy`: exit 0, clean (no clippy warnings or errors — `--deny warnings` passed). `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 7m 25s`; full derivation wall-clock 530s (~8.8 min).
- `workspace-test`: exit 0. Compile `Finished 'dev' profile ... in 7m 35s`, then nextest run `Summary [29.525s] 933 tests run: 933 passed, 5 skipped`; full derivation wall-clock 591s (~9.9 min).

Dep-recompile cost assessment: acceptable. The dominant cost is the libduckdb-sys C++ amalgamation and aws-lc-sys/ring compiles under the pinned-toolchain override (not yet in any binary cache, as in task 2.5). On warm CI caches these `-sys` paths become substitutable and the gate wall-clock drops to local member + binary compile + test execution. The documented fallback (a retained crane `buildDepsOnly` feeding only these two gates) is NOT taken: ~9-10 min per gate from a warm dep store is within the prior crane-gate envelope, and the fallback would re-introduce the crane input these tasks exist to remove. See Task 3.5 below.

### Test-count parity vs the baseline (CCV adequacy bar)

`933 tests run: 933 passed, 0 failed, 5 skipped` → 938 tests total defined. The 5 skipped are exactly the 5 network-requiring `#[ignore]` test attributes (`crates/ironstar-analytics-infra/src/analytics.rs:371,409`; `crates/ironstar/tests/duckdb_integration.rs:23,102`; `crates/ironstar/tests/chart_integration.rs:119`). The crane `workspace-test` runs the identical command on the identical `combinedSrc` with the same toolchain and the same no-`--run-ignored` behavior, so it discovers and runs the identical set and skips the identical 5 — exact parity. The CLAUDE.md "911-test baseline" is stale (the workspace grew to 938 total); the `just rust-check-full` path uses `--run-ignored all` and would run all 938. No divergence from the crane gate's behavior; surface-back trigger #2 does not fire.

### Check / package surface

- `nix eval .#checks.aarch64-darwin --apply builtins.attrNames` → 14 names exactly: `dev-platform`, `gitleaks`, `ironstar`, `ironstar-docs`, `ironstar-docs-e2e`, `ironstar-docs-unit`, `ironstar-e2e`, `ironstar-eventcatalog`, `ironstar-eventcatalog-e2e`, `ironstar-eventcatalog-unit`, `structure-package-set-invariant`, `treefmt`, `workspace-clippy`, `workspace-test`. Count 14 → 14 held.
- `nix eval .#packages.aarch64-darwin --apply builtins.attrNames` → the 20 per-crate `*-test`/`*-clippy` attrs are gone (none remain). Package count 35 → 15 (the full delta of exactly 20 removed attrs; the remaining 15 are `default`, `dev-platform`, `frontendAssets`, `ironstar`, `ironstar-c2n`, `ironstar-docs`, `ironstar-docs-deps`, `ironstar-eventcatalog`, `ironstar-eventcatalog-deps`, `ironstar-release`, `ironstar-release-c2n`, `playwright-browsers-nixpkgs`, `signoz-backend`, `signoz-frontend`, `signoz-otel-collector`). The task ledger's "33 → 13" estimate undercounted the SigNoz + playwright packages contributed by other modules; the load-bearing fact is the 20-attr deletion, which holds.
- `nix build .#checks.aarch64-darwin.structure-package-set-invariant` → green (exit 0) WITHOUT modifying `modules/checks/package-set-invariant.nix`: the invariant already excludes `*-test`/`*-clippy` suffixes via `isPerCrateSuffix`, so deleting those packages neither adds nor removes a relevant-package-without-check.

### Content-addressed lockfile refinement (cache-name stability)

`importCargoLock`'s `lockFile` is pinned to `builtins.path { path = self + "/Cargo.lock"; name = "ironstar-cargo-lock"; }` rather than `self + "/Cargo.lock"` directly. Reading `self + "/Cargo.lock"` ties the resulting `cargo-vendor-dir` to the whole flake-source identity, so any unrelated commit re-derives the vendor dir (and cascades a full gate recompile); confirmed empirically — editing unrelated files changed the `cargo-vendor-dir.drv` and gate drvPaths. With the content-addressed `builtins.path`, the vendor dir references only `…-ironstar-cargo-lock` (the lockfile content), so it rebuilds only when `Cargo.lock` content changes — honoring the design's content-addressed-name-stability risk mitigation and mirroring crane's `name="ironstar-src"` discipline. A second full gate build after this change reproduced the identical result (`933 passed, 5 skipped`; clippy clean), confirming the change is behaviorally neutral. That rebuild's wall-clock (800s for both gates concurrently) ran higher than the first build (530s clippy / 591s test) due to concurrent nix invocations contending on this aarch64-darwin host; the steady-state per-gate cost from a warm dep store is the ~9-10 min figure, and on warm CI caches the override-touched `-sys` paths substitute and the figure drops further.

## Task 4 — drift detection (CI auto-regen + no-network staleness check)

Host: aarch64-darwin (Darwin 25.2.0 arm64); nixpkgs same channel as prior tasks; crate2nix locked at rev `c994c83` (crate2nix 0.15.0).

### Check count delta: 14 -> 15 (flag for task 5)

The `cargo-nix-lock-sync` check raises the flake check surface from 14 to 15. Tasks-ledger task 5 acceptance and design.md still say "14 checks must stay 14" — that wording was authored before this regulator existed and should read 15 once task 4 lands. Per the dispatch instruction this section flags the discrepancy for the orchestrator rather than editing task 5. Concretely, the devshell closure-class invariant (`devShell.inputsFrom = builtins.attrValues self'.checks`) now spans 15 entries; the added entry is a tiny `coreutils`-only `runCommand` with no compiler closure, so the closure-class impact is negligible. The package-set-invariant is unaffected: `cargo-nix-lock-sync` is a check with no same-named package, so it adds neither a relevant-package-without-check nor a check-without-package mismatch.

### Check derivation shape

`modules/checks/cargo-nix-lock-sync.nix` is a pure, no-network, no-IFD `pkgs.runCommand` (nativeBuildInputs `coreutils` only). Inputs are content-addressed on exactly the two files via `builtins.path` with fixed names (`cargo-nix-lock-sync-cargo-lock`, `cargo-nix-lock-sync-cargo-nix`), the same discipline as `modules/rust.nix`'s `ironstar-cargo-lock` pin, so the check rebuilds only when `Cargo.lock` or `Cargo.nix` content changes and not on unrelated commits.

Parsing is package-scoped, not a raw `version =` line count (the surface-back-trigger #1 hazard a prior worker hit as a 577-vs-576 artifact):

- Cargo.lock carries one pre-package format header `version = 4` at line 3, outside any `[[package]]` block. A naive `rg -c '^version = '` yields 577. The check anchors on `sed -n '/^name = /,$p'`, which begins printing only at the first top-level `name =` line, excluding the header; the remaining `name`/`version` lines strictly alternate (verified: 576 `name =` lines, 0 indented `version =` lines, no dependency-table version fields), paired by `paste - -` into `name@version`.
- Cargo.nix pairs each `crateName = "…";` line with the `version = "…";` line on the immediately-following line (verified: every `version` line is exactly `crateName`+1; all 576 of each are at identical 8-space crate-level indentation; crate2nix dependency entries use `packageId`, never `version`, so no stray version fields interleave).

Both sets are `LC_ALL=C sort -u`'d and compared with `diff -u`; on mismatch the check prints the per-line diff and the actionable line `Run \`just regenerate-cargo-nix\` and commit the updated Cargo.nix.`, then `exit 1`.

### (a) LOCAL green + CCV integrity (red-on-mismatch) — required, DONE

The new check file is untracked, so the dirty git-tree flake source excludes it from `.#checks` (flakes only see tracked/staged files, and git staging is orchestrator-routed in this WO session). Verification was therefore performed against a standalone `nix build --impure --expr` derivation carrying the byte-identical build script, pointed at the real in-tree files (and a `/tmp` perturbed copy for the red case). Once the orchestrator stages/commits the file, the registered `nix build .#checks.aarch64-darwin.cargo-nix-lock-sync` exercises the same logic; that registered-path build is the only step deferred, and it is mechanical (the derivation logic is proven here).

- GREEN (clean tree): standalone derivation against the real `Cargo.lock` + `Cargo.nix` built with exit 0. Output: `Cargo.lock package set: 576 entries`, `Cargo.nix  crate set:  576 entries`, `Cargo.nix is in lockstep with Cargo.lock (576 packages).` The 576==576 reconciliation matches task 1.4's manual `comm` reconciliation.
- RED (deliberate mismatch): a `/tmp` copy of `Cargo.nix` with adler2's version perturbed `2.0.1` -> `2.0.999` (single crate-level `version` line in the copy; in-tree files untouched). The standalone derivation against the perturbed copy FAILED with `builder failed with exit code 1`, printing the precise diff (`-adler2@2.0.1` / `+adler2@2.0.999`) and the actionable `Run \`just regenerate-cargo-nix\` and commit the updated Cargo.nix.` message. This satisfies the CCV integrity obligation: the regulator demonstrably fails on a stale `Cargo.nix`.
- RESTORATION evidence: in-tree `Cargo.nix` sha256 `32b9eb098f00731764ec57f1313a77cf052c9d46d852052f7e49a271f6d94342` identical before and after the demo; `git status --short Cargo.nix Cargo.lock` empty (both byte-identical, untouched). Perturbation lived only in `/tmp/Cargo.nix.perturbed`, removed after.

### (b) LOCAL static — workflow YAML validation, DONE

`nix run nixpkgs#actionlint -- .github/workflows/regenerate-lock-files.yaml` exits 0. The single diagnostic printed is at line 41 (`uses: ./.github/actions/setup-nix`), objecting to the `type:` key on a composite-action input inside `.github/actions/setup-nix/action.yml` — pre-existing, unrelated to this change (the `Setup nix` step is byte-identical to the original; composite-action inputs do not support `type:`, only `workflow_dispatch` inputs do). Surface-back trigger #4 (pre-existing actionlint errors) is flagged in the WO report but does not block; it is the local action's metadata, not an error in this edit.

YAML parse (python `yaml.safe_load`) confirms:
- trigger paths: `bun.lock`, `package.json`, `packages/**/package.json`, `flake.nix`, `Cargo.lock`, `Cargo.toml`, `crates/**/Cargo.toml` (the three Rust paths added; the four pre-existing paths unchanged).
- concurrency group: `regenerate-lock-files-${{ github.ref }}` (generalized from the bun-specific `regenerate-bun-nix-${{ github.ref }}`; the file now regenerates three lock artifacts, so the group name is no longer bun-specific — `cancel-in-progress: true` unchanged).
- step order: Checkout -> Setup nix -> Regenerate flake.lock -> Amend flake.lock -> Regenerate bun.nix -> Amend bun.nix -> Regenerate Cargo.nix -> Amend Cargo.nix. The Cargo.nix regenerate+amend pair mirrors the bun.nix pair byte-for-byte in structure (same `git diff --quiet` guard, same `github-actions[bot]` identity, same `git add`/`commit`/`push`). The flake.lock and bun.nix steps are byte-identical to the original.

The regenerate step is `nix run .#regenerate-cargo-nix`, a new in-repo flake app added to `modules/apps/regenerate.nix` parallel to `regenerate-bun-nix`. It wraps `inputs'.crate2nix.packages.default` (the flake-pinned crate2nix `c994c83`, satisfying the design's `--inputs-from`-pinned-generator requirement) and runs `crate2nix generate` then `treefmt ./Cargo.nix`, producing the same output as the `just regenerate-cargo-nix` recipe. App evals green: `nix eval --raw .#apps.aarch64-darwin.regenerate-cargo-nix.program` resolves to a `regenerate-cargo-nix` wrapper, same shape as the bun app.

GitHub Actions SHA discipline: no new third-party action was introduced. The crate2nix step uses an in-repo flake app (`nix run .#regenerate-cargo-nix`), requiring no action SHA. The `actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6` pin and the local `./.github/actions/setup-nix` reference are reused verbatim from the original file. Surface-back trigger #3 (new third-party action) does not fire; no `gh api` SHA verification was needed.

### (c) DEFERRED — live regenerate-and-auto-commit dry-run

The live workflow dry-run (push a test branch with a trivial `Cargo.lock` change, observe auto-regen of `Cargo.nix` and auto-commit by `github-actions[bot]`) requires a pushed branch, which is user-gated in this WO session (the orchestrator routes commits/pushes). It fires on the eventual CI run after push. The auto-commit mechanism is the same `github-actions[bot]` config + `git add`/`commit`/`push` the flake.lock and bun.nix steps already use in production, so the new step replicates a proven mechanism (surface-back trigger #2, auth/signing the new step cannot replicate, does NOT fire — the new step reuses the existing mechanism verbatim).
