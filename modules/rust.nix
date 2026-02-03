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

      # Frontend assets built from web-components/ via Rolldown.
      # Produces static/dist/ contents for rust-embed at compile time.
      frontendAssets = pkgs.stdenvNoCC.mkDerivation (finalAttrs: {
        pname = "ironstar-frontend";
        version = "0.1.0";
        src = inputs.self + "/web-components";

        nativeBuildInputs = [
          pkgs.nodejs
          pkgs.pnpmConfigHook
          pkgs.pnpm
        ];

        pnpmDeps = pkgs.fetchPnpmDeps {
          inherit (finalAttrs) pname version src;
          fetcherVersion = 2;
          hash = "sha256-Mb6rwHxXIMNissTEd/VGEM4cCQ0Gy36fBYKODcRMFp8=";
        };

        buildPhase = ''
          runHook preBuild
          # Rolldown outputs to ../static/dist relative to web-components/
          mkdir -p ../static
          pnpm build
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          cp -r ../static/dist $out
          runHook postInstall
        '';
      });

      # Combined source: Rust source + frontend assets + migrations.
      # Crane's cleanCargoSource filters non-Rust files, so we must explicitly
      # include directories needed by compile-time macros:
      # - static/dist/ for rust-embed
      # - crates/ironstar/migrations/ for include_str! in sqlx queries
      combinedSrc = pkgs.runCommand "ironstar-src" { } ''
        cp -r ${src} $out
        chmod -R u+w $out
        mkdir -p $out/static
        cp -r ${frontendAssets} $out/static/dist
        # Remove any empty migrations dir from cleaned source, then copy real migrations
        rm -rf $out/crates/ironstar/migrations
        cp -r ${inputs.self + "/crates/ironstar/migrations"} $out/crates/ironstar/
      '';

      # WORKAROUND: libduckdb-sys build.rs emits rerun-if-changed with absolute
      # OUT_DIR path. On nix, each derivation has unique sandbox path, causing
      # cargo to see the path as "missing" and rebuild libduckdb-sys (~7 min).
      # Patch the vendored crate to remove the problematic directive.
      # See: https://github.com/duckdb/duckdb-rs/issues/XXX (TODO: file upstream)
      cargoVendorDir = crane-lib.vendorCargoDeps {
        inherit src;
        overrideVendorCargoPackage =
          p: drv:
          if p.name == "libduckdb-sys" then
            drv.overrideAttrs (old: {
              postPatch = (old.postPatch or "") + ''
                substituteInPlace build.rs \
                  --replace-fail \
                  'println!("cargo:rerun-if-changed={out_dir}/{lib_name}/manifest.json");' \
                  '// Patched: absolute OUT_DIR path causes nix cross-derivation rebuild'
              '';
            })
          else
            drv;
      };

      # Common args for consistent caching across all crane derivations.
      # Pure crane pattern: single commonArgs shared by all derivations.
      # See: nix-cargo-crane/docs/faq/constant-rebuilds.md
      commonArgs = {
        src = combinedSrc; # Includes frontend assets for rust-embed
        inherit cargoVendorDir;
        pname = "ironstar";
        strictDeps = true;
        # Use dev profile for faster compilation during development.
        # Release builds use [profile.release] from Cargo.toml (strip, lto, opt-level=z).
        CARGO_PROFILE = "dev";
        # Match crate.nix nativeBuildInputs for identical derivation hash
        nativeBuildInputs = [ pkgs.pkg-config ];
      };

      # Single cargoArtifacts derivation shared by all crane outputs
      # Note: buildDepsOnly automatically appends "-deps" suffix to pname
      cargoArtifacts = crane-lib.buildDepsOnly commonArgs;

      # Release profile artifacts for optimized builds (strip, lto, opt-level=z)
      # Separate from dev to preserve fast iteration on default package
      cargoArtifactsRelease = crane-lib.buildDepsOnly (
        commonArgs
        // {
          CARGO_PROFILE = "release";
        }
      );
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
        ironstar-release = crane-lib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = cargoArtifactsRelease;
            CARGO_PROFILE = "release";
          }
        );
        # Exposed for isolated testing: `nix build .#frontendAssets`
        inherit frontendAssets;
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
