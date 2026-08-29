# Build the build-graph report: a queryable DuckDB projection of the
# inputs.drvs build closure plus rendered review artifacts.
#
# nix run .#build-graph-report
#
# Where build-graph-snapshot emits a single committed scalar envelope reviewed
# through its git diff, the report materializes the same hash-free build-graph
# projection in a relational shape (nodes, edges, root_membership) so DuckDB can
# answer the duplication, ego-neighborhood, divergence, and member-condensation
# questions the scalar snapshot cannot. Its outputs land under logs/build-graph/
# (gitignored) and are regenerated on demand, not committed.
#
# Node and edge identity is normalize.py's hash-free six-tuple, imported as a
# module by emit_edges.py so the report's node identity cannot drift from the
# committed snapshot's node identity.
#
# Pinned to system x86_64-linux: the nixbot worker set offers only that
# system-platform combination, so it is the only system CI evaluates
# (nixbot.toml scopes `attribute` to `checks.x86_64-linux`), the crate2nix
# roots eval purely there with no IFD, and checks.ironstar-e2e is excluded as
# IFD-bound on linux. The canonical root list is kept in sync with
# build-graph-snapshot.nix by construction below.
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
      emitEdges = ./build-graph-report/emit_edges.py;
      loadDuckdb = ./build-graph-report/load_duckdb.sql;
      render = ./build-graph-report/render.py;
      normalizeLib = pkgs.runCommandLocal "build-graph-normalize-lib" { } ''
        mkdir -p "$out"
        cp ${./build-graph-snapshot/normalize.py} "$out/normalize.py"
      '';
    in
    {
      apps.build-graph-report = {
        type = "app";
        meta.description = "Build the queryable DuckDB build-graph report (inputs.drvs closure projection) and rendered review artifacts under logs/build-graph/.";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "build-graph-report";
            runtimeInputs = [
              pkgs.nix
              pkgs.git
              pkgs.python3
              pkgs.duckdb
              pkgs.graphviz
            ];
            text = ''
              repo_root=$(git rev-parse --show-toplevel)
              cd "$repo_root"
              out_dir="$repo_root/logs/build-graph"
              raw_dir="$out_dir/raw"
              mkdir -p "$out_dir"

              if [ ! -d "$raw_dir" ] || [ -z "$(ls -A "$raw_dir" 2>/dev/null)" ]; then
                echo "No raw extractions found; extracting build-graph for ${system} from the canonical roots..."
                mkdir -p "$raw_dir"
                for root in ${lib.escapeShellArgs canonicalRoots}; do
                  cat=''${root%%.*}
                  name=''${root#*.}
                  echo "  nix derivation show -r .#$cat.${system}.$name"
                  nix derivation show -r ".#$cat.${system}.$name" \
                    > "$raw_dir/''${cat}__''${name}.json"
                done
              else
                echo "Reusing existing raw extractions in $raw_dir"
              fi

              echo "Emitting hash-free node and edge tables..."
              export PYTHONPATH="${normalizeLib}''${PYTHONPATH:+:$PYTHONPATH}"
              python3 ${emitEdges} "$raw_dir" "${system}" "$out_dir"

              echo "Loading DuckDB database..."
              duckdb "$out_dir/build-graph.duckdb" \
                -c "SET VARIABLE raw_dir = '$out_dir';" \
                -f ${loadDuckdb}

              echo "Rendering review artifacts..."
              python3 ${render} "$raw_dir" "$out_dir/graph-viz"

              echo ""
              echo "Build-graph report written to $out_dir:"
              echo "  nodes.ndjson, edges.ndjson, root_membership.ndjson"
              echo "  build-graph.duckdb"
              echo "  graph-viz/ (rendered DAGs and ego cones)"
              echo "Canned review queries: ${./build-graph-report/queries.sql}"
              echo "Query interactively: duckdb $out_dir/build-graph.duckdb"
            '';
          }
        );
      };
    };
}
