# Deprecated CI/CD artifacts

Archived CI/CD artifacts from successive pipeline consolidation phases.

## Archived files

### Phase 1: imperative CI to nix-fast-build (2026-04-05)

Superseded by the nix-first pipeline (`ci.yaml` using `nix-fast-build`).

- `ci.yaml` -- original multi-job CI/CD pipeline with 12 imperative jobs, replaced by a single `nix-fast-build` check job that evaluates all validation as `.#checks` attributes.
- `package-test.yaml` -- per-package JS/TS test workflow (reusable), superseded by `docs-unit`, `docs-e2e`, `eventcatalog-unit`, and `eventcatalog-e2e` flake check derivations.
- `ci-build-category.sh` -- CI build category dispatcher script (from `scripts/ci/`), superseded by `nix-fast-build` evaluating `.#checks` directly.

### Phase 2: nix-fast-build to buildbot (2026-04-09)

Superseded by buildbot-nix CI with `nix-eval` and `nix-build` check reporting.

- `ci-nix-fast-build.yaml` -- CI/CD workflow with `nix-fast-build` check job, replaced by buildbot-nix evaluation and build checks. The remaining CD jobs (deploy, release) were extracted into `cd.yaml`.

## Restoration

To restore any file, move it back to its original location:

- `ci.yaml`, `ci-nix-fast-build.yaml`, and `package-test.yaml` to `.github/workflows/`
- `ci-build-category.sh` to `scripts/ci/`

## Archive date

Phase 1 archived 2026-04-05 on the `ironstar-ohy-nix-ci-consolidation` bookmark as part of the nix CI consolidation epic (ohy).
Phase 2 archived 2026-04-09 on the same bookmark.
