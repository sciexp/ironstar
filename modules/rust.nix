{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];

  perSystem =
    {
      config,
      self',
      pkgs,
      lib,
      ...
    }:
    let
      inherit (config.rust-project) crane-lib src;

      # Workspace-level cargoArtifacts for tests
      cargoArtifacts = crane-lib.buildDepsOnly {
        inherit src;
        pname = "ironstar-deps";
      };
    in
    {
      # Configure per-crate crane args via crate.nix files
      rust-project = {
        crateNixFile = "crate.nix";
      };

      # Expose the main package
      packages =
        let
          inherit (config.rust-project) crates;
        in
        {
          ironstar = crates."ironstar".crane.outputs.drv.crate;
          # Don't set default here - let packages.nix handle it
        };

      # Workspace-level checks
      checks = lib.mkMerge [
        {
          rust-fmt = crane-lib.cargoFmt {
            inherit src;
            pname = "ironstar-fmt";
          };

          rust-test = crane-lib.cargoTest {
            inherit src cargoArtifacts;
            pname = "ironstar-test";
          };
        }
      ];
    };
}
