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

      # Common args for consistent caching across all crane derivations.
      # Critical: must match rust-flake's autowired args exactly to share deps.
      # See: nix-cargo-crane/docs/faq/constant-rebuilds.md
      commonArgs = {
        inherit src;
        pname = "ironstar";
        strictDeps = true;
        # Use dev profile for faster compilation during development.
        # Release builds use [profile.release] from Cargo.toml (strip, lto, opt-level=z).
        CARGO_PROFILE = "dev";
        # Must match rust-flake's per-crate pattern: -p <crate-name>
        # rust-flake overrides cargoExtraArgs with "-p ironstar" in crate.nix
        cargoExtraArgs = "-p ironstar";
        # Match crate.nix nativeBuildInputs for identical derivation hash
        nativeBuildInputs = [ pkgs.pkg-config ];
      };

      # Workspace-level cargoArtifacts for tests
      # Note: buildDepsOnly automatically appends "-deps" suffix to pname
      cargoArtifacts = crane-lib.buildDepsOnly commonArgs;
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
            "rustfmt"
            "llvm-tools-preview" # Required for cargo-llvm-cov coverage
          ];
        };
        # Global defaults for all autowired crate outputs (clippy, doc, crate).
        # Note: rust-flake hardcodes cargoExtraArgs = "-p <crate>" in crate.nix,
        # so we only set attributes that aren't overridden.
        defaults.perCrate.crane.args = {
          CARGO_PROFILE = "dev";
        };
      };

      # Workspace-level checks (per-crate clippy is autowired separately)
      # Note: crane functions auto-append suffixes (-fmt, -nextest, -doctest)
      checks = lib.mkMerge [
        {
          rust-fmt = crane-lib.cargoFmt {
            inherit src;
            pname = "ironstar";
          };

          rust-test = crane-lib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              # Allow empty test suite during early development
              cargoNextestExtraArgs = "--no-tests=pass";
            }
          );

          # Doctests disabled: examples as integration tests in crates/*/tests/
          # See CLAUDE.md "Testing conventions" for rationale
          # rust-doctest = crane-lib.cargoDocTest {
          #   inherit src cargoArtifacts;
          #   pname = "ironstar";
          # };
        }
      ];
    };
}
