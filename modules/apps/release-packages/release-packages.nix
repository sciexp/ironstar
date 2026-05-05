# release-packages.nix - Parametric semantic-release wrapper as a flake app.
#
#   nix run .#release-packages -- ironstar-docs
#   nix run .#release-packages -- ironstar-docs --dry-run
#   nix run .#release-packages -- ironstar-eventcatalog
#   nix run .#release-packages -- info ironstar-docs
#
# Why: a single flake app dispatches semantic-release over both
# ironstar-docs and ironstar-eventcatalog by accepting the package name
# as positional argv $1. The script routes to the correct package
# directory using eval-time injected node_modules paths, configures git,
# invokes semantic-release, filters @semantic-release/github when
# --dry-run is set, and provides an `info` subcommand.
#
# Hermetic: semantic-release and all plugins are provided by the
# package-specific deps derivation and linked into the package directory
# at runtime. Callers do not need to run `bun install`.
#
# Expected caller environment (CI-only):
#   GITHUB_TOKEN, CI, RELEASE_REPO_ROOT,
#   GIT_AUTHOR_NAME / GIT_AUTHOR_EMAIL,
#   GIT_COMMITTER_NAME / GIT_COMMITTER_EMAIL.
#
# Template bifurcation (writeShellApplication): INTERPOLATION FORM.
# `text` is a nix string that injects two eval-time-computed paths
# (IRONSTAR_DOCS_PAYLOAD via config.packages.ironstar-docs,
# IRONSTAR_EVENTCATALOG_PAYLOAD via config.packages.ironstar-eventcatalog)
# into the script preamble before the readFile'd sidecar body.
# Contrast with the vanixiets release.nix exemplar (PURE READFILE FORM),
# which only handles a single package and so injects its node_modules path
# via runtimeEnv only. Here, dispatch over two packages requires both
# payloads to be visible to the script, so interpolation form is used.
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
      apps.release-packages = {
        type = "app";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "release-packages";
            runtimeInputs = [
              pkgs.nodejs
              pkgs.bun
              pkgs.jq
              pkgs.coreutils
              pkgs.git
              pkgs.gnugrep
              # Explicitly declare every host-PATH binary that
              # release-packages.sh OR any transitive semantic-release
              # plugin / node_modules helper might shell out to. The
              # buildbot-effects bwrap sandbox provides only /nix/store
              # ro-bind + writeShellApplication runtimeInputs PATH (no
              # host PATH binaries); a missing input surfaces only at
              # runtime as `command not found`.
              pkgs.gnused
              pkgs.gawk
              pkgs.findutils
            ];
            text = ''
              export IRONSTAR_DOCS_PAYLOAD=${lib.escapeShellArg config.packages.ironstar-docs}
              export IRONSTAR_EVENTCATALOG_PAYLOAD=${lib.escapeShellArg config.packages.ironstar-eventcatalog}
              ${builtins.readFile ./release-packages.sh}
            '';
          }
        );
      };
    };
}
