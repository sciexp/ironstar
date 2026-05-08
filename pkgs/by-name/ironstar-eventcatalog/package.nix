{
  inputs,
  lib,
  stdenv,
  nodejs,
  ...
}:
let
  bun2nix = inputs.bun2nix.packages.${stdenv.system}.default;
  playwrightDriver = inputs.playwright-web-flake.packages.${stdenv.system}.playwright-driver;
in
stdenv.mkDerivation (finalAttrs: {
  pname = "ironstar-eventcatalog";
  version = "0.0.0-development";

  src = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../package.json
      ../../../bun.lock
      (lib.fileset.maybeMissing ../../../patches)
      ../../../packages/eventcatalog
    ];
  };

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

  # Install layout: payload is the flat dist/ tree at $out (top-level _astro/,
  # *.html, ...). A normalized wrangler.json is emitted alongside it with
  # paths rewritten to match the flat layout: ./dist references in
  # packages/eventcatalog/wrangler.jsonc become root-relative (".") so
  # `wrangler --config $out/wrangler.json` resolves all artefacts under the
  # same prefix. apps.deploy-sites consumes $out as the materialise source
  # (see modules/apps/deploy-sites/).
  #
  # node is used (over jq) to parse the source wrangler.jsonc because jq's
  # strict-JSON parser rejects // and /* */ comments. The inline JSONC
  # stripper handles block comments, line comments, and trailing commas.
  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -R packages/eventcatalog/dist/* $out/
    ${nodejs}/bin/node -e '
      const fs = require("fs");
      const src = fs.readFileSync("packages/eventcatalog/wrangler.jsonc", "utf8");
      const stripped = src
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/(^|[^:])\/\/.*$/gm, "$1")
        .replace(/,(\s*[}\]])/g, "$1");
      const cfg = JSON.parse(stripped);
      cfg.assets = cfg.assets || {};
      cfg.assets.directory = ".";
      fs.writeFileSync(process.env.out + "/wrangler.json", JSON.stringify(cfg, null, 2) + "\n");
    '
    runHook postInstall
  '';

  passthru.tests.unit = stdenv.mkDerivation {
    pname = "ironstar-eventcatalog-unit";
    version = finalAttrs.version;
    inherit (finalAttrs) src;

    nativeBuildInputs = [
      bun2nix.hook
    ];

    bunDeps = finalAttrs.bunDeps;
    dontUseBunBuild = true;
    dontUseBunInstall = true;
    dontRunLifecycleScripts = true;

    # Match the main derivation's linker mode so hoisted
    # node_modules resolve correctly for vitest
    bunInstallFlags = finalAttrs.bunInstallFlags;

    buildPhase = ''
      runHook preBuild
      cd packages/eventcatalog
      bun run test:unit
      cd ../..
      runHook postBuild
    '';

    installPhase = ''
      touch $out
    '';

    meta = {
      description = "Vitest unit tests for ironstar-eventcatalog";
    };
  };

  passthru.tests.e2e = stdenv.mkDerivation {
    pname = "ironstar-eventcatalog-e2e";
    version = finalAttrs.version;
    inherit (finalAttrs) src;

    nativeBuildInputs = [
      bun2nix.hook
      nodejs
    ];

    bunDeps = finalAttrs.bunDeps;
    dontUseBunBuild = true;
    dontUseBunInstall = true;
    dontRunLifecycleScripts = true;
    __darwinAllowLocalNetworking = true;

    # Match the main derivation's linker mode so hoisted
    # node_modules resolve correctly for Playwright
    bunInstallFlags = finalAttrs.bunInstallFlags;

    env = {
      CI = "true";
      PLAYWRIGHT_BROWSERS_PATH = "${playwrightDriver.browsers}";
      PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
      PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
    };

    buildPhase = ''
      runHook preBuild
      mkdir -p packages/eventcatalog/dist
      cp -r ${finalAttrs.finalPackage}/* packages/eventcatalog/dist/

      # Add workspace root node_modules/.bin to PATH for serve binary
      export PATH="$PWD/node_modules/.bin:$PATH"

      cd packages/eventcatalog
      # Run Playwright via node directly — bun's child_process.fork() IPC
      # is incompatible with Playwright's worker model
      ${nodejs}/bin/node ../../node_modules/@playwright/test/cli.js test
      cd ../..
      runHook postBuild
    '';

    installPhase = ''
      touch $out
    '';

    meta = {
      description = "Playwright E2E tests for ironstar-eventcatalog";
    };
  };

  meta = {
    description = "Ironstar EventCatalog site for event-driven architecture documentation";
    license = lib.licenses.mit;
  };
})
