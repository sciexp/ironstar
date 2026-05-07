# deploy-sites.nix - Parametric Cloudflare Workers deploy app.
#
#   nix run .#deploy-sites -- ironstar-docs preview <branch>
#   nix run .#deploy-sites -- ironstar-docs production
#   nix run .#deploy-sites -- ironstar-eventcatalog preview <branch>
#   nix run .#deploy-sites -- ironstar-eventcatalog production
#
# Why: a single flake app dispatches over both ironstar-docs and
# ironstar-eventcatalog by accepting the package name as positional argv $1.
# The script then routes to the correct nix-built CF Worker payload via
# eval-time injected store paths.
#
# Template bifurcation (writeShellApplication): INTERPOLATION FORM.
# `text` is a nix string that injects two eval-time-computed payload
# paths (IRONSTAR_DOCS_PAYLOAD via config.packages.ironstar-docs,
# IRONSTAR_EVENTCATALOG_PAYLOAD via config.packages.ironstar-eventcatalog)
# into the script preamble before the readFile'd sidecar body. The script
# then dispatches on $1 to select which payload to deploy.
#
# Per-package node_modules trees are injected via runtimeEnv (vanixiets
# modules/apps/docs/deploy.nix:49-51 pattern) so the writeShellApplication
# wrapper guarantees they are present at script invocation time without
# requiring re-export inside the .sh body. The deps derivations land at
# pkgs/by-name/ironstar-{docs,eventcatalog}-deps and produce
# $out/packages/<pkg>/node_modules/.bin/wrangler.
#
# Contrast with release-packages.nix, which uses the same interpolation
# form because it also dispatches between two nix-built derivation outputs.
{ ... }:
{
  perSystem =
    {
      config,
      pkgs,
      lib,
      ...
    }:
    {
      apps.deploy-sites = {
        type = "app";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "deploy-sites";
            # Secrets flow via inherited env (CLOUDFLARE_API_TOKEN, etc.),
            # never via `sops exec-env` inside the script.
            #
            # sed/awk/grep/find are explicitly declared because the
            # hercules-ci-effects bwrap sandbox PATH does not include them
            # by default. Required for the writeShellApplication invariant
            # that PATH equals runtimeInputs at runtime.
            runtimeInputs = [
              pkgs.nodejs
              pkgs.bun
              pkgs.jq
              pkgs.coreutils
              pkgs.git
              pkgs.gnugrep
              pkgs.gnused
              pkgs.gawk
              pkgs.findutils
            ];
            runtimeEnv = {
              IRONSTAR_DOCS_NODE_MODULES = "${config.packages.ironstar-docs-deps}/packages/docs/node_modules";
              IRONSTAR_EVENTCATALOG_NODE_MODULES = "${config.packages.ironstar-eventcatalog-deps}/packages/eventcatalog/node_modules";
            };
            text = ''
              export IRONSTAR_DOCS_PAYLOAD=${lib.escapeShellArg config.packages.ironstar-docs}
              export IRONSTAR_EVENTCATALOG_PAYLOAD=${lib.escapeShellArg config.packages.ironstar-eventcatalog}
              ${builtins.readFile ./deploy-sites.sh}
            '';
          }
        );
      };
    };
}
