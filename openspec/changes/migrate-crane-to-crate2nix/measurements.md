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
