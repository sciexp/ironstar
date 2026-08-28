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

      # Cargo source filter: directories, `.rs`/`.toml` files, `Cargo.lock`, and
      # `.cargo/config`, plus the `.sql` allowlist this repo adds for include_str!
      # macros. Keeping all `.toml` files retains the cargo/nextest/clippy/taplo/
      # gitleaks configs the workspace build reads.
      cargoSourceFilter =
        path: type:
        type == "directory"
        || lib.hasSuffix ".rs" path
        || lib.hasSuffix ".toml" path
        || lib.hasSuffix ".sql" path
        || baseNameOf path == "Cargo.lock"
        || (baseNameOf (dirOf path) == ".cargo" && baseNameOf path == "config");

      # Source filtering: builtins.path with a fixed name produces a content-addressed
      # store path whose hash depends only on filtered content, not on the flake's
      # self identity, preventing full rebuilds when unrelated files change.
      #
      # The filter admits every directory unconditionally, so a directory is
      # retained even when nothing inside it survives the file predicate. Adding
      # a directory anywhere in the repository therefore changes this source and
      # invalidates the workspace gates built from it, while adding a filtered-out
      # file beside existing files does not.
      src = builtins.path {
        path = self;
        name = "ironstar-src";
        filter = cargoSourceFilter;
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
      # cargoSourceFilter drops non-cargo files, so we must explicitly include the
      # directories needed by compile-time macros:
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

      # Read version from the workspace Cargo.toml at eval time, avoiding IFD:
      # ./Cargo.toml is a plain file in the flake source, not a derivation output.
      # Single source of truth: [workspace.package] version in Cargo.toml.
      cargoTomlContents = builtins.fromTOML (builtins.readFile (self + "/Cargo.toml"));
      workspaceVersion = cargoTomlContents.workspace.package.version;

      # Workspace clippy gate (single workspace-wide cargo invocation).
      #
      # Offline vendored deps come from the canonical nixpkgs idiom
      # rustPlatform.importCargoLock + cargoSetupHook (no IFD: the lockfile is a
      # plain file read at eval time, and all 565 crates.io checksums plus 0 git
      # sources make vendoring fully offline with no outputHashes).
      #
      # The pinned 1.94.1 rustToolchain (which carries the clippy extension)
      # supplies cargo/rustc/clippy so the gate matches fmt and the devshell. The
      # native deps are the full set the single-derivation workspace compile needs:
      # cc (libduckdb-sys C++ amalgamation), cmake + perl (aws-lc-sys), perl (ring),
      # pkg-config + sqlite (libsqlite3-sys). HOME=/tmp guards any build-time
      # DuckDB extension write.
      # Pin the lockfile to a content-addressed store path of Cargo.lock alone, so
      # the vendor dir's hash tracks only the lockfile content. Reading `self +
      # "/Cargo.lock"` would tie it to the whole flake-source identity, rebuilding
      # the vendor dir on any unrelated commit.
      cargoLockSrc = builtins.path {
        path = self + "/Cargo.lock";
        name = "ironstar-cargo-lock";
      };
      # nixpkgs' importCargoLock resolves crates.io to the api/v1 download
      # endpoint, which rejects the `curl/<ver> Nix/<ver>` user agent fetchurl
      # sends and answers 403 on every crate; the static CDN serves the same
      # bytes to any user agent. Upstream fixed this by re-pointing the registry
      # default (nixpkgs c0a89c37, NixOS/nixpkgs#524985), which our pin predates.
      # That default is not exposed as a parameter, and `extraRegistries` cannot
      # stand in for it: keys there only add registries, and each one also appends
      # a `[source."<url>"]` stanza to the generated .cargo/config.toml, so naming
      # the crates.io index makes cargo abort with `crates-io` defined twice.
      # Rewriting the URL inside the fetchurl this builder receives yields
      # upstream's URLs alongside upstream's unmodified cargo config. A nixpkgs
      # bump past c0a89c37 makes the wrapper a no-op and retires it.
      # See https://github.com/rust-lang/crates.io/issues/13482
      importCargoLock = pkgs.rustPlatform.importCargoLock.override {
        fetchurl =
          args:
          pkgs.fetchurl (
            args
            // {
              url = lib.replaceStrings [ "https://crates.io/api/v1/crates/" ] [
                "https://static.crates.io/crates/"
              ] args.url;
            }
          );
      };
      cargoVendorDeps = importCargoLock {
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

      # workspace-clippy is the sole monolithic gate: a single workspace-wide cargo
      # clippy over combinedSrc with `--profile dev` for parity with the dev-profile
      # binary build. (The test gate is the 11 per-member runTests checks aggregated
      # below, not a monolithic nextest.)
      workspaceClippy = pkgs.stdenv.mkDerivation {
        pname = "ironstar-clippy";
        version = workspaceVersion;
        src = combinedSrc;
        cargoDeps = cargoVendorDeps;
        strictDeps = true;
        nativeBuildInputs = workspaceGateNativeBuildInputs;
        buildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.sqlite ];
        # libsqlite3-sys finds sqlite via pkg-config; provide the dev lib on Linux.
        HOME = "/tmp";
        # cmake's setup hook injects a configurePhase that does not apply to a
        # cargo workspace; neutralize it so cargoSetupHook's vendoring stands.
        dontUseCmakeConfigure = true;
        buildPhase = ''
          runHook preBuild
          cargo --version
          cargo clippy --profile dev --locked --all-targets -- --deny warnings
          runHook postBuild
        '';
        installPhase = ''
          runHook preInstall
          mkdir -p $out
          runHook postInstall
        '';
        doCheck = false;
      };

      # crate2nix substrate (the Rust build substrate).
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
      # rebuild them (the name="ironstar-src" stability discipline).
      ironstarMemberSrc = builtins.path {
        path = inputs.self + "/crates/ironstar";
        name = "ironstar-member-source";
        filter = cargoSourceFilter;
      };
      ironstarAnalyticsInfraMemberSrc = builtins.path {
        path = inputs.self + "/crates/ironstar-analytics-infra";
        name = "ironstar-analytics-infra-member-source";
        filter = cargoSourceFilter;
      };

      # Empty ducklake-catalogs tree: the .db files are gitignored, so
      # ironstar-analytics-infra embeds an empty catalog. rust-embed silently
      # embeds nothing when the folder is absent, so an empty directory is
      # required to materialize the intended empty embed rather than a missing one.
      ducklakeCatalogsSrc = pkgs.runCommand "ironstar-ducklake-catalogs" { } ''
        mkdir -p $out
      '';

      # ironstar member tree: crate at $out/crates/ironstar, with $out/static/dist
      # (the built frontendAssets) two levels up so
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
          # On aarch64-darwin the dev profile (-C debuginfo=2) emits a bin/ironstar.dSYM
          # directory beside the executable. The crate2nix test runner stages the real
          # binary with a non-recursive `cp ${crate}/bin/*` (templates/nix/crate2nix
          # default.nix:191-193) which aborts under set -e on that directory, so the
          # ironstar member's tests never run. buildRustCrate runs postInstall after
          # installing bin/ (extraDerivationAttrs wins the final // merge), so stripping
          # the dSYM here leaves a bare bin/ironstar and unblocks the test stage.
          postInstall = (attrs.postInstall or "") + ''
            rm -rf "$out"/bin/*.dSYM
          '';
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

      # Per-member test checks, one per workspace member.
      #
      # Each member's dev-profile crate is reused (one build serves both the binary
      # and its tests) and overridden with runTests = true, which routes through
      # crate2nix's crateWithTest: it compiles the lib unit tests and, for the
      # ironstar binary member, its tests/ integration targets (the pinned nixpkgs
      # build-rust-crate builds both), then runs each compiled test binary directly,
      # tee'ing the run log to the captured passthru.test derivation. A test failure
      # fails that derivation by construction (set -e; non-zero exit propagates), so
      # each per-member check can fail. testPreRun mirrors the gate's HOME=/tmp for
      # DuckDB extension writes. The src/workspace_member injection on ironstar and
      # ironstar-analytics-infra propagates into the test build automatically because
      # crateWithTest inherits the crate's src and crateOverrides.
      #
      # The 5 network #[ignore] tests are skipped by default (no --ignored flag).
      workspaceMemberNames = [
        "ironstar"
        "ironstar-core"
        "ironstar-shared-kernel"
        "ironstar-todo"
        "ironstar-session"
        "ironstar-analytics"
        "ironstar-workspace"
        "ironstar-event-store"
        "ironstar-event-bus"
        "ironstar-analytics-infra"
        "ironstar-session-store"
      ];
      memberTest =
        name:
        (cargoNixDev.workspaceMembers.${name}.build.override {
          runTests = true;
          testPreRun = "export HOME=/tmp";
        }).passthru.test;
      perMemberTestChecks = lib.genAttrs (map (name: "${name}-test") workspaceMemberNames) (
        checkName: memberTest (lib.removeSuffix "-test" checkName)
      );

      # Zero-build-cost aggregate preserving the workspace-test check name. A
      # linkFarm over the 11 per-member test outputs forces each to build (the run
      # logs are realized) while adding no compiler closure of its own, so the
      # devshell inputsFrom and CI ergonomics that referenced workspace-test persist
      # without the monolithic gate's recompile-everything cost.
      workspaceTestAggregate = pkgs.linkFarm "ironstar-workspace-test" (
        lib.mapAttrsToList (name: path: { inherit name path; }) perMemberTestChecks
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

          # crate2nix is the Rust build substrate: the dev build is ironstar, the
          # release build is ironstar-release.
          ironstar = cargoNixDev.workspaceMembers."ironstar".build;
          ironstar-release = cargoNixRelease.workspaceMembers."ironstar".build;
          inherit frontendAssets;
        };

        # Manual wiring: checks
        # Workspace-level checks run by `nix flake check`.
        checks = {
          inherit (self'.packages) ironstar ironstar-docs ironstar-eventcatalog;
          "dev-platform" = self'.packages.dev-platform;

          # workspace-clippy: single workspace-wide cargo clippy gate.
          workspace-clippy = workspaceClippy;

          # workspace-test: zero-cost linkFarm aggregate over the 11 per-member
          # crate2nix runTests checks. The name persists for the devshell inputsFrom
          # and CI ergonomics; the work is the finer per-member regulators below.
          workspace-test = workspaceTestAggregate;

          # Doctests disabled (doctest = false); examples live as integration tests
          # in crates/*/tests/. See CLAUDE.md "Testing conventions" for rationale.
        }
        # Per-member test checks (ironstar-core-test ... ironstar-test): finer
        # regulators at per-member granularity. Each reruns only on its own member +
        # reverse-dep cone change, delivering per-crate test cache granularity.
        # workspace-test (above) aggregates all 11 at zero extra cost.
        // perMemberTestChecks;
      };
    };
}
