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

### Phase 3: cd.yaml jobs to herculesCI effects (2026-05-07)

Superseded by herculesCI effects (`deploy-sites`, `release-packages`) running on buildbot-nix.

- `cd.yaml` -- pre-effects-retirement snapshot of the CD workflow. The `preview-release-version` job (obh.34) was the first to be retired, with its functionality migrated to `apps.preview-version` invoked from the `release-packages` herculesCI effect on non-main branches (obh.16).

### Phase 4: deploy/release workflows and cd demotion (2026-05-09)

Superseded by herculesCI effects fully owning preview and production CD; the live `cd.yaml` demoted to a manual `workflow_dispatch` shell matching the vanixiets terminal shape.

- `deploy-site.yaml` -- reusable site-deploy workflow, superseded by the `deploy-sites` herculesCI effect at `modules/effects/herculesCI/deploy-sites.nix`.
- `package-release.yaml` -- reusable package-release workflow, superseded by the `release-packages` herculesCI effect at `modules/effects/herculesCI/release-packages.nix`.

The live `cd.yaml` was rewritten to a `workflow_dispatch`-only shape carrying just `set-variables` and `bootstrap-verification`, mirroring the vanixiets template's terminal CD form. The prior cd.yaml shape (with `preview-site-deploy`, `production-release-packages`, `production-site-deploy`, and `release-build` jobs) is preserved in the existing Phase 3 snapshot at `.github/deprecated/cd.yaml` and remains the recovery source for any of those jobs.

The `release-build` job (a placeholder for future Rust binary releases gated on `vars.ENABLE_RUST_RELEASE`) was archived inline by removal from the live `cd.yaml`. It is recoverable from the Phase 3 snapshot when Rust binary release work begins.

This phase completes the buildbot-nix CI/CD migration retain-list reduction, closing obh.30 and unblocking obh.25 (production-deploy verification) on the next merge to main.

## Restoration

To restore any file, move it back to its original location:

- `ci.yaml`, `ci-nix-fast-build.yaml`, `package-test.yaml`, `cd.yaml`, `deploy-site.yaml`, and `package-release.yaml` to `.github/workflows/`
- `ci-build-category.sh` to `scripts/ci/`

## Archive date

Phase 1 archived 2026-04-05 on the `ironstar-ohy-nix-ci-consolidation` bookmark as part of the nix CI consolidation epic (ohy).
Phase 2 archived 2026-04-09 on the same bookmark.
Phase 3 archived 2026-05-07 on the `buildbot-nix-migration` bookmark as part of the buildbot-nix CI/CD migration epic (obh).
Phase 4 archived 2026-05-09 on the same `buildbot-nix-migration` bookmark, completing the obh retain-list reduction.
