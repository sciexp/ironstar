---
title: Build-graph review-session runbook
---

This runbook governs the periodic human review of ironstar's Nix build graph.
It exists because the build graph is no longer gated by an automated check: the former graph-drift regulator was retired with the crate2nix migration, and its place is taken by a deliberate, infrequent review session driven by the tooling described here.
The deterministic review surface is the committed snapshot diff; the queryable and visual substrate is the on-demand compendium.
Nothing in this workflow runs per pull request, and nothing here imposes a failure condition.

## When to run

Run a build-graph review session when the workspace's dependency structure changes in a way that could shift compile multiplicity, profile splits, or vendor monoliths.
The triggering events are a dependency add or removal, or a workspace rearchitecting such as splitting or merging a member crate.
This is always an explicit human decision, never an automated gate and never a per-PR step.

The review session has three steps.
First, regenerate the committed snapshot with `just regenerate-build-graph-snapshot`.
Second, inspect the change with `git diff -- modules/apps/build-graph-snapshot/snapshot.json`; this diff is the review surface (see the next section).
Third, build the queryable and visual compendium with `just build-graph-report` and work through the queries and checklist below.

## Diff substrate

The snapshot diff is the deterministic review surface because the snapshot is a single committed scalar envelope projected from a hash-free six-tuple node identity, so an unchanged substrate produces a byte-identical file.
Within the snapshot's `provenance` block, `root_drv_paths_sha256` is the precise substrate fingerprint: it hashes the realized root derivation paths, so a changed hash means the realized substrate moved and the rest of the diff is worth reading closely.
The sibling `cargo_nix_sha256` is only a human-meaningful pointer to one input, the generated `Cargo.nix`; do not read a stable `cargo_nix_sha256` as evidence that a change is generator-only, because the realized substrate can move while that one input's hash holds.

When the substrate fingerprint changes and you want the edge-level delta rather than the scalar delta, regenerate `logs/build-graph/edges.ndjson` across both substrates (the old and new states) and diff the two NDJSON files.
For a visual delta, regenerate the renderings and compare `logs/build-graph/graph-viz/index.md` and its linked artifacts across the two states.

## The four canned DuckDB queries

The compendium loads `logs/build-graph/build-graph.duckdb` with `nodes`, `edges`, and `root_membership`, and the canned review queries live in `modules/apps/build-graph-compendium/queries.sql`.
Run them all against the loaded database with:

```bash
duckdb logs/build-graph/build-graph.duckdb < modules/apps/build-graph-compendium/queries.sql
```

The four queries answer the questions the scalar snapshot cannot.
Q1 (duplication by crate) lists every dev-profile crate whose distinct store-hash compile count exceeds one, and its summary scalar reproduces the snapshot's `dev_profile_crates_with_multiplicity`.
Q2 (heavy-crate ego neighborhoods) returns the one-hop in-edges and out-edges of each heavy-crate seed across all roots, separating build dependencies (`direction='out'`) from consumers (`direction='in'`).
Q3 (dev/release divergence) is a full anti-join over `(pname, version)` crate compiles, naming which profile is missing for each crate present in only one profile.
Q4 (member condensation) contracts the graph to member scope and counts the shared dependency nodes reached by more than one member's test closure, measuring the cross-member shared-dependency surface.

## Optimization-gap checklist

Work through these four items against the query output and the snapshot diff; each names a class of avoidable build cost.
Feature-unification spread: look for crate multiplicity in Q1 that is not explained by genuine version skew, since spread without distinct versions points at feature-flag fragmentation that unification could collapse.
Profile splits: from Q3, ask whether each dev, release, and test split is load-bearing, or whether a split exists only because a profile setting diverges without a reason that matters here.
Vendor monoliths: track the vendor-monolith count trend across review sessions, since a rising count signals dependency bloat accreting into the vendored tree.
New heavy-crate multiplicities: compare fresh heavy-crate distinct-compile counts against `reference/ceilings.json`, flagging any crate whose multiplicity has grown past its retired baseline value for explicit human attention.

## Reference data

The retired baseline ceilings live at `modules/apps/build-graph-compendium/reference/ceilings.json`.
They are orientation-only: no check consumes the file and the compendium never gates on it, so they are never a failure condition.
Read them as a comparison anchor when judging whether a fresh multiplicity or node count has drifted from the crate2nix baseline-zero topology.

## Edge-semantics note

Two incommensurable edge semantics appear in the compendium, and conflating them produces nonsense comparisons.
The NDJSON edge list and the DuckDB `edges` table carry the `inputs.drvs` build-closure projection: an edge means one derivation is a build input of another.
The member-dag and crate-overview renderings carry the `env.dependencies` crate DAG parse: an edge means one crate declares a dependency on another in the resolved crate graph.
These are different graphs over different node populations; do not diff one against the other or read a count from one as if it constrained the other.

## Manual nixgraph escape hatch

For an ad-hoc provenance query outside the canned compendium, sbomnix's `nixgraph` is the off-the-shelf alternative, but it is locked to `dot -Kdot` hierarchical layout and degrades on dense scopes, so a shallow inverse query is the only safe default:

```bash
DRV=$(nix eval --raw .#packages.x86_64-linux.ironstar-release.drvPath)
nix shell nixpkgs#sbomnix nixpkgs#graphviz -c \
  nixgraph --buildtime --inverse 'rust_zenoh' --depth 4 --colorize 'rust_ironstar' --out nixgraph-zenoh-inverse.png "$DRV"
```

Never run `nixgraph --buildtime` at depth >= 2 over the full closure, and never run `--until` over the full closure: the hierarchical `dot` render produces a non-terminating job or a multi-megabyte max-height ribbon with zero legible nodes.
