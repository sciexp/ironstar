# Regenerate the committed build-graph snapshot from the canonical roots.
#
# nix run .#build-graph-snapshot
#
# The snapshot is the operating envelope the graph-drift regulator
# (modules/checks/build-graph-invariants.nix) gates against. It is generated
# OUTSIDE the build sandbox because it needs recursive nix (`nix derivation
# show -r`), which a flake check derivation cannot invoke. The output is keyed
# on a hash-free tuple and carries no store hashes or timestamps, so two runs
# produce byte-identical output and the regulator is content-addressed.
#
# Pinned to system x86_64-linux: that is what buildbot CI evaluates, and the
# crate2nix roots eval purely there (and cross-system from darwin) with no IFD.
# The crane packages.ironstar/ironstar-release and checks.ironstar-e2e roots are
# excluded: they are IFD-bound on linux and crane is removed at task 6.
{ ... }:
{
  perSystem =
    {
      pkgs,
      lib,
      ...
    }:
    let
      system = "x86_64-linux";
      perMemberTestRoots = map (member: "checks.${member}-test") [
        "ironstar"
        "ironstar-core"
        "ironstar-shared-kernel"
        "ironstar-todo"
        "ironstar-session"
        "ironstar-analytics"
        "ironstar-workspace"
        "ironstar-event-store"
        "ironstar-event-bus"
        "ironstar-analytics-infra"
        "ironstar-session-store"
      ];
      canonicalRoots = [
        "packages.ironstar-c2n"
        "packages.ironstar-release-c2n"
        "checks.workspace-clippy"
        "checks.workspace-test"
        "checks.cargo-nix-lock-sync"
      ]
      ++ perMemberTestRoots;
      normalizer = ./build-graph-snapshot/normalize.py;
    in
    {
      apps.build-graph-snapshot = {
        type = "app";
        meta.description = "Regenerate the committed build-graph snapshot (hash-free graph envelope) from the canonical x86_64-linux roots.";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "build-graph-snapshot";
            runtimeInputs = [
              pkgs.nix
              pkgs.git
              pkgs.python3
            ];
            text = ''
              repo_root=$(git rev-parse --show-toplevel)
              cd "$repo_root"
              raw_dir=$(mktemp -d)
              trap 'rm -rf "$raw_dir"' EXIT
              echo "Extracting build-graph for ${system} from the canonical roots..."
              for root in ${lib.escapeShellArgs canonicalRoots}; do
                cat=''${root%%.*}
                name=''${root#*.}
                echo "  nix derivation show -r .#$cat.${system}.$name"
                nix derivation show -r ".#$cat.${system}.$name" \
                  > "$raw_dir/''${cat}__''${name}.json"
              done
              echo "Normalizing to the committed hash-free snapshot..."
              python3 ${normalizer} "$raw_dir" "${system}" \
                > modules/checks/build-graph-snapshot.json
              echo "Wrote modules/checks/build-graph-snapshot.json"
              git --no-pager diff --stat -- modules/checks/build-graph-snapshot.json || true
            '';
          }
        );
      };
    };
}
