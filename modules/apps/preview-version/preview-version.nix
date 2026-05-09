# preview-version.nix - Parametric semantic-release preview wrapper as a flake app.
#
#   nix run .#preview-version                              # root, on main
#   nix run .#preview-version -- main ironstar-docs        # docs preview on main
#   nix run .#preview-version -- main ironstar-eventcatalog
#
# Hermetic: semantic-release and plugins are provided by the per-package
# deps derivation linked into the temporary worktree at runtime; no
# `nix develop -c bun ...` invocation. Compatible with the bwrap effect
# sandbox (no nix daemon, no network).
#
# Template bifurcation (writeShellApplication): PURE READFILE FORM,
# parametrised per-package via runtimeEnv (mirrors vanixiets
# modules/apps/docs/preview-version.nix). Per-package node_modules trees
# are injected so the script can switch on argv $2 (pkgName).
{ ... }:
{
  perSystem =
    {
      pkgs,
      lib,
      config,
      ...
    }:
    {
      apps.preview-version = {
        type = "app";
        meta.description = "Preview the semantic-release version that would publish for an ironstar npm package after merging into a target branch.";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "preview-version";
            runtimeInputs = [
              pkgs.nodejs
              pkgs.git
              pkgs.jq
              pkgs.gnugrep
              pkgs.coreutils
              pkgs.gnused
              pkgs.gawk
              pkgs.findutils
            ];
            runtimeEnv = {
              IRONSTAR_DOCS_NODE_MODULES = "${config.packages.ironstar-docs-deps}/packages/docs/node_modules";
              IRONSTAR_EVENTCATALOG_NODE_MODULES = "${config.packages.ironstar-eventcatalog-deps}/packages/eventcatalog/node_modules";
            };
            text = builtins.readFile ./preview-version.sh;
          }
        );
      };
    };
}
