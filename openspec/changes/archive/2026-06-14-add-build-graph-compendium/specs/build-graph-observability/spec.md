## ADDED Requirements

### Requirement: build-graph compendium is an on-demand, non-gating instrument

The build-graph compendium SHALL be an on-demand observability instrument rather than a CI gate.
It MUST emit a strictly byte-deterministic committed snapshot, a queryable DuckDB dataset whose duplication fidelity reproduces the committed snapshot, and curated renderings plus a review runbook.
It MUST NOT contribute any flake check to the check surface, and no CI workflow MUST invoke it; the compendium never turns CI red and never pushes build-graph artifacts from CI.
Every artifact and the generated index MUST state which edge semantics it carries — the `inputs.drvs` build-closure projection (carried by `edges.ndjson` and the DuckDB `edges` table) or the `env.dependencies`/`buildDependencies` crate-DAG parse (carried by the member-dag and crate-overview renderings) — and no count reconciliation between the two regimes is expected.

#### Scenario: snapshot is byte-deterministic over an unchanged substrate

- **WHEN** the snapshot app is run twice over an unchanged build substrate
- **THEN** both runs produce byte-identical output equal to the committed `snapshot.json`, because the snapshot has sorted keys, no store hashes, and no timestamps

#### Scenario: the compendium is non-gating and absent from CI

- **WHEN** `nix eval .#checks.<system> --apply builtins.attrNames` is evaluated and every CI workflow is inspected
- **THEN** the check surface contains no build-graph check and no CI workflow invokes the compendium, so build-graph data can never turn CI red

#### Scenario: the DuckDB dataset reproduces the snapshot's duplication value

- **WHEN** the DuckDB duplication query runs against `logs/build-graph/build-graph.duckdb`
- **THEN** it reproduces the committed snapshot's `dev_profile_crates_with_multiplicity` value (202 in the realized snapshot), so the dataset cannot silently diverge from the snapshot it is loaded from
