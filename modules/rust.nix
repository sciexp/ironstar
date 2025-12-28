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
      # Critical: cargoExtraArgs must match between buildDepsOnly and all checks
      # to avoid rebuilding dependencies (especially expensive ones like libduckdb-sys).
      # See: nix-cargo-crane/docs/faq/constant-rebuilds.md
      commonArgs = {
        inherit src;
        pname = "ironstar";
        # Use dev profile for faster compilation during development.
        # Release builds use [profile.release] from Cargo.toml (strip, lto, opt-level=z).
        CARGO_PROFILE = "dev";
        # Consistent feature flags prevent cache invalidation.
        # --locked: use Cargo.lock exactly
        # --all-features: build all features so deps are complete
        cargoExtraArgs = "--locked --all-features";
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
        # Must match commonArgs to share cached dependencies.
        defaults.perCrate.crane.args = {
          CARGO_PROFILE = "dev";
          cargoExtraArgs = "--locked --all-features";
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
