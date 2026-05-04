---
title: Buildbot-nix migration brief
---

Per-repo migration brief covering ironstar's continued participation in the GHA→buildbot-nix CI and herculesCI-effects CD program.
This brief is the authoritative scoping document for the corresponding beads epic; the epic plan derives from it during `/session-plan`.
The skeleton this brief follows lives at `~/projects/sciexp/planning/docs/notes/orientation/ci-cd-migration-briefing-skeleton.md` and the eight-component refactoring contract it draws on lives at `~/projects/sciexp/planning/docs/notes/orientation/ci-cd-refactoring-pattern-reference.md`.

## 1. Repo summary

Ironstar is the Rust + Datastar template at `~/projects/rust-workspace/ironstar/` (`github.com/sciexp/ironstar`, default branch `main`).
The Cargo workspace contains 11 crates plus a `packages/` map of website packages (`packages/docs` Astro Starlight, `packages/eventcatalog` EventCatalog).
Beads is initialized; the program-wide migration matrix at `~/projects/sciexp/planning/docs/notes/orientation/ci-cd-migration-matrix.md` records ironstar as the reference implementation for Phase 1 (CI) and as the canonical Phase 2 pattern crystallizing here.

CI shape (post-`ohy`, merged to main on 2026-04-09 via PR #386): buildbot-nix owns CI evaluation and build via the 14 flake checks enumerated in the matrix (`treefmt`, `gitleaks`, `nix-unit`, `workspace-clippy`, `workspace-test`, `ironstar`, `dev-platform`, `ironstar-docs`, `ironstar-docs-unit`, `ironstar-docs-e2e`, `ironstar-eventcatalog`, `ironstar-eventcatalog-unit`, `ironstar-eventcatalog-e2e`, `ironstar-e2e`).
`buildbot-nix.toml` already declares `attribute = "checks.x86_64-linux"` per the magnetite fleet constraint.
`flake.nix` already wires `cache.scientistexperience.net?priority=45` into `nixConfig.extra-substituters` with the trusted public key `cache.scientistexperience.net-1:N9ZeWasooJLXEwaN+rd4MMyBuGpAtcUAXrEUPBT5cXI=`, plus `sciexp.cachix.org?priority=50`, `nix-community.cachix.org`, and `crane.cachix.org`.
The `.github/deprecated/` archive narrates the imperative-CI→`nix-fast-build` and `nix-fast-build`→buildbot-nix archival phases.
The justfile already exposes `nix-flake-io` (line 1072) and `check-fast` (line 985) for local fan-out.

CD shape: GHA `cd.yaml` orchestrates `set-variables`, `bootstrap-verification`, `preview-release-version` (semantic-release dry-run, matrix over `packages/*`), `preview-site-deploy` (calls reusable `deploy-site.yaml` per package), `production-release-packages` (calls `package-release.yaml` per package), `production-site-deploy` (calls `deploy-site.yaml` per package), and `release-build` (Rust binary build, gated `vars.ENABLE_RUST_RELEASE == 'true'`, currently inert).
Reusable workflows: `deploy-site.yaml` (Cloudflare Pages deploy via wrangler), `package-release.yaml` (semantic-release per package), `release.yaml` (disabled-by-default Rust binary release, `on:` block commented out).
Other workflows: `pr-check.yaml`, `pr-merge.yaml`, `update-flake-inputs.yaml`, `labeler.yml`.
There is no `apps.<system>.<name>` declaration in the flake (`nix-flake-io` reports the apps section empty), no herculesCI attribute, and no effects modules — Phase 2 introduces all three.

The brief drafts off main and ignores the open observability migration PR #388 (branch `ejp-observability-migration`, ~3 weeks idle).
Observability rebases onto migrated main when both land.
The migration epic delivers as 1 PR per repo: brief commit and all subsequent implementation commits stack on the same `buildbot-nix-migration` branch with atomic-commit-per-file discipline preserving bisect granularity at fast-forward merge time.
A separate cross-repo coordination PR lands in vanixiets for the per-consumer secrets generator (2 PRs total per target repo: 1 in ironstar, 1 in vanixiets).

The migration draws from two pattern sources, each addressing a distinct concern.
**Substrate** is verbatim reuse from vanixiets at `~/projects/nix-workspace/vanixiets/`: bun2nix derivations referencing root `bun.lock` and `bun.nix`, `mkEffect` wrappers, secrets generator pattern, `HERCULES_CI_SECRETS_JSON` envelope, bwrap-sandbox workarounds.
Vanixiets's N=1 case validates the substrate even though it does not validate topology choices.
**Topology** is the forward extension matching ironstar's existing GHA matrix: eval-time effect-fanout via `builtins.readDir ./packages` + `lib.listToAttrs` generating per-package herculesCI effect attributes at the effects layer, while underlying flake apps remain single shared parametric apps.
This is the only genuinely novel work; the GHA `cd.yaml` already iterates over `packages/*` via matrix, and the fanout pattern is the topology-preserving mapping onto herculesCI effects.

The bun + bun2nix substrate landing in this migration reconciles CLAUDE.md framing (the ny3 epic's documented pnpm decision, recorded at `/Users/crs58/projects/sciexp/planning/contexts/ironstar.md:136`) with working-tree reality (`packageManager: "bun@1.3.4"` in the root `package.json`, root `bun.lock` of 313KB, root `bun.nix` of 270KB) and adds the nix-driven derivation pattern.
Reconciliation is more accurate than supersession — the ny3 decision was documented but never implemented in the working tree.

## 2. Phase 1 acceptance criteria

Phase 1 is substantially complete in ironstar (see section 1).
The remaining Phase 1 acceptance criteria close the gaps that the program-wide skeleton names and that the post-`ohy` state has not yet picked up.

- **`nix-flake-io` baseline captured.** Run `just nix-flake-io` and commit the captured output to `logs/ironstar-nix-flake-io-baseline.log`. The `logs/` directory is gitignored; the baseline log is committed explicitly via `git add -f` (or jj equivalent) so the audit artifact survives. The captured output enumerates 14 checks, 31 packages, 1 devShell, 0 apps, 0 overlays, 0 nixosModules, 16 inputs, and the `treefmt` formatter — confirming no `apps.<system>.<name>` declarations exist (a Phase 2 precondition).
- **Workflow three-bucket categorization documented in epic description.** Use vanixiets's retained set as the parameterized reference. Categorization for ironstar, derived from the survey:

  | Workflow | Bucket | Rationale |
  |---|---|---|
  | `cd.yaml` | retain (workflow_dispatch-only after Phase 2 archival) | Vanixiets retains its `cd.yaml` as a workflow_dispatch-only manual orchestrator. After Phase 2 archives the deploy and release jobs, `cd.yaml` retains only the orchestration scaffolding plus `release-build` (deferred per Q4(a) carve-out) and `bootstrap-verification` (no flake-check substitute exists; remains a GHA job). |
  | `deploy-site.yaml` | migrate (Phase 2) | Reusable Cloudflare deploy invoked from `cd.yaml`. Replaced by the `deploy-sites` herculesCI effect parametric over `packages/*`. |
  | `package-release.yaml` | migrate (Phase 2) | Reusable semantic-release invoked from `cd.yaml`. Replaced by the `release-packages` herculesCI effect parametric over `packages/*`. |
  | `release.yaml` | retain (deferred) | Disabled-by-default Rust binary release (Q4(a) carve-out). Stays in `.github/workflows/` until packaging strategy stabilizes. |
  | `update-flake-inputs.yaml` | retain | Mirrors vanixiets's retained pattern (Q4(b)). |
  | `pr-check.yaml` | retain | Generally-applicable retention from the skeleton's parameterized reference set. |
  | `pr-merge.yaml` | retain | Generally-applicable retention. |
  | `labeler.yml` | retain | Issue-labeling workflow with no flake-check or effect equivalent. Mirrors the vanixiets-sibling pattern recorded in the matrix. |

- **Mergify alignment.** `.github/mergify.yml` already exists at the correct path. Update the `required_checks` block to add `check-success=buildbot/nix-effects` alongside the existing `buildbot/nix-eval` and `buildbot/nix-build` so the rolled-up effects status gates merges once Phase 2 lands. Existing `set-variables`, `check-fast-forward`, and `bootstrap-verification` checks remain because the corresponding GHA jobs survive Phase 2 archival. Bot account names and human-author auto-approve (currently commented out, optional) reviewed for alignment with vanixiets's pattern: bot conditions reference `sciexp-flake-updater[bot]` (correct for the `sciexp/` org); the optional auto-approve-bot-PRs rule remains commented out unless AC chooses to enable it.
- **Flake check coverage.** Already complete per the matrix: 14 checks span all 8 categories where applicable to ironstar (format, secrets-scan, lint, unit-test, integration/e2e, nix-infrastructure, build/eval; type-check is N/A for the Rust workspace). No new check derivations are required for Phase 1 close-out.
- **`buildbot-nix.toml` declares `attribute = "checks.x86_64-linux"`.** Already complete (the file is 6 lines, header comment plus the attribute declaration). Verify the file remains untouched through Phase 2; consider whether to add `effects_branches = ["*"]` and `effects_on_pull_requests = true` (matching vanixiets) at the Phase 2 boundary so PR effect dispatch is enabled — this is a Phase 2 concern and is captured there.
- **`cache.scientistexperience.net` substituter wiring.** Already complete (`flake.nix:67-79` declares all four substituters and trusted public keys).
- **nix-fast-build local fan-out via justfile.** Already complete (`check-fast` at justfile line 985). Verify the recipe still drives the post-Phase-2 check set without modification.
- **Predecessor CI archival narrative.** Already complete (`.github/deprecated/README.md` narrates Phase 1 imperative-CI→nix-fast-build on 2026-04-05 and Phase 2 nix-fast-build→buildbot on 2026-04-09). The Phase 2-of-this-epic CD-to-effects archival extends this narrative as a third documented phase at the corresponding milestone.

#### Bun + bun2nix migration sub-track

This sub-track wires the bun2nix substrate into ironstar's existing root `bun.lock` + bun workspaces layout (vanixiets-aligned, ironstar-already-aligned). No per-package lockfile work — the root lockfile stays root. The dependency chain is encoded so `/session-plan` generates issues in the correct order.

a. **Verify `bun2nix` flake input.** Already present at `flake.nix:32-33` (verified during briefing survey). This AC item is a verification (no-op build), not an addition. It still gates derivation migrations because the implementing WO must confirm the input remains in place and is reachable from the per-package derivations before proceeding.
b. **Migrate `packages/docs` derivation to bun + bun2nix.** Mirror vanixiets's `vanixiets-docs` substrate pattern. Reference paths: `~/projects/nix-workspace/vanixiets/pkgs/by-name/vanixiets-docs/package.nix` (main derivation including `dontUseBunBuild = true; dontUseBunInstall = true` at lines 113-114, the Typst diagram-compilation buildPhase at lines 121-148, and the bundled-workerd autoPatchelf at lines 73-80) and `~/projects/nix-workspace/vanixiets/pkgs/by-name/vanixiets-docs-deps/package.nix` (the hermetic node_modules tree derivation). The per-package derivation references the root `bun.lock` and root `bun.nix` via `../../../bun.lock` and `../../../bun.nix` paths. NO per-package lockfile work — root stays root.
c. **Research issue for `packages/eventcatalog`.** Scope per D4: "which `nativeBuildInputs` and pre-build commands does eventcatalog's catalog generation need + does it require workerd-equivalent binary patching?" Anchor on vanixiets-docs's Typst + `dontUseBunBuild` + `dontUseBunInstall` precedent at `~/projects/nix-workspace/vanixiets/pkgs/by-name/vanixiets-docs/package.nix:113-114,121-148`. Deliverable: build/test/deploy plan covering verbatim transfers (substrate elements that translate as-is from vanixiets-docs), adaptations (eventcatalog catalog-generation specifics), and novel work (e.g., EventCatalog-specific binary patching needs, if any).
d. **Migrate `packages/eventcatalog` derivation per the research-issue plan.** Gated on (c) completion. Same root-`bun.lock` / root-`bun.nix` reference pattern as `packages/docs` per (b). Eventcatalog-specific buildPhase pre-build commands per (c)'s plan.
e. **Migrate justfile recipes** from `pnpm-*` to `bun-*`. Parametric over `packages/*`. Can run in parallel with derivation migrations (d) once the bun2nix flake input verification (a) is complete.
f. **Reconcile CLAUDE.md framing with working-tree reality.** Edit target is `/Users/crs58/projects/sciexp/planning/contexts/ironstar.md` — the symlink-resolved file from `/Users/crs58/projects/rust-workspace/ironstar/CLAUDE.md`. The framing edit reframes the ny3 epic's documented pnpm decision (currently at line 136 of the symlink target as a single reference; verified during briefing survey) as documentation-reality reconciliation, NOT substrate change. The working tree is already on bun (`packageManager: "bun@1.3.4"`); the CLAUDE.md update aligns documentation with reality and records the bun2nix-driven derivation pattern as the new substrate addition.

The six-of-fourteen flake-checks affected (`ironstar-docs`, `ironstar-docs-unit`, `ironstar-docs-e2e`, `ironstar-eventcatalog`, `ironstar-eventcatalog-unit`, `ironstar-eventcatalog-e2e`) update naturally as the derivations migrate. The test pattern follows vanixiets's `passthru.tests.{unit,linkcheck,e2e}` block at `~/projects/nix-workspace/vanixiets/pkgs/by-name/vanixiets-docs/package.nix:167-263` rather than a separate `modules/checks/<pkg>.nix` file.

## 3. Phase 2 acceptance criteria

Phase 2 introduces flake apps as the herculesCI-effect substrate, the effects modules themselves, the per-consumer secrets generator in vanixiets, and the corresponding archival of `deploy-site.yaml` and `package-release.yaml`.

The five user-confirmed scope decisions from the dispatch prompt (Q1–Q5, restated in section 7a) are encoded as facts here.
The override clause in section 7a applies: the `/session-plan` WO may revise these only if material new information surfaces during planning, and must surface the contradiction to AC for re-confirmation rather than silently revising.

### Substrate: flake apps

- **`hercules-ci-effects` added as a flake input** in `flake.nix`, with `inputs.flake-parts.follows`, `inputs.nixpkgs.follows`, and `inputs.nixpkgs-stable.follows` declared per vanixiets's pattern. The `flake-module.nix` import lives at `modules/effects/flake-module.nix` (matching vanixiets's `modules/effects/flake-module.nix`) and is picked up automatically by the `import-tree ./modules` mechanism already present in `flake.nix:62`.
- **Flake apps via `writeShellApplication`** declared under `perSystem.apps.<name>` in per-app modules, exposed at `apps.<system>.<name>` for `nix run .#<name>` invocation and for `config.apps.<name>.program` resolution in the effects modules. Two apps required by the Q4(b) first-pass scope: `deploy-sites` (parametric over `packages/*`) and `release-packages` (parametric over `packages/*`). Factoring detail (thin per-package + iterating effects vs fat parametric apps + thin effects, plus utility apps that need no parametricity such as `list-packages-json` from vanixiets) is deferred to `/session-plan` per Q1, Q5.
- **Hermetic `runtimeInputs` and `runtimeEnv`** declarations on each `writeShellApplication`. Minimum `runtimeInputs` for the deploy/release apps include `nodejs` (for wrangler / semantic-release), `bun` (vanixiets-aligned, ironstar-already-aligned per the AC-ratified bun + bun2nix substrate decision in section 7a; npm-fallback available inside the nix sandbox where bun is incompatible with specific tooling such as semantic-release plugins or custom build steps), `jq`, `coreutils`, `git`, `gnugrep`, `gnused`, `gawk`, `findutils` — sed/awk/grep/find are explicit because the bwrap sandbox PATH does not include them by default (see bwrap pitfalls below).
- **Eval-time store-path injection** for the website artifacts: the `deploy-sites` app's `runtimeEnv` (or its equivalent payload variable) points at `${config.packages.ironstar-docs}` and `${config.packages.ironstar-eventcatalog}` so the bwrap sandbox can resolve the built sites without a working-tree bind. Mirrors vanixiets's `DOCS_PAYLOAD` injection at `modules/apps/docs/deploy.nix:26-58`.
- **Sidecar bash scripts co-located with the flake-app wrapper** at `modules/apps/<group>/<name>.sh`, paired one-to-one with `<name>.nix`. Group names align with the app names: `modules/apps/deploy-sites/deploy-sites.{nix,sh}`, `modules/apps/release-packages/release-packages.{nix,sh}`. (Alternative grouping deferred to `/session-plan`.)
- **Justfile recipes transitioned** from inline `nix develop -c just X → script` to `nix run --accept-flake-config .#<flake-app>` for the migrate-bucket recipes (`docs-deploy-{preview,production}`, `eventcatalog-deploy-production`, `cf-build-deploy`, `cf-versions-deploy` — per the matrix's pre-transition enumeration at lines 459–858 of the current justfile). Pre-transition recipes left in place where they target ad-hoc local invocation; the post-transition `nix run` form is the CD path.

### Effects: modules in ironstar's tree

Effects modules live in ironstar's own tree at `modules/effects/herculesCI/<effect>.nix`, NOT in vanixiets — only the per-consumer secrets generator is aggregated under vanixiets.

- **`modules/effects/herculesCI/deploy-sites.nix`** declares per-package effect attributes (`effects.deploy-${pkgName}`) generated at flake-eval time via `builtins.readDir ./packages` + `lib.listToAttrs`. Each per-package attribute imports `config.apps.deploy-sites.program` for eval-time store-path resolution and invokes it with `pkgName` as argv. Per-package effects run in their own bwrap sandboxes in parallel. Branch dispatcher per attribute: `production` on `isMain`, `preview <branch>` otherwise. Substrate exemplar: `~/projects/nix-workspace/vanixiets/modules/effects/vanixiets/herculesCI/deploy-docs.nix` (the topology generalizes vanixiets's hardcoded-single deploy to per-package fanout; the substrate is verbatim reuse).
- **`modules/effects/herculesCI/release-packages.nix`** declares per-package effect attributes (`effects.release-${pkgName}`) via the same `builtins.readDir ./packages` + `lib.listToAttrs` fanout mechanism. Each per-package attribute invokes `config.apps.release-packages.program` with `pkgName` as argv and branches internally on `isMain` per the dry-run rehearsal contract in the buildbot-nix.toml Phase 2 boundary subsection (production semantic-release with `@semantic-release/github` enabled on main; dry-run with `@semantic-release/github` filtered out via `preview-version.sh main <pkg-path>` on PR pushes and other non-main branches). Substrate exemplar: `~/projects/nix-workspace/vanixiets/modules/effects/vanixiets/herculesCI/release-packages.nix` (the topology generalizes vanixiets's loop-internal release to per-package fanout; the substrate is verbatim reuse). The eventual `release-packages-dry-run` rehearsal attribute may be added in a follow-on issue, mirroring vanixiets's `mkReleasePackagesEffect { dryRun = true; }` pattern.

### (C) Effect-fanout topology

This is the canonical pattern for both release and deploy effects. The two layers are kept distinct.

**Layering.** Per-package fanout happens at the **effects layer** (per-package herculesCI effect attributes generated at flake-eval time). The underlying **apps layer** uses single shared parametric flake apps (`apps.deploy-sites`, `apps.release-packages`) taking `pkgName` as argv, mirroring vanixiets's `apps.release` convention at `~/projects/nix-workspace/vanixiets/modules/apps/release/release.nix` (single shared `apps.release` flake app that takes a package path as argv; not per-package release apps). This keeps the apps surface minimal while enabling per-package parallelism at the effects layer.

Both `release-*` and `deploy-*` effect attributes are generated at flake-eval time via `builtins.readDir ./packages` + `lib.listToAttrs`, producing per-package effect attributes (`effects.deploy-${pkgName}`, `effects.release-${pkgName}`). Each runs in its own bwrap sandbox in parallel. The substrate (bun2nix derivations referencing root `bun.lock`, `mkEffect` wrappers, secrets handling, sandbox workarounds, vanixiets's per-package `release.sh <pkg-path>` and `preview-version.sh main <pkg-path>` scripts) is verbatim vanixiets reuse. The topology generalizes vanixiets's loop-internal release + hardcoded-single deploy to uniform fanout for both, preserving ironstar's existing GHA `cd.yaml` matrix topology.

Concrete sketch (illustrative; concrete factoring deferred to `/session-plan`):

```nix
let
  packageNames = lib.attrNames (lib.filterAttrs
    (n: t: t == "directory")
    (builtins.readDir ../../../packages));
in
herculesCI.onPush.default.outputs.effects = lib.listToAttrs (map
  (pkgName: {
    name = "deploy-${pkgName}";  # or "release-${pkgName}"
    value = withSystem "x86_64-linux" ({ config, pkgs, ... }:
      hci-effects.mkEffect {
        name = "deploy-${pkgName}";
        inputs = [ pkgs.git ];
        effectScript = ''
          # invokes config.apps.deploy-sites.program with pkgName as argv
        '';
      });
  })
  packageNames);
```

### Bwrap-sandbox pitfalls (each a discrete acceptance criterion)

The hercules-ci-effects bwrap sandbox imposes constraints that vanixiets's effects modules retrofit. Each constraint is a discrete acceptance criterion in this epic so `/session-plan` generates explicit issues rather than rediscovering at PR-1 time.

- **`oauth2:` username prefix on the remote URL for fine-grained-PAT git auth.** The semantic-release authentication path in `release-packages.nix:179` exports `GIT_CREDENTIALS="oauth2:${GITHUB_TOKEN}"` because GitHub routes fine-grained PATs (`github_pat_*`) into the wrong credential validator under the `x-access-token` username (which is reserved for GitHub App installation tokens `ghs_*`). Ironstar's `release-packages` effect must apply the same prefix.
- **`pkgs.git` in `mkEffect.inputs`.** The `mkEffect` defaultInputs do not include git; any effect that performs a git clone (release-packages's clone preamble) must declare `inputs = [ pkgs.git ]` explicitly. Vanixiets's `release-packages.nix:94` is the exemplar.
- **Env-first GIT_* contract.** Set `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`, `GIT_COMMITTER_NAME`, `GIT_COMMITTER_EMAIL` explicitly in the effect script because the bwrap sandbox does not bind the working tree's `.git` directory and the `/nix/store` ro-bind would block `.git/config` writes. Vanixiets's `release-packages.nix:185-188` is the exemplar.
- **Null-guards on `repo.{branch,rev}` for tag pushes.** Tag-triggered effects have null `branch` (and may have null `rev` shape variations); guard with `if branch == null then null else ...` for builtins.match calls and `if branch == null then "" else toString branch` for shell interpolation. Vanixiets's `release-packages.nix:21` and `:123` exhibit the pattern. Ironstar's `release-packages` effect needs the same guards for the `refs/pull/<N>/merge` synthetic ref handling.
- **Eval-time store-path resolution for flake-app program references.** Use `config.apps.<name>.program` resolved at eval time, NOT `nix run .#<name>` at runtime, because the bwrap sandbox does not bind the working tree and `.#` cannot resolve. Vanixiets's `deploy-docs.nix:26,70` and `release-packages.nix:48-50,191-193` are the exemplars. Both ironstar effects must follow this pattern.
- **Synthetic `refs/pull/<N>/merge` ref handling for PR-merge dispatch.** The `release-packages` effect must handle `refs/pull/<N>/merge` synthetic refs for PR-merge dispatch by fetching `+refs/pull/<N>/head:refs/remotes/origin/pr-<N>-head` explicitly, because the T0 buildbot-eval SHA drifts from the T1 runtime content for `merge` refs. Vanixiets's `release-packages.nix:124-155` is the exemplar.
- **Avoid `config.repo.remoteHttpUrl` in clone URLs.** Buildbot-nix bakes the App installation token into `remoteHttpUrl`; using it in echo-able shell strings would leak the token via banner echo. Hard-code `https://github.com/sciexp/ironstar.git` instead. Vanixiets's `release-packages.nix:114-116` makes the contract explicit.
- **`whoami` and `hostname` not on bwrap PATH.** Hard-code `DEPLOY_DEPLOYER` and `DEPLOY_HOST` literals in the deploy-sites effect script. Vanixiets's `deploy-docs.nix:56-58` is the exemplar.
- **Stale-rev guard for non-PR-merge production runs.** The `release-packages` production path (non-PR-merge, non-dry-run) must verify `git rev-parse HEAD == git rev-parse origin/$GIT_BRANCH` after fetching the branch, because buildbot-nix-evaluated revs may have drifted by effect execution time. Vanixiets's `release-packages.nix:74-88` is the exemplar.

### Cross-repo: vanixiets coordination PR

Triggered (not pre-emptive) once the ironstar effects modules are concrete enough that the secrets list is known. Delivered as a separate PR in vanixiets, distinct from the in-ironstar migration PR. Total per target repo: 2 PRs (1 in ironstar, 1 in vanixiets).

- **`modules/effects/ironstar/secrets.nix` in vanixiets** declaring `clan.core.vars.generators.ironstar-effects-secrets`, exported as `flake.modules.nixos.effects-ironstar-secrets`. Mirrors `~/projects/nix-workspace/vanixiets/modules/effects/vanixiets/secrets.nix` shape: `restartUnits = [ "buildbot-master.service" ]` for systemd LoadCredential restart-on-secret-change; per-component `files.<prompt>.deploy = false` so only the composed JSON `secrets` file deploys to the buildbot-nix host; `runtimeInputs = [ pkgs.jq ]` for the composition `script`.
- **Secrets list (initial).** The first-pass Phase 2 effects (`deploy-sites` for Cloudflare Pages, `release-packages` for semantic-release) require:
  - `CLOUDFLARE_API_TOKEN` — Cloudflare API token (Workers/Pages:Edit + relevant zone scopes), shared across preview and production deploys.
  - `CLOUDFLARE_ACCOUNT_ID` — Cloudflare account ID paired with the API token.
  - `GITHUB_TOKEN` — GitHub fine-grained PAT for semantic-release plugin authentication.

  `SOPS_AGE_KEY` is not required by these two effects (SOPS encryption/decryption operations stay in GHA-retained workflows and devshell paths). Whether to include it for forward-compatibility is an open question for `/session-plan` (see 7b).
- **`HERCULES_CI_SECRETS_JSON` envelope shape.** The composition `script` produces a JSON object with the shape `{"<SECRET-NAME>": {"data": {"value": "<value>"}}}` keyed by uppercase environment-variable name. Effects access fields via `jq -r '.<SECRET-NAME>.data.value' "$HERCULES_CI_SECRETS_JSON"`. Mirrors vanixiets's exemplar at `modules/effects/vanixiets/secrets.nix:108-119`.
- **`perRepoSecretFiles` registration.** Wire the new generator with `services.buildbot-nix.master.effects.perRepoSecretFiles."github:sciexp/ironstar" = config.clan.core.vars.generators.ironstar-effects-secrets.files.secrets.path;`. Note `sciexp/`, not `cameronraysmith/` — ironstar is the first sibling to break vanixiets's existing `cameronraysmith/<repo>` org-uniformity pattern in this map.

### Buildbot-nix.toml Phase 2 boundary

- **Add `effects_branches = ["*"]` and `effects_on_pull_requests = true`** to `buildbot-nix.toml` at the Phase 2 boundary (vanixiets parity, AC-ratified per the override clause in section 7a). The default `effects_branches` value would gate dispatch to the default branch only; explicitly setting `["*"]` allows preview-on-PR to function. Verified during briefing survey: vanixiets's `~/projects/nix-workspace/vanixiets/buildbot-nix.toml` declares both keys with these exact values; ironstar's current `buildbot-nix.toml` does not declare either key. The Phase 2 boundary edit appends the two keys with the verified vanixiets values.
- **Dry-run rehearsal contract.** Under the (C) effect-fanout topology described below, each per-package release effect attribute (`effects.release-${pkgName}`) is a single attribute that branches internally on `isMain`. On `isMain`, the effect calls `release.sh <pkg-path>` (production semantic-release with `@semantic-release/github` enabled). On non-main branches (including PR pushes), the effect calls `preview-version.sh main <pkg-path>` (dry-run with `@semantic-release/github` filtered out via the script). Internal branching IS the dry-run mechanism. There is no separate `release-packages-dry-run` attribute. The contract mirrors vanixiets's `~/projects/nix-workspace/vanixiets/modules/effects/vanixiets/herculesCI/release-packages.nix:235`, which declares only `release-packages = mkReleasePackagesEffect { dryRun = false; }`; the `dryRun = true` form there is reserved for an opt-in rehearsal attribute that vanixiets does not currently wire and that ironstar likewise does not wire in the first-pass scope.

### Predecessor CD archival

- **Phase 2 archival of `deploy-site.yaml` and `package-release.yaml`** to `.github/deprecated/<file>.yaml` at Phase 2 completion, with `.github/deprecated/README.md` updated to narrate the third archival phase (CD-to-effects, with date and replacement). Vanixiets's deprecated README does not yet narrate its own CD-to-effects archival as a third phase; ironstar does narrate it explicitly per the skeleton's discipline.
- **Update `cd.yaml`** to remove the `preview-site-deploy`, `production-release-packages`, and `production-site-deploy` jobs. Retain `set-variables`, `bootstrap-verification`, `preview-release-version` (semantic-release dry-run preview), and `release-build` (deferred per Q4(a)). The retained `cd.yaml` becomes the workflow_dispatch-only manual orchestrator vanixiets's pattern describes.

## 4. Cross-phase per-job refactoring components

The eight-component refactoring contract groups its components by phase plus a cross-phase set (per the refactoring pattern reference). Phase 1 is substantially complete; Phase 2 introduces components 2 (sidecar), 3 (flake app), 4 (writeShellApplication wrapping), 5 (justfile transition), and 6 (effects module). The cross-phase notes here apply within Phase 2 of this epic.

- **Component 2 (sidecar):** sidecar bash scripts co-located with their flake-app wrapper at `modules/apps/<group>/<name>.sh`, paired one-to-one with `<name>.nix`. The vanixiets convention applies; ironstar has no equivalent convention applied yet. Avoid the parallel hand-written `scripts/` tree pattern that vanixiets's legacy scripts inhabit — new work uses the co-located convention exclusively.
- **Component 4 (writeShellApplication wrapping):** hermetic invariant — runtime PATH equals `runtimeInputs`. Both deploy-sites and release-packages apps use the wrapping. The interpolation form for eval-time payload injection: `text = "export PAYLOAD=${lib.escapeShellArg config.packages.<name>}\n${builtins.readFile ./<name>.sh}"`. The pure form `text = builtins.readFile ./<name>.sh` is acceptable when no eval-time payload is needed. Header comment in each `.nix` file documents which form applies.
- **Component 5 (justfile transition):** justfile recipes transition from `nix develop -c just X → script` to `nix run --accept-flake-config .#<flake-app>` for the migrate-bucket recipes. Pre-transition recipes legitimately not transitioning (per the matrix's caveat for vanixiets): `gh-*` workflow trigger/log/cancel recipes (not effects), `act` local-CI recipes (developer tooling, not effects), and SOPS rotation scripts (developer tooling, not effects). Ironstar should follow the same demarcation.

## 5. Repo-specific carve-outs

Three carve-outs apply per the Q4(a) and Q4(b) AC adjudications.

- **`release.yaml` deferred** until packaging strategy stabilizes. The workflow is currently disabled-by-default (the production `on:` trigger is commented out). It stays in `.github/workflows/` as a retain-bucket workflow until a follow-on epic addresses Rust binary packaging (cargo crate publishing, GitHub Releases, cachix-push integration, etc.).
- **`release-build` deferred** from Phase 2 effect scope. Its eventual home is a flake check (Phase 1 territory) producing the binary, with cachix-push integration in a follow-on epic. The current `release-build` job in `cd.yaml` is gated `vars.ENABLE_RUST_RELEASE == 'true'` and is currently inert; it stays in `cd.yaml` (retain bucket within `cd.yaml`) until the follow-on epic moves the build to a flake check.
- **`update-flake-inputs.yaml` retained as a GHA workflow.** Mirrors vanixiets's retained pattern. Not migrated to a buildbot-nix flake check or a herculesCI effect. Stays in `.github/workflows/`.

## 6. References

Absolute paths only. The brief defers to the canonical references rather than restating them.

- **Vanixiets** (canonical CI + CD reference): `~/projects/nix-workspace/vanixiets/`. Specifically:
  - `flake.nix` (top-level outputs, substituter wiring)
  - `modules/effects/flake-module.nix` (hercules-ci-effects flakeModule import)
  - `modules/effects/vanixiets/herculesCI/deploy-docs.nix` (deploy-docs effect exemplar)
  - `modules/effects/vanixiets/herculesCI/release-packages.nix` (release-packages effect exemplar)
  - `modules/effects/vanixiets/secrets.nix` (per-consumer secrets generator exemplar)
  - `.github/mergify.yml` (mergify alignment exemplar)
  - `.github/workflows/` (retained set after migration)
  - `buildbot-nix.toml` (`attribute = "checks.x86_64-linux"`, `effects_branches`, `effects_on_pull_requests`)
  - `justfile` (specifically `check-fast` and `nix-flake-io` recipes)
- **Refactoring pattern reference**: `~/projects/sciexp/planning/docs/notes/orientation/ci-cd-refactoring-pattern-reference.md` — the eight-component contract, phase grouping, three-bucket workflow categorization, per-consumer secrets generator layout, `HERCULES_CI_SECRETS_JSON` envelope, and Rust check pattern translation guide.
- **Migration matrix**: `~/projects/sciexp/planning/docs/notes/orientation/ci-cd-migration-matrix.md` — current-state snapshot per target repo.
- **Briefing skeleton**: `~/projects/sciexp/planning/docs/notes/orientation/ci-cd-migration-briefing-skeleton.md` — the structure this brief follows.

The brief does not cite buildbot-nix or hercules-ci-effects upstream internals beyond what is necessary to apply the vanixiets pattern.

## 7. Adjudication state

### 7a. Pre-adjudicated AC inputs

The five user-confirmed scope decisions are encoded as facts the `/session-plan` WO consumes, not as questions to re-answer.

**Override clause.** The `/session-plan` WO may override these decisions only if material new information surfaces during planning that contradicts a decision. In that case, the WO surfaces the contradiction to AC for re-confirmation rather than silently revising. Q1–Q5 below, plus the buildbot-nix.toml Phase 2 boundary settings (`effects_branches`, `effects_on_pull_requests`) and the bun + bun2nix substrate decision recorded after Q5, are not "open" or "to be decided" at the planning step.

- **Q1.** Just-recipes are NOT exposed as effects directly. Component 3 (flake app as herculesCI-effect substrate) is the Phase 2 substrate; just-recipes may continue to drive local invocation and may transition to `nix run .#<app>` form per component 5. Factoring detail (thin per-package + iterating effects vs fat parametric apps + thin effects, plus utility apps that need no parametricity such as `list-packages-json` from vanixiets) is deferred to `/session-plan` in ironstar.
- **Q2.** Dedicated `ironstar-effects-secrets` clan-vars generator at `~/projects/nix-workspace/vanixiets/modules/effects/ironstar/secrets.nix` in the vanixiets repository, registered as `services.buildbot-nix.master.effects.perRepoSecretFiles."github:sciexp/ironstar"`. Note `sciexp/` org — ironstar is the first sibling to break vanixiets's `cameronraysmith/<repo>` org-uniformity pattern in this map. Ironstar's effects modules themselves live in ironstar's own tree at `modules/effects/herculesCI/<effect>.nix`, NOT in vanixiets.
- **Q3.** Mergify config at `.github/mergify.yml` (already at correct path; no move required). Required checks aligned with vanixiets: `buildbot/nix-eval`, `buildbot/nix-build`, `buildbot/nix-effects` — three checks, the `nix-effects` rolled-up status replacing per-effect granularity. Per-repo diffs reviewed for repo name (`sciexp` org), bot account names (`sciexp-flake-updater[bot]` for flake-update bot, mirroring vanixiets's `vanixiets-flake-updater[bot]`), dependabot-vs-renovate handling (renovate per `renovate.json`), human-author auto-approve target (currently optional and commented out; remains commented unless AC chooses to enable).
- **Q4.** Two halves.
  - (a) Defer `release.yaml`, `release-build`, and `update-flake-inputs` from the first-pass Phase 2 scope.
  - (b) First-pass Phase 2 effects = `deploy-sites` + `release-packages` ONLY.
  Specifically: `update-flake-inputs` is a **retained** GHA workflow (not deferred-then-migrated), mirroring vanixiets's retained pattern; stays in `.github/workflows/`. `release-build`'s eventual home is a flake check (Phase 1 territory) producing the binary, with cachix-push integration as a follow-on epic. `release.yaml` stays deferred until packaging strategy stabilizes.
- **Q5.** `deploy-sites` and `release-packages` effects parametric over `packages/*` (one parametric effect each, iterating over the packages map). Currently `packages/docs` (Astro Starlight) and `packages/eventcatalog` (EventCatalog). Not thin-per-package + iterating effects. Detailed factoring (whether the parametric body iterates inline within a single effect like vanixiets's `release-packages.nix` does, or fans out at the herculesCI attribute level into `effects.deploy-sites-docs` / `effects.deploy-sites-eventcatalog`) is deferred to `/session-plan` in ironstar.
- **AC-ratified: bun + bun2nix substrate, root `bun.lock` + bun workspaces, effect-fanout topology.**
  - Substrate: verbatim reuse from vanixiets. Reference paths: `~/projects/nix-workspace/vanixiets/flake.nix:94-99` (bun2nix flake input declaration), `~/projects/nix-workspace/vanixiets/pkgs/by-name/vanixiets-docs/package.nix` (main derivation with `dontUseBunBuild = true; dontUseBunInstall = true` at lines 113-114, Typst diagram-compilation buildPhase at lines 121-148, bundled-workerd autoPatchelf at lines 73-80, and `passthru.tests.{unit,linkcheck,e2e}` at lines 167-263), `~/projects/nix-workspace/vanixiets/pkgs/by-name/vanixiets-docs-deps/package.nix` (hermetic node_modules tree derivation), `~/projects/nix-workspace/vanixiets/modules/apps/release/release.nix` (single shared `apps.release` flake app taking package path as argv; PURE READFILE FORM per the source comment at lines 22-29), `~/projects/nix-workspace/vanixiets/modules/apps/docs/deploy.nix` (deploy app; INTERPOLATION FORM per the source comment at lines 9-15), `~/projects/nix-workspace/vanixiets/modules/effects/vanixiets/herculesCI/release-packages.nix` (release effect with single declared attribute `release-packages = mkReleasePackagesEffect { dryRun = false; }` at line 235), `~/projects/nix-workspace/vanixiets/modules/effects/vanixiets/herculesCI/deploy-docs.nix` (deploy effect, hardcoded-single).
  - Lockfile: root `bun.lock` + root `bun.nix` + bun workspaces (vanixiets-aligned, ironstar-already-aligned: ironstar's working tree has root `bun.lock` of 313KB, root `bun.nix` of 270KB, root `package.json` declares `"workspaces": ["packages/*"]` and `"packageManager": "bun@1.3.4"`). Per-package derivations reference the root lockfile via `../../../bun.lock` and `../../../bun.nix` paths. NO per-package lockfiles.
  - Topology: eval-time effect-fanout via `builtins.readDir ./packages` + `lib.listToAttrs`. Per-package effect attributes for both release and deploy, running in parallel. Underlying flake apps are single shared parametric apps (`apps.deploy-sites`, `apps.release-packages`) taking `pkgName` as argv, mirroring vanixiets's `apps.release` convention.
  - Effects' runtime inputs use bun with npm-fallback inside the nix sandbox.
  - Ny3-reconciliation framing: ny3's pnpm decision was documented in CLAUDE.md but not implemented in the working tree. The bun + bun2nix substrate landing in this migration reconciles CLAUDE.md framing with working-tree reality and adds the nix-driven derivation pattern as the new substrate addition.
  - Eventcatalog migration carries discovered-scope risk; first issue under that sub-track is a research/investigation issue per D4 narrowing (which `nativeBuildInputs` and pre-build commands does eventcatalog's catalog generation need + does it require workerd-equivalent binary patching, anchored on vanixiets-docs's Typst + `dontUseBunBuild = true` + `dontUseBunInstall = true` precedent at `~/projects/nix-workspace/vanixiets/pkgs/by-name/vanixiets-docs/package.nix:113-114,121-148`), then implementation issues unblock from there.
  - Observability rationale for fanout-over-loop-internal: buildbot-nix sets `status_name = f"effects.{effect}"` per effect attribute (`~/projects/nix-workspace/buildbot-nix/buildbot_nix/buildbot_nix/nix_eval.py:716-720`). Per-package effect attributes produce per-package GitHub status checks (e.g. `effects.deploy-docs`, `effects.deploy-eventcatalog`) AND a rolled-up `buildbot/nix-effects` aggregate. Mergify gates on the aggregate (stable interface preserved); per-effect status checks deliver granular failure observability. Loop-internal would lose the per-package status surface. This grounds the (C) topology choice in verified buildbot-nix mechanics, making it durable against future re-litigation.

  Subject to the same override clause as Q1–Q5.

### 7b. Open scoping questions for `/session-plan`

These are questions the WO may surface during planning that genuinely cannot be adjudicated until full repo context is loaded. These are NOT re-openings of Q1–Q5; they are questions the dispatch prompt did not pre-adjudicate.

- **Sidecar grouping under `modules/apps/`.** Should the sidecars be grouped per app (`modules/apps/deploy-sites/{deploy-sites.nix, deploy-sites.sh}`) or under a higher-level grouping (`modules/apps/cd/{deploy-sites,release-packages}.{nix,sh}`)? Vanixiets uses `modules/apps/<group>/<name>.{nix,sh}` with groups like `docs`, `release`, `cluster`, `bootstrap`. Ironstar's groups have not been chosen.
- **`SOPS_AGE_KEY` inclusion in the per-consumer secrets generator.** The first-pass Phase 2 effects (`deploy-sites`, `release-packages`) do not require `SOPS_AGE_KEY`. Whether to include it for forward-compatibility (analogous to vanixiets's inclusion despite the SOPS-bootstrap effect being vanixiets-specific) is a `/session-plan` question. Default disposition: include only what current effects need; expand the generator when a future effect requires a new secret.
- **Per-package eligibility-signaling for deploy vs release.** Today both ironstar packages are both deploy-eligible and release-eligible, so the eval-time fanout iterating over every directory in `packages/*` produces the correct effect set without filtering. The eligibility filter (e.g., presence of `wrangler.jsonc` signals deploy-eligible; a `release` block in `package.json` signals release-eligible; or an explicit nix attribute) is a factoring detail for `/session-plan` to surface before silently assuming "iterate over every directory" remains correct as the package set evolves.
- **Per-package peer-dep-conflict factoring concern.** Vanixiets has N=1 site package, so the root `bun.lock` + bun workspaces pattern has no observed peer-dep-conflict precedent at N>1. Ironstar lands at N=2 (`packages/docs` + `packages/eventcatalog`); if peer-dep-range conflicts surface during the bun + bun2nix migration or during downstream dependency upgrades, resolution is a `/session-plan` or implementation-time concern rather than a Phase 1 architectural decision. The lockfile-architecture decision (root `bun.lock` per AC-ratified Q1-resolution in 7a) stands regardless.
- **Whether to migrate `cf-build-deploy` and `cf-versions-deploy`** justfile recipes to flake-app form within the first-pass Phase 2 scope. The matrix enumerates these as pre-transition CD recipes; the dispatch prompt's Q4(b) names "deploy-sites + release-packages ONLY" as the effects, which constrains the effect attributes but leaves the supporting justfile recipe transitions ambiguous. Default disposition: transition any justfile recipe whose target is one of the two new effects; defer transition of recipes whose target remains GHA-bound (e.g. recipes that drive the retained `release.yaml`).
- **`release-packages-dry-run` rehearsal attribute.** Vanixiets's `mkReleasePackagesEffect` parametrizes a `dryRun` flag and the second invocation produces a `release-packages-dry-run` attribute for manual rehearsal. Whether to introduce the rehearsal attribute in the first-pass scope or defer to a follow-on issue is a `/session-plan` question. The `preview-release-version` GHA job in `cd.yaml` provides the current dry-run path; if it is retained in `cd.yaml`, the rehearsal effect attribute may be deferred.
- **Whether the existing `bootstrap-verification` GHA job stays in `cd.yaml` or migrates.** The matrix lists `bootstrap-verification` as a "common GHA-only job" with no flake-check equivalent, and the per-repo `cd.yaml` already runs it. Default disposition: retain in `cd.yaml`; surface as `/session-plan` question only if the WO discovers a clean flake-check substitute.

## Beads epic structure

- **Parent epic per repo**, NOT per phase. Title: `ironstar: CI/CD migration to buildbot-nix and herculesCI effects` (or repo-convention equivalent at the WO's discretion).
- **Child issues per acceptance criterion**, grouped by phase in the issue descriptions but flat in the dependency graph.
- **Delivery shape: 1 PR per target repo**, encoded as an explicit attribute on the parent epic. Brief commit and all subsequent implementation commits stack on the same `buildbot-nix-migration` branch. Atomic-commit-per-file discipline preserves bisect granularity at fast-forward merge time.
- The cross-repo secrets-add PR in vanixiets is a separate PR (2 PRs total per target repo: 1 in ironstar, 1 in vanixiets).
- Issues within the epic provide pre-merge logical bisect points; commits within the PR provide post-merge bisect points.
