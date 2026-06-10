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
      rustToolchainVersion = "1.94.1";

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

      # Source filtering: builtins.path with a fixed name produces a content-addressed
      # store path whose hash depends only on filtered content, not on the flake's
      # self identity, preventing full rebuilds when unrelated files change.
      src = builtins.path {
        path = self;
        name = "ironstar-src";
        filter =
          path: type:
          # Include SQL files for include_str! macros
          (lib.hasSuffix ".sql" path)
          ||
            # Default crane filter for Rust files
            (crane-lib.filterCargoSources path type);
      };

      # Content-addressed source paths for derivation hash stability.
      # builtins.path produces store paths that only change when actual
      # content changes, preventing full rebuilds on unrelated commits.
      webComponentsSrc = builtins.path {
        path = inputs.self + "/web-components";
        name = "web-components-source";
      };
      migrationsSrc = builtins.path {
        path = inputs.self + "/crates/ironstar/migrations";
        name = "ironstar-migrations";
      };

      # Frontend assets built from web-components/ via Rolldown.
      # Produces static/dist/ contents for rust-embed at compile time.
      frontendAssets = pkgs.stdenvNoCC.mkDerivation (finalAttrs: {
        pname = "ironstar-frontend";
        version = "0.1.0";
        src = webComponentsSrc;

        nativeBuildInputs = [
          pkgs.nodejs
          pkgs.pnpmConfigHook
          pkgs.pnpm
        ];

        pnpmDeps = pkgs.fetchPnpmDeps {
          inherit (finalAttrs) pname version src;
          fetcherVersion = 2;
          hash = "sha256-FFY/QxR6Bd3nSN2pc401R+1JAGX57VqCgN0lFXIEv8Q=";
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
        cp -r ${migrationsSrc} $out/crates/ironstar/migrations
      '';

      cargoVendorDir = crane-lib.vendorCargoDeps { inherit src; };

      # Read version from the workspace Cargo.toml at eval time.
      # This avoids IFD: crane reads ./Cargo.toml (a plain file in the flake source)
      # instead of inferring from combinedSrc (a runCommand derivation).
      # Single source of truth: [workspace.package] version in Cargo.toml.
      # pname is set explicitly because workspace root Cargo.toml has no [package] name.
      cargoTomlContents = builtins.fromTOML (builtins.readFile (self + "/Cargo.toml"));
      workspaceVersion = cargoTomlContents.workspace.package.version;

      # Common args for consistent caching across all crane derivations.
      # Pure crane pattern: single commonArgs shared by all derivations.
      # See: nix-cargo-crane/docs/faq/constant-rebuilds.md
      commonArgs = {
        src = combinedSrc;
        inherit cargoVendorDir;
        pname = "ironstar";
        version = workspaceVersion;
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

      # Crane-free workspace test and clippy gate.
      #
      # The two correctness regulators (workspace-test, workspace-clippy) run one
      # workspace-wide cargo invocation each, replicating the crane check's exact
      # commands without depending on crane. Offline vendored deps come from the
      # canonical nixpkgs idiom rustPlatform.importCargoLock + cargoSetupHook,
      # reading the same ./Cargo.lock crane vendors today (no IFD: the lockfile is
      # a plain file read at eval time, and all 565 crates.io checksums plus 0 git
      # sources make vendoring fully offline with no outputHashes).
      #
      # The pinned 1.94.1 rustToolchain (which carries the clippy extension)
      # supplies cargo/rustc/clippy so the gate matches fmt and the devshell. The
      # native deps are the full set the single-derivation workspace compile needs:
      # cc (libduckdb-sys C++ amalgamation), cmake + perl (aws-lc-sys), perl (ring),
      # pkg-config + sqlite (libsqlite3-sys). HOME=/tmp guards any build-time
      # DuckDB extension write, exactly as crane's commonArgs HOME=/tmp does.
      # Pin the lockfile to a content-addressed store path of Cargo.lock alone, so
      # the vendor dir's hash tracks only the lockfile content. Reading `self +
      # "/Cargo.lock"` would tie it to the whole flake-source identity, rebuilding
      # the vendor dir on any unrelated commit (the content-addressed name stability
      # risk in design.md). This mirrors crane's content-addressed src discipline.
      cargoLockSrc = builtins.path {
        path = self + "/Cargo.lock";
        name = "ironstar-cargo-lock";
      };
      cargoVendorDeps = pkgs.rustPlatform.importCargoLock {
        lockFile = cargoLockSrc;
      };

      workspaceGateNativeBuildInputs = [
        rustToolchain
        pkgs.rustPlatform.cargoSetupHook
        pkgs.stdenv.cc
        pkgs.cmake
        pkgs.perl
        pkgs.pkg-config
      ];

      # The crane gate set CARGO_PROFILE=dev in commonArgs, so nextest ran with
      # `--cargo-profile dev` and clippy with `--profile dev` (cargoWithProfile).
      # The default nextest test profile is preserved (no `.config/nextest.toml`
      # `--profile ci`), matching the crane check exactly per the design's
      # default-profile-for-parity ruling. Ignored tests are NOT run (the crane
      # check uses no `--run-ignored`), so network/ignored tests are skipped here
      # exactly as today.
      mkWorkspaceGate =
        {
          pnameSuffix,
          extraNativeBuildInputs ? [ ],
          buildCommand,
        }:
        pkgs.stdenv.mkDerivation {
          pname = "ironstar${pnameSuffix}";
          version = workspaceVersion;
          src = combinedSrc;
          cargoDeps = cargoVendorDeps;
          strictDeps = true;
          nativeBuildInputs = workspaceGateNativeBuildInputs ++ extraNativeBuildInputs;
          buildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.sqlite ];
          # libsqlite3-sys finds sqlite via pkg-config; provide the dev lib on Linux.
          HOME = "/tmp";
          # cmake's setup hook injects a configurePhase that does not apply to a
          # cargo workspace; neutralize it so cargoSetupHook's vendoring stands.
          dontUseCmakeConfigure = true;
          buildPhase = ''
            runHook preBuild
            cargo --version
            ${buildCommand}
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            mkdir -p $out
            runHook postInstall
          '';
          doCheck = false;
        };

      workspaceTest = mkWorkspaceGate {
        pnameSuffix = "-nextest";
        extraNativeBuildInputs = [ pkgs.cargo-nextest ];
        buildCommand = "cargo nextest run --cargo-profile dev --no-tests=pass";
      };

      workspaceClippy = mkWorkspaceGate {
        pnameSuffix = "-clippy";
        buildCommand = "cargo clippy --profile dev --locked --all-targets -- --deny warnings";
      };

      # crate2nix substrate (additive; crane above is untouched).
      #
      # The committed ./Cargo.nix emits one buildRustCrate derivation per crate,
      # giving per-dependency-crate nix cache granularity. It is imported with the
      # pinned 1.94.1 rustToolchain threaded into buildRustCrateForPkgs so per-crate
      # builds match fmt and the devshell, and with ironstarCrateOverrides merged
      # onto pkgs.defaultCrateOverrides.

      # Per-member derived source trees re-establishing the parent layout that the
      # two rust-embed reads require. Under crate2nix each local member's src is its
      # own crates/<name> subdir, so the `$CARGO_MANIFEST_DIR/../../` reads in
      # assets.rs and embedded_catalogs.rs would resolve above an isolated member
      # tree and silently embed nothing. These derived trees place the member at
      # $out/crates/<name> and re-materialize the sibling asset dirs two levels up.
      #
      # Fixed derivation names keep these content-addressed: a no-op commit must not
      # rebuild them (mirrors crane's name="ironstar-src" stability discipline).
      ironstarMemberSrc = builtins.path {
        path = inputs.self + "/crates/ironstar";
        name = "ironstar-member-source";
        filter = path: type: (lib.hasSuffix ".sql" path) || (crane-lib.filterCargoSources path type);
      };
      ironstarAnalyticsInfraMemberSrc = builtins.path {
        path = inputs.self + "/crates/ironstar-analytics-infra";
        name = "ironstar-analytics-infra-member-source";
        filter = path: type: (lib.hasSuffix ".sql" path) || (crane-lib.filterCargoSources path type);
      };

      # Empty ducklake-catalogs tree: today crane's combinedSrc never injects this
      # path and the .db files are gitignored, so ironstar-analytics-infra embeds an
      # empty catalog. This preserves that empty-embed status quo (design D5/Open
      # Questions orchestrator ruling); a real catalog is out of scope.
      ducklakeCatalogsSrc = pkgs.runCommand "ironstar-ducklake-catalogs" { } ''
        mkdir -p $out
      '';

      # ironstar member tree: crate at $out/crates/ironstar, with $out/static/dist
      # (the built frontendAssets, matching crane's combinedSrc) two levels up so
      # `#[folder = "$CARGO_MANIFEST_DIR/../../static/dist"]` resolves. The migrations
      # dir (crates/ironstar/migrations, include_str!) travels inside the member tree.
      ironstarSrcInjected = pkgs.runCommand "ironstar-c2n-member-src" { } ''
        mkdir -p $out/crates $out/static
        cp -r ${ironstarMemberSrc} $out/crates/ironstar
        chmod -R u+w $out/crates/ironstar
        rm -rf $out/crates/ironstar/migrations
        cp -r ${migrationsSrc} $out/crates/ironstar/migrations
        cp -r ${frontendAssets} $out/static/dist
      '';

      # ironstar-analytics-infra member tree: crate at $out/crates/<name>, with
      # $out/assets/ducklake-catalogs two levels up so
      # `#[folder = "$CARGO_MANIFEST_DIR/../../assets/ducklake-catalogs"]` resolves.
      ironstarAnalyticsInfraSrcInjected = pkgs.runCommand "ironstar-analytics-infra-c2n-member-src" { } ''
        mkdir -p $out/crates $out/assets
        cp -r ${ironstarAnalyticsInfraMemberSrc} $out/crates/ironstar-analytics-infra
        cp -r ${ducklakeCatalogsSrc} $out/assets/ducklake-catalogs
      '';

      # Per-crate overrides merged onto pkgs.defaultCrateOverrides. A crateName-keyed
      # override is applied as `crate // (override crate)` inside buildRustCrate
      # (nixpkgs build-rust-crate/default.nix line 329), so it can add native inputs,
      # set env, and override src.
      #
      # In the pinned nixpkgs there is no aws-lc-sys / ring / libduckdb-sys default
      # override, so these three are additive; libsqlite3-sys keeps the nixpkgs default
      # (pkg-config + sqlite). No workspace-wide pkg-config; no Linux openssl buildInput.
      ironstarCrateOverrides = p: {
        libduckdb-sys = attrs: {
          nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [ p.stdenv.cc ];
          # libduckdb-sys bundles a C++ amalgamation; HOME guards any build-time write.
          HOME = "/tmp";
        };
        aws-lc-sys = attrs: {
          nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [
            p.cmake
            p.perl
          ];
        };
        ring = attrs: {
          nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [ p.perl ];
        };
        # src is the full derived tree (member at crates/<name>, asset dirs two levels
        # up); workspace_member cd's buildRustCrate into the member subdir so
        # CARGO_MANIFEST_DIR is crates/<name> and `../../<asset>` resolves to the
        # re-materialized sibling dirs (configure-crate.nix lines 60-61, 160).
        ironstar = attrs: {
          src = ironstarSrcInjected;
          workspace_member = "crates/ironstar";
        };
        ironstar-analytics-infra = attrs: {
          src = ironstarAnalyticsInfraSrcInjected;
          workspace_member = "crates/ironstar-analytics-infra";
        };
      };

      # `release` is a top-level Cargo.nix argument baked into each crate config at
      # import time (crate2nix default.nix threads it into buildByPackageIdImpl; it is
      # not a `.build.override` argument), so dev and release builds need separate
      # imports. crate2nix defaults release = true, so dev passes release = false.
      mkCargoNix =
        release:
        import (self + "/Cargo.nix") {
          inherit pkgs release;
          buildRustCrateForPkgs =
            p:
            p.buildRustCrate.override {
              rustc = rustToolchain;
              cargo = rustToolchain;
              defaultCrateOverrides = p.defaultCrateOverrides // ironstarCrateOverrides p;
            };
        };
      cargoNixDev = mkCargoNix false;
      cargoNixRelease = mkCargoNix true;
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
          ironstar = crane-lib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              doCheck = false;
            }
          );
          ironstar-release = crane-lib.buildPackage (
            commonArgs
            // {
              cargoArtifacts = cargoArtifactsRelease;
              CARGO_PROFILE = "release";
              doCheck = false;
            }
          );
          inherit frontendAssets;

          # crate2nix parallel packages (additive transition; renamed to
          # ironstar/ironstar-release at the substrate swap in task 5).
          ironstar-c2n = cargoNixDev.workspaceMembers."ironstar".build;
          ironstar-release-c2n = cargoNixRelease.workspaceMembers."ironstar".build;
        };

        # Manual wiring: checks
        # Workspace-level checks run by `nix flake check`.
        checks = {
          inherit (self'.packages) ironstar ironstar-docs ironstar-eventcatalog;
          "dev-platform" = self'.packages.dev-platform;

          # Crane-free workspace gate (see mkWorkspaceGate above). Runs one
          # workspace-wide cargo nextest and one cargo clippy, replicating the
          # former crane checks' exact commands offline via importCargoLock.
          workspace-test = workspaceTest;
          workspace-clippy = workspaceClippy;

          # Doctests disabled: examples as integration tests in crates/*/tests/
          # See CLAUDE.md "Testing conventions" for rationale
          # rust-doctest = crane-lib.cargoDocTest (
          #   commonArgs // { inherit cargoArtifacts; }
          # );
        };
      };
    };
}
