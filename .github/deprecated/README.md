# Deprecated CI/CD artifacts

Archived CI/CD artifacts superseded by the nix-first pipeline (`ci.yaml` using `nix-fast-build`).

## Archived files

- `ci.yaml` -- original multi-job CI/CD pipeline with 12 imperative jobs, replaced by a single `nix-fast-build` check job that evaluates all validation as `.#checks` attributes.
- `package-test.yaml` -- per-package JS/TS test workflow (reusable), superseded by `docs-unit`, `docs-e2e`, `eventcatalog-unit`, and `eventcatalog-e2e` flake check derivations.
- `ci-build-category.sh` -- CI build category dispatcher script (from `scripts/ci/`), superseded by `nix-fast-build` evaluating `.#checks` directly.

## Restoration

To restore any file, move it back to its original location:

- `ci.yaml` and `package-test.yaml` to `.github/workflows/`
- `ci-build-category.sh` to `scripts/ci/`

## Archive date

Archived 2026-04-05 on the `ironstar-ohy-nix-ci-consolidation` bookmark as part of the nix CI consolidation epic (ohy).
