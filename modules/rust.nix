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
      # Pure crane pattern: single commonArgs shared by all derivations.
      # See: nix-cargo-crane/docs/faq/constant-rebuilds.md
      commonArgs = {
        inherit src;
        pname = "ironstar";
        strictDeps = true;
        # Use dev profile for faster compilation during development.
        # Release builds use [profile.release] from Cargo.toml (strip, lto, opt-level=z).
        CARGO_PROFILE = "dev";
        # System duckdb for dev builds (avoids libduckdb-sys C++ compilation).
        # For production bundled builds, remove duckdb and re-enable "bundled" feature in Cargo.toml.
        buildInputs = [ pkgs.duckdb ];
        nativeBuildInputs = [ pkgs.pkg-config ];
      };

      # Single cargoArtifacts derivation shared by all crane outputs
      # Note: buildDepsOnly automatically appends "-deps" suffix to pname
      cargoArtifacts = crane-lib.buildDepsOnly commonArgs;
    in
    {
      # Configure rust-project toolchain (no per-crate defaults needed)
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
      };

      # Manual wiring: packages
      packages = {
        default = self'.packages.ironstar;
        ironstar = crane-lib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      };

      # Manual wiring: checks
      # Note: crane functions auto-append suffixes (-fmt, -nextest, -clippy)
      checks = {
        workspace-fmt = crane-lib.cargoFmt {
          inherit src;
          pname = "ironstar";
        };

        workspace-test = crane-lib.cargoNextest (
          commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            # Allow empty test suite during early development
            cargoNextestExtraArgs = "--no-tests=pass";
          }
        );

        workspace-clippy = crane-lib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }
        );

        # Doctests disabled: examples as integration tests in crates/*/tests/
        # See CLAUDE.md "Testing conventions" for rationale
        # rust-doctest = crane-lib.cargoDocTest (
        #   commonArgs // { inherit cargoArtifacts; }
        # );
      };
    };
}
