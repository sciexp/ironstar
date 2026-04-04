{
  inputs,
  lib,
  stdenv,
  nodejs,
  ...
}:
let
  bun2nix = inputs.bun2nix.packages.${stdenv.system}.default;

  src = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../package.json
      ../../../bun.lock
      ../../../patches
      ../../../packages/eventcatalog
    ];
  };
in
stdenv.mkDerivation {
  pname = "ironstar-eventcatalog";
  version = "0.0.0-development";

  inherit src;

  nativeBuildInputs = [
    bun2nix.hook
  ];

  bunDeps = bun2nix.fetchBunDeps {
    bunNix = ../../../bun.nix;
  };

  # Prevent lifecycle scripts from running patch-package
  # (not available in the sandbox and redundant for this build)
  dontRunLifecycleScripts = true;

  dontUseBunBuild = true;
  dontUseBunInstall = true;

  # EventCatalog copies from @eventcatalog/core into
  # .eventcatalog-core/ and writes catalog content into it.
  # The hoisted linker creates real file copies (not read-only
  # symlinks to the nix store) so they can be modified.
  bunInstallFlags =
    if stdenv.hostPlatform.isDarwin then
      [
        "--linker=hoisted"
        "--backend=copyfile"
      ]
    else
      [
        "--linker=hoisted"
      ];

  buildPhase = ''
    runHook preBuild

    # EventCatalog writes .eventcatalog-core/ within the source
    # tree; make it writable for the build
    chmod -R u+w .

    cd packages/eventcatalog

    # Make node_modules/.bin and Node.js available to child
    # processes (cross-env, npx, astro)
    NODE_MODULES_BIN="$(cd ../.. && pwd)/node_modules/.bin"
    export PATH="${nodejs}/bin:$NODE_MODULES_BIN:$PATH"

    # The EventCatalog build has two phases:
    #   1. catalogToAstro: copies catalog content into .eventcatalog-core/
    #   2. Astro SSG build: renders pages from .eventcatalog-core/
    #
    # Phase 2 fails under Bun because elkjs creates a Worker at
    # module evaluation time, and Bun's Worker implementation is
    # incompatible with the elk.bundled.js pattern.
    #
    # We run the full EventCatalog CLI (which handles phase 1 and
    # attempts phase 2). Phase 2 produces 0 pages because Astro
    # cannot resolve modules without a local node_modules. Then we
    # re-run the Astro build with proper module resolution.

    # Run EventCatalog CLI for catalogToAstro (phase 2 silently
    # produces 0 pages, which is fine)
    ${nodejs}/bin/node \
      ../../node_modules/@eventcatalog/core/dist/eventcatalog.js \
      build || true

    # Symlink root node_modules into .eventcatalog-core/ so Astro
    # can resolve its dependencies when building from that directory
    ln -sf "$(cd ../.. && pwd)/node_modules" .eventcatalog-core/node_modules

    # Run Astro SSG build under Node.js (proper Worker support)
    cd .eventcatalog-core
    export PROJECT_DIR="$(cd .. && pwd)"
    export CATALOG_DIR="$(pwd)"
    export ENABLE_EMBED=false
    export EVENTCATALOG_STARTER=false
    export EVENTCATALOG_SCALE=false
    ${nodejs}/bin/node ../../../node_modules/astro/astro.js build
    cd ..

    cd ../..
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -R packages/eventcatalog/dist/* $out/
    runHook postInstall
  '';
}
