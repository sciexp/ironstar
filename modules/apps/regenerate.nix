# Regenerate workspace-level generated nix files.
#
# nix run .#regenerate-bun-nix
# nix run .#regenerate-cargo-nix
{ ... }:
{
  perSystem =
    {
      inputs',
      pkgs,
      lib,
      config,
      ...
    }:
    {
      apps.regenerate-bun-nix = {
        type = "app";
        meta.description = "Regenerate bun.nix from bun.lock using the pinned bun2nix and treefmt-format the result.";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "regenerate-bun-nix";
            runtimeInputs = [
              inputs'.bun2nix.packages.default
              pkgs.git
              config.treefmt.build.wrapper
            ];
            text = ''
              repo_root=$(git rev-parse --show-toplevel)
              cd "$repo_root"
              echo "Regenerating bun.nix from bun.lock using pinned bun2nix..."
              bun2nix --lock-file ./bun.lock --output-file ./bun.nix
              echo "Formatting bun.nix with treefmt..."
              treefmt ./bun.nix
              git --no-pager diff --stat -- ./bun.nix || true
            '';
          }
        );
      };

      apps.regenerate-cargo-nix = {
        type = "app";
        meta.description = "Regenerate Cargo.nix from Cargo.lock/Cargo.toml using the flake-pinned crate2nix and treefmt-format the result.";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "regenerate-cargo-nix";
            runtimeInputs = [
              inputs'.crate2nix.packages.default
              pkgs.git
              config.treefmt.build.wrapper
            ];
            text = ''
              repo_root=$(git rev-parse --show-toplevel)
              cd "$repo_root"
              echo "Regenerating Cargo.nix from Cargo.lock using the flake-pinned crate2nix..."
              crate2nix generate
              echo "Formatting Cargo.nix with treefmt..."
              treefmt ./Cargo.nix
              git --no-pager diff --stat -- ./Cargo.nix || true
            '';
          }
        );
      };
    };
}
