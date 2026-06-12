# Regenerate the committed build-graph snapshot from the canonical roots.
#
# nix run .#build-graph-snapshot
#
# The snapshot is a committed, byte-deterministic, NON-GATING record of the
# build-graph shape. It is not a flake check and gates nothing: it is a tracked
# artifact whose git diff is the review surface. Regenerate it on an explicit
# human decision — when a dependency is added or removed, or the workspace is
# rearchitected — and never per-PR. Review changes to it through its git diff
# per docs/notes/reference/build-graph-review-runbook.md.
#
# It is generated OUTSIDE the build sandbox because it needs recursive nix
# (`nix derivation show -r`), which a flake check derivation cannot invoke. The
# output is keyed on a hash-free tuple and carries no store hashes or timestamps,
# so two runs produce byte-identical output.
#
# Pinned to system x86_64-linux: that is what buildbot CI evaluates, and the
# crate2nix roots eval purely there (and cross-system from darwin) with no IFD.
# packages.ironstar/ironstar-release are the crate2nix builds and are canonical
# roots. checks.ironstar-e2e is excluded: it is IFD-bound on linux (it builds the
# bun2nix frontend asset during eval).
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
        "packages.ironstar"
        "packages.ironstar-release"
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
              raw_dir="$repo_root/logs/build-graph/raw"
              rm -rf "$raw_dir"
              mkdir -p "$raw_dir"
              drv_paths_file=$(mktemp)
              trap 'rm -f "$drv_paths_file"' EXIT
              echo "Extracting build-graph for ${system} from the canonical roots..."
              for root in ${lib.escapeShellArgs canonicalRoots}; do
                cat=''${root%%.*}
                name=''${root#*.}
                echo "  nix derivation show -r .#$cat.${system}.$name"
                nix derivation show -r ".#$cat.${system}.$name" \
                  > "$raw_dir/''${cat}__''${name}.json"
                nix eval --raw ".#$cat.${system}.$name.drvPath" >> "$drv_paths_file"
                printf '\n' >> "$drv_paths_file"
              done
              cargo_nix_sha256=$(sha256sum Cargo.nix | cut -d' ' -f1)
              root_drv_paths_sha256=$(sort "$drv_paths_file" | sha256sum | cut -d' ' -f1)
              echo "Normalizing to the committed hash-free snapshot..."
              python3 ${normalizer} "$raw_dir" "${system}" \
                "$cargo_nix_sha256" "$root_drv_paths_sha256" \
                > modules/apps/build-graph-snapshot/snapshot.json
              echo "Wrote modules/apps/build-graph-snapshot/snapshot.json"
              git --no-pager diff --stat -- modules/apps/build-graph-snapshot/snapshot.json || true
            '';
          }
        );
      };
    };
}
