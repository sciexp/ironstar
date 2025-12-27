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
      rustToolchainVersion = "1.92.0";
      inherit (config.rust-project) crane-lib src;

      # Workspace-level cargoArtifacts for tests
      # Note: buildDepsOnly automatically appends "-deps" suffix to pname
      cargoArtifacts = crane-lib.buildDepsOnly {
        inherit src;
        pname = "ironstar";
      };
    in
    {
      # Configure per-crate crane args via crate.nix files
      # Packages (ironstar, ironstar-doc) are autowired by rust-flake via autoWire in crate.nix
      rust-project = {
        crateNixFile = "crate.nix";
        toolchain = pkgs.rust-bin.stable.${rustToolchainVersion}.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
        };
      };

      # Workspace-level checks (per-crate clippy is autowired separately)
      checks = lib.mkMerge [
        {
          rust-fmt = crane-lib.cargoFmt {
            inherit src;
            pname = "ironstar-fmt";
          };

          rust-test = crane-lib.cargoNextest {
            inherit src cargoArtifacts;
            pname = "ironstar-test";
            partitions = 1;
            partitionType = "count";
          };

          rust-doctest = crane-lib.cargoDocTest {
            inherit src cargoArtifacts;
            pname = "ironstar-doctest";
          };
        }
      ];
    };
}
