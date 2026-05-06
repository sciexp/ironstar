# herculesCI effect: per-package Cloudflare deployment branch-dispatcher.
#
# Per-package fanout: `builtins.readDir ./packages` + `lib.listToAttrs`
# generates one `effects.deploy-${pkgName}` attribute per directory under
# `packages/`. Each attribute runs in its own bwrap sandbox and dispatches
# `production` (on `isMain`) or `preview <branch>` (otherwise) to the shared
# parametric `apps.deploy-sites` flake app.
#
# The effect-name vocabulary is the bare directory name (`docs`,
# `eventcatalog`); the `apps.deploy-sites` argv vocabulary is the
# `ironstar-`-prefixed package name (`ironstar-docs`,
# `ironstar-eventcatalog`). The `pkgArg` let-binding localises the
# prefix-mapping at the dispatch boundary (see resolution S1).
#
# Substrate exemplar:
# `~/projects/nix-workspace/vanixiets/modules/effects/vanixiets/herculesCI/deploy-docs.nix`
# (single hardcoded effect → per-package fanout).
{
  config,
  inputs,
  lib,
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

      isMain = branch == "main";

      # ironstar prefix lives at the nix-derivation / app-argv vocabulary
      # only; effect names are the bare dir name. Boundary mapping (S1).
      pkgArg = pkg: "ironstar-${pkg}";

      # Eval-time enumeration over packages/. Per resolution S2:
      # readDir returns { docs = "directory"; eventcatalog = "directory"; ... };
      # filter to dirs only and emit one effect attribute per package.
      packageNames = lib.attrNames (
        lib.filterAttrs (_: type: type == "directory") (builtins.readDir ../../../packages)
      );

      mkDeployEffect =
        pkgName:
        withSystem "x86_64-linux" (
          { config, pkgs, ... }:
          let
            hci-effects = inputs.hercules-ci-effects.lib.withPkgs pkgs;

            # Eval-time store-path resolution per obh.21 (no `nix run .#`
            # under bwrap; no nix daemon, no network).
            deploySitesProgram = config.apps.deploy-sites.program;

            actionBanner = if isMain then "promote" else "preview-upload";

            # Null-guard on branch/shortRev per obh.20: when branch is null
            # (tag push) or empty, fall back to shortRev for the preview slug.
            previewBranchArg = if branch != null && branch != "" then branch else shortRev;

            # S1: prefix application at the dispatch boundary.
            packageArg = pkgArg pkgName;
          in
          hci-effects.mkEffect {
            name = "deploy-${pkgName}";

            effectScript = ''
              set -euo pipefail

              echo "=== effects.deploy-${pkgName} (per-package deploy dispatcher) ==="
              echo "package: ${packageArg}"
              echo "branch:   ${lib.escapeShellArg (toString branch)}"
              echo "rev:      ${lib.escapeShellArg (toString rev)}"
              echo "shortRev: ${lib.escapeShellArg (toString shortRev)}"
              echo "isMain:   ${if isMain then "true" else "false"}"

              echo "DEPLOY-${lib.toUpper pkgName}-ACTION: ${actionBanner}"

              export CLOUDFLARE_API_TOKEN="$(jq -r '.CLOUDFLARE_API_TOKEN.data.value' "$HERCULES_CI_SECRETS_JSON")"
              export CLOUDFLARE_ACCOUNT_ID="$(jq -r '.CLOUDFLARE_ACCOUNT_ID.data.value' "$HERCULES_CI_SECRETS_JSON")"

              export GIT_REV=${lib.escapeShellArg (toString rev)}
              export GIT_REV_SHORT=${lib.escapeShellArg (toString shortRev)}
              export GIT_REV_SHORT12=${lib.escapeShellArg (builtins.substring 0 12 (toString rev))}
              export GIT_BRANCH=${lib.escapeShellArg (if branch == null then "" else toString branch)}
              export GIT_COMMIT_MSG=${lib.escapeShellArg "effect deploy from rev ${toString shortRev}"}
              export GIT_WORKTREE_STATUS=clean

              # Why: whoami/hostname not on bwrap PATH; hardcoded literals per obh.24.
              export DEPLOY_DEPLOYER=hercules-ci-effects
              export DEPLOY_HOST=magnetite

              if [ -z "''${CLOUDFLARE_API_TOKEN:-}" ] || [ "$CLOUDFLARE_API_TOKEN" = "null" ]; then
                echo "error: CLOUDFLARE_API_TOKEN missing from \$HERCULES_CI_SECRETS_JSON" >&2
                exit 1
              fi
              if [ -z "''${CLOUDFLARE_ACCOUNT_ID:-}" ] || [ "$CLOUDFLARE_ACCOUNT_ID" = "null" ]; then
                echo "error: CLOUDFLARE_ACCOUNT_ID missing from \$HERCULES_CI_SECRETS_JSON" >&2
                exit 1
              fi

              # Why: bwrap sandbox does not bind working tree; .# cannot resolve. Use eval-time /nix/store path (obh.21).
              DEPLOY_SITES=${deploySitesProgram}

              ${
                if isMain then
                  ''
                    # Production path: deploy-sites <pkg-arg> production.
                    deploy_log="$(mktemp -t deploy-${pkgName}-prod.XXXXXX.log)"
                    set +e
                    "$DEPLOY_SITES" ${lib.escapeShellArg packageArg} production 2>&1 | tee "$deploy_log"
                    deploy_rc=''${PIPESTATUS[0]}
                    set -e
                    if grep -q "falling back to direct deploy" "$deploy_log"; then
                      echo "DEPLOY-${lib.toUpper pkgName}-ACTION: fresh-deploy-and-promote"
                    fi
                    if [ "$deploy_rc" -ne 0 ]; then
                      echo "error: deploy-${pkgName} production exited $deploy_rc" >&2
                      exit "$deploy_rc"
                    fi
                  ''
                else
                  ''
                    # Preview path: deploy-sites <pkg-arg> preview <branch-or-shortRev>.
                    preview_log="$(mktemp -t deploy-${pkgName}-preview.XXXXXX.log)"
                    set +e
                    "$DEPLOY_SITES" ${lib.escapeShellArg packageArg} preview ${lib.escapeShellArg previewBranchArg} 2>&1 | tee "$preview_log"
                    upload_rc=''${PIPESTATUS[0]}
                    set -e
                    preview_url="$(grep -oE 'Preview URL: https://[^[:space:]]+' "$preview_log" | head -1 | awk '{print $3}' || true)"
                    if [ -n "$preview_url" ]; then
                      echo "DEPLOY-${lib.toUpper pkgName}-PREVIEW-URL: $preview_url"
                    else
                      echo "warning: could not parse preview URL from deploy-sites output" >&2
                    fi
                    if [ "$upload_rc" -ne 0 ]; then
                      echo "error: deploy-${pkgName} preview exited $upload_rc" >&2
                      exit "$upload_rc"
                    fi
                  ''
              }

              echo "=== deploy-${pkgName} effect complete (exit 0) ==="
            '';
          }
        );

      # Per-package effect attrs: { deploy-docs = ...; deploy-eventcatalog = ...; }
      deployEffects = lib.listToAttrs (
        map (pkgName: {
          name = "deploy-${pkgName}";
          value = mkDeployEffect pkgName;
        }) packageNames
      );
    in
    {
      onPush.default.outputs.effects = deployEffects;
    };
}
