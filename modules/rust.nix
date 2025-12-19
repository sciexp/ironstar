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
      # Packages (ironstar, ironstar-doc) are autowired by rust-flake via autoWire in crate.nix
      rust-project = {
        crateNixFile = "crate.nix";
      };

      # Workspace-level checks (per-crate clippy is autowired separately)
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
