{
  inputs,
  self,
  flake-parts-lib,
  ...
}:
{
  # Inject nixpkgs module into perSystem via mkPerSystemOption.
  # This pattern (from hercules-ci/flake-parts#74) enables the nixpkgs.overlays
  # and nixpkgs.hostPlatform options inside perSystem, which rust-flake previously provided.
  options.perSystem = flake-parts-lib.mkPerSystemOption (
    { system, ... }:
    {
      imports = [
        "${inputs.nixpkgs}/nixos/modules/misc/nixpkgs.nix"
      ];
      nixpkgs = {
        hostPlatform = system;
        overlays = [
          (import inputs.rust-overlay)
        ];
      };
    }
  );

  config.perSystem =
    {
      config,
      self',
      pkgs,
      lib,
      system,
      ...
    }:
    let
      rustToolchainVersion = "1.92.0";

      # Rust toolchain via rust-overlay (replaces rust-flake config.rust-project.toolchain)
      rustToolchain = pkgs.rust-bin.stable.${rustToolchainVersion}.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
          "clippy"
          "rustfmt"
          "llvm-tools-preview"
        ];
      };

      # Crane library overridden with our toolchain (replaces config.rust-project.crane-lib)
      crane-lib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

      # Source filtering (replaces config.rust-project.src)
      src = lib.cleanSourceWith {
        src = self;
        filter =
          path: type:
          # Include SQL files for include_str! macros
          (lib.hasSuffix ".sql" path)
          ||
            # Default crane filter for Rust files
            (crane-lib.filterCargoSources path type);
      };

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

      cargoVendorDir = crane-lib.vendorCargoDeps { inherit src; };

      # Common args for consistent caching across all crane derivations.
      # Pure crane pattern: single commonArgs shared by all derivations.
      # See: nix-cargo-crane/docs/faq/constant-rebuilds.md
      commonArgs = {
        src = combinedSrc;
        inherit cargoVendorDir;
        pname = "ironstar";
        strictDeps = true;
        # Use dev profile for faster compilation during development.
        # Release builds use [profile.release] from Cargo.toml (strip, lto, opt-level=z).
        CARGO_PROFILE = "dev";
        # DuckDB INSTALL writes extensions to ~/.duckdb/extensions/. Nix sandbox
        # sets HOME=/homeless-shelter (non-writable), so tests that install
        # extensions fail. Provide a writable HOME for the build sandbox.
        HOME = "/tmp";
        nativeBuildInputs = [ pkgs.pkg-config ];
        # openssl required on Linux for TLS-dependent crates
        buildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.openssl ];
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

      # Per-crate library names for isolated test and clippy derivations.
      # All share the workspace cargoArtifacts for cache efficiency.
      libCrates = [
        "ironstar-analytics"
        "ironstar-analytics-infra"
        "ironstar-core"
        "ironstar-event-bus"
        "ironstar-event-store"
        "ironstar-session"
        "ironstar-session-store"
        "ironstar-shared-kernel"
        "ironstar-todo"
        "ironstar-workspace"
      ];

      perCrateTest =
        name:
        crane-lib.cargoNextest (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = name;
            cargoNextestExtraArgs = "-p ${name} --no-tests=pass";
          }
        );

      perCrateClippy =
        name:
        crane-lib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = name;
            cargoClippyExtraArgs = "-p ${name} --all-targets -- --deny warnings";
          }
        );
    in
    {
      options.ironstar.rustToolchain = lib.mkOption {
        type = lib.types.package;
        description = "Rust toolchain package for use by other modules";
      };

      config = {
        ironstar.rustToolchain = rustToolchain;

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
        # Workspace-level checks validate the entire codebase.
        # Per-crate checks enable isolated CI feedback and granular caching.
        checks =
          lib.genAttrs (map (n: "${n}-test") libCrates) (attr: perCrateTest (lib.removeSuffix "-test" attr))
          // lib.genAttrs (map (n: "${n}-clippy") libCrates) (
            attr: perCrateClippy (lib.removeSuffix "-clippy" attr)
          )
          // {
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
    };
}
