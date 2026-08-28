# herculesCI effect: per-package semantic-release run-context dispatcher.
#
# Per-package fanout: `builtins.readDir ./packages` + `lib.listToAttrs`
# generates one `effects.release-${pkgName}` attribute per directory under
# `packages/`. Each attribute runs in its own bwrap sandbox and dispatches
# either production semantic-release or `apps.preview-version` — both are
# flake apps consumed via eval-time-resolved program store paths.
#
# The production-versus-preview choice, and the clone strategy that depends
# on whether the run is a pull request, are taken at RUNTIME from the run
# context resolved by `flake.lib.mkEffectRunContext` (see
# modules/lib/effect-run-context.nix for why evaluation-time branch names
# cannot carry those decisions). Both dispatch bodies and both clone
# strategies are therefore present in every generated script; shell `if`
# statements select between them.
#
# Topology divergence from vanixiets: vanixiets's release-packages.nix uses
# a single `effects.release-packages` attribute that loops at runtime over
# `apps.list-packages-json` output. That topology is incompatible with
# ironstar (no list-packages-json by design) and supersedes the audit
# trail we need: per-package effect attrs give per-package buildbot
# evidence and isolation, matching the obh.15 deploy-sites topology.
#
# The effect-name vocabulary is the bare directory name (`release-docs`,
# `release-eventcatalog`); the `apps.release-packages` and
# `apps.preview-version` argv vocabularies are the `ironstar-`-prefixed
# package name (`ironstar-docs`, `ironstar-eventcatalog`). The `pkgArg`
# let-binding localises the prefix-mapping at the dispatch boundary
# (resolution S1).
#
# Substrate exemplars:
# `~/projects/nix-workspace/vanixiets/modules/effects/vanixiets/herculesCI/release-packages.nix`
# (script body content: env vars, ref handling, secrets, helpers — borrowed verbatim;
#  top-level structure: per-package fanout — adapted from obh.15 deploy-sites).
{
  config,
  inputs,
  lib,
  self,
  withSystem,
  ...
}:
{
  herculesCI =
    herculesCI:
    let
      # Nullable: null on tag pushes (no branch). Per obh.20 null-guard.
      branch = herculesCI.config.repo.branch;
      shortRev = herculesCI.config.repo.shortRev;
      rev = herculesCI.config.repo.rev;

      # Eval-time fallback inputs for the run-context resolver: used verbatim
      # when the build service issues no identity token.
      evalBranchArg = if branch == null then "" else toString branch;

      # ironstar prefix lives at the nix-derivation / app-argv vocabulary
      # only; effect names are the bare dir name. Boundary mapping (S1).
      pkgArg = pkg: "ironstar-${pkg}";

      # Eval-time enumeration over packages/. Per resolution S2:
      # readDir returns { docs = "directory"; eventcatalog = "directory"; ... };
      # filter to dirs only and emit one effect attribute per package.
      packageNames = lib.attrNames (
        lib.filterAttrs (_: type: type == "directory") (builtins.readDir ../../../packages)
      );

      mkReleaseEffect =
        pkgName:
        withSystem "x86_64-linux" (
          { config, pkgs, ... }:
          let
            hci-effects = inputs.hercules-ci-effects.lib.withPkgs pkgs;

            # Eval-time store-path resolution per obh.21 (no `nix run .#`
            # under bwrap; no nix daemon, no network).
            releasePackagesProgram = config.apps.release-packages.program;
            previewVersionProgram = config.apps.preview-version.program;
            runContextProgram = lib.getExe (self.lib.mkEffectRunContext pkgs);

            # S1: prefix application at the dispatch boundary.
            packageArg = pkgArg pkgName;
            packagePath = "packages/${pkgName}";
          in
          hci-effects.mkEffect {
            name = "release-${pkgName}";

            # Why: mkEffect's defaultInputs do not include git; clone preamble
            # below requires it (obh.18). semantic-release also shells out to
            # git for tag/push operations.
            inputs = [ pkgs.git ];

            # JSON-encoded string, not a nix list: a bare list reaches the
            # derivation as the space-joined `ironstar-ci`, which the build
            # service rejects as unparseable before the effect starts.
            # Declaring an audience is what makes it inject
            # NIXBOT_ID_TOKEN_REQUEST_URL / _TOKEN.
            idTokenAudiences = builtins.toJSON [ self.lib.effectIdTokenAudience ];

            # Only destinations named here are written into
            # $HERCULES_CI_SECRETS_JSON; an omitted map grants nothing under
            # the current build service. The previous service ignored the map
            # and passed the whole composed secrets file, so declaring one
            # narrows what the sandbox sees there rather than changing it.
            # Left name is what the script reads; right name is the key in
            # the composed secrets file.
            secretsMap.GITHUB_TOKEN = "GITHUB_TOKEN";

            effectScript = ''
              set -euo pipefail

              echo "=== effects.release-${pkgName} (per-package semantic-release dispatcher) ==="
              echo "package:      ${packageArg}"
              echo "pkg-path:     ${packagePath}"
              echo "branch(eval): ${lib.escapeShellArg (toString branch)}"
              echo "rev:          ${lib.escapeShellArg (toString rev)}"
              echo "shortRev:     ${lib.escapeShellArg (toString shortRev)}"

              eval "$(${runContextProgram} ${lib.escapeShellArg evalBranchArg} ${lib.escapeShellArg (toString shortRev)})"

              echo "run-context:   $IRONSTAR_RUN_CONTEXT_SOURCE"
              echo "event:         $IRONSTAR_EVENT"
              echo "pr-number:     $IRONSTAR_PR_NUMBER"
              echo "is-production: $IRONSTAR_IS_PRODUCTION"

              if [ "$IRONSTAR_IS_PRODUCTION" = true ]; then
                echo "RELEASE-${lib.toUpper pkgName}-ACTION: release"
              else
                echo "RELEASE-${lib.toUpper pkgName}-ACTION: preview-version"
              fi

              export GITHUB_TOKEN="$(jq -r '.GITHUB_TOKEN.data.value' "$HERCULES_CI_SECRETS_JSON")"

              if [ -z "''${GITHUB_TOKEN:-}" ] || [ "$GITHUB_TOKEN" = "null" ]; then
                echo "error: GITHUB_TOKEN missing from \$HERCULES_CI_SECRETS_JSON" >&2
                exit 1
              fi

              # Why: do not use config.repo.remoteHttpUrl — buildbot-nix bakes
              # the App installation token into it; would leak via banner echo.
              # Hardcoded clone URL per obh.23.
              clone_url="https://github.com/sciexp/ironstar.git"

              clone_dir="$(mktemp -d -t release-${pkgName}-clone.XXXXXX)"

              trap 'rm -rf "$clone_dir"' EXIT

              GIT_REV=${lib.escapeShellArg (toString rev)}
              # Empty when the run has no branch name: a tag push, or a
              # pull-request identity token, which carries no head ref.
              GIT_BRANCH="$IRONSTAR_BRANCH"
              if [ "$IRONSTAR_EVENT" = pull_request ]; then
                # GitHub's refs/pull/<N>/merge is a synthetic test-merge
                # ref recomputed on base advance, head update, or
                # merge-test scheduler fire; the T0 eval SHA drifts from
                # T1 runtime content. refs/pull/<N>/head is the
                # dev-pushed source-branch tip, stable until the next dev
                # push (obh.22).
                git clone "$clone_url" "$clone_dir"
                git -C "$clone_dir" fetch --tags origin

                # `git fetch origin refs/pull/<N>/head` alone updates
                # FETCH_HEAD but does NOT auto-create the remote-tracking
                # ref; the explicit `+ref:remote-tracking-ref` mapping
                # closes that gap (idiom from buildbot-nix
                # buildbot_nix/buildbot_nix/nix_eval.py:GitLocalPrMerge).
                git -C "$clone_dir" fetch origin \
                  "+refs/pull/$IRONSTAR_PR_NUMBER/head:refs/remotes/origin/pr-$IRONSTAR_PR_NUMBER-head"
                head_sha="$(git -C "$clone_dir" rev-parse "origin/pr-$IRONSTAR_PR_NUMBER-head")"

                echo "RELEASE-CLONE-PR-HEAD: $IRONSTAR_PR_NUMBER $head_sha"
                echo "RELEASE-CLONE-PR-DISPATCH: $IRONSTAR_PR_NUMBER eval-rev=$GIT_REV head=$head_sha"

                echo "RELEASE-CLONE-START: $clone_url $GIT_REV $GIT_BRANCH"

                git -C "$clone_dir" checkout -B "pr-$IRONSTAR_PR_NUMBER-head" "$head_sha"
                echo "RELEASE-CLONE-CHECKOUT: $head_sha"

                # Trivially true post-fetch unless force-push race lost the head ref; set -e propagates abort.
                git -C "$clone_dir" rev-parse --verify "origin/pr-$IRONSTAR_PR_NUMBER-head" >/dev/null
              else
                echo "RELEASE-CLONE-START: $clone_url $GIT_REV $GIT_BRANCH"

                git clone "$clone_url" "$clone_dir"
                git -C "$clone_dir" fetch --tags origin

                if [ -n "$GIT_BRANCH" ]; then
                  checkout_branch="$GIT_BRANCH"
                else
                  checkout_branch="release-${pkgName}-detached"
                fi
                git -C "$clone_dir" checkout -B "$checkout_branch" "$GIT_REV"
                echo "RELEASE-CLONE-CHECKOUT: $GIT_REV"

                # Stale-rev guard (obh.25): compare clone HEAD against
                # origin/$GIT_BRANCH; abort on mismatch. Skipped for
                # tag/detached pushes where $GIT_BRANCH is empty.
                if [ -n "$GIT_BRANCH" ]; then
                  git -C "$clone_dir" fetch origin "$GIT_BRANCH"
                  head_rev="$(git -C "$clone_dir" rev-parse HEAD)"
                  remote_rev="$(git -C "$clone_dir" rev-parse "origin/$GIT_BRANCH")"
                  if [ "$head_rev" != "$remote_rev" ]; then
                    echo "RELEASE-CLONE-STALE: expected $head_rev remote $remote_rev" >&2
                    exit 1
                  fi
                fi
              fi

              echo "RELEASE-CLONE-READY: $clone_dir"

              # semantic-release's get-git-auth-url.js treats GIT_CREDENTIALS
              # as user:password and constructs the authenticated URL
              # in-process. The ironstar-effects-secrets PAT (Read+Write) is
              # the canonical authority — the buildbot-nix App installation
              # token (Read-only) is NOT reused for release mutation.
              # Username MUST NOT be `x-access-token` here: that string is
              # reserved for GitHub App installation tokens (ghs_*) and
              # routes fine-grained PATs (github_pat_*) into the wrong
              # credential validator, surfacing as "Invalid username or
              # token. Password authentication is not supported for Git
              # operations." at git push despite Bearer-auth (API)
              # succeeding. `oauth2` is the conventional username for
              # fine-grained and classic PATs over HTTPS Basic auth (obh.17).
              export GIT_CREDENTIALS="oauth2:''${GITHUB_TOKEN}"

              # CI=true bypasses semantic-release's env-ci abort.
              # GIT_AUTHOR/COMMITTER are honoured natively without writing
              # .git/config (which the bwrap /nix/store ro-bind would block)
              # (obh.19).
              export CI=true
              export GIT_BRANCH
              export RELEASE_REPO_ROOT="$clone_dir"
              export GIT_AUTHOR_NAME=semantic-release
              export GIT_AUTHOR_EMAIL=semantic-release@ironstar.local
              export GIT_COMMITTER_NAME=semantic-release
              export GIT_COMMITTER_EMAIL=semantic-release@ironstar.local

              # Why: bwrap sandbox does not bind working tree; .# cannot
              # resolve. Use eval-time /nix/store paths (obh.21).
              RELEASE_PACKAGES=${releasePackagesProgram}
              PREVIEW_VERSION=${previewVersionProgram}

              # Operate from the cloned tree so semantic-release's
              # git-aware operations (tag, push, refs) target the right
              # repository. `apps.release-packages` reads RELEASE_REPO_ROOT
              # but the script also calls `git rev-parse --show-toplevel`
              # transitively via semantic-release; cd into $clone_dir
              # ensures consistency.
              cd "$clone_dir"

              if [ "$IRONSTAR_IS_PRODUCTION" = true ]; then
                # Production path: apps.release-packages dispatches the
                # parametric semantic-release wrapper for ${packageArg}.
                release_log="$(mktemp -t release-${pkgName}-prod.XXXXXX.log)"
                set +e
                "$RELEASE_PACKAGES" ${lib.escapeShellArg packageArg} 2>&1 | tee "$release_log"
                release_rc=''${PIPESTATUS[0]}
                set -e
                if [ "$release_rc" -ne 0 ]; then
                  echo "error: release-${pkgName} production exited $release_rc" >&2
                  exit "$release_rc"
                fi
                echo "RELEASE-${lib.toUpper pkgName}-OK: production"
              else
                # Preview path: apps.preview-version dispatches the
                # parametric semantic-release dry-run wrapper for
                # ${packageArg}. Eval-time-injected node_modules trees
                # avoid any nix-daemon invocation under bwrap (obh.33).
                preview_log="$(mktemp -t release-${pkgName}-preview.XXXXXX.log)"
                set +e
                "$PREVIEW_VERSION" main ${lib.escapeShellArg packageArg} 2>&1 | tee "$preview_log"
                preview_rc=''${PIPESTATUS[0]}
                set -e
                if [ "$preview_rc" -ne 0 ]; then
                  echo "error: release-${pkgName} preview exited $preview_rc" >&2
                  exit "$preview_rc"
                fi
                echo "RELEASE-${lib.toUpper pkgName}-OK: preview"
              fi

              echo "=== release-${pkgName} effect complete (exit 0) ==="
            '';
          }
        );

      # Per-package effect attrs: { release-docs = ...; release-eventcatalog = ...; }
      releaseEffects = lib.listToAttrs (
        map (pkgName: {
          name = "release-${pkgName}";
          value = mkReleaseEffect pkgName;
        }) packageNames
      );
    in
    {
      onPush.default.outputs.effects = releaseEffects;
    };
}
