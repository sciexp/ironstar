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
