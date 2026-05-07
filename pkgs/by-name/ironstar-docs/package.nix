{
  inputs,
  lib,
  nodejs-slim,
  stdenv,
  ...
}:
let
  bun2nix = inputs.bun2nix.packages.${stdenv.system}.default;
  playwrightDriver = inputs.playwright-web-flake.packages.${stdenv.system}.playwright-driver;
in
stdenv.mkDerivation (finalAttrs: {
  pname = "ironstar-docs";
  version = "0.0.0-development";

  src = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../package.json
      ../../../bun.lock
      ../../../tsconfig.json
      ../../../patches
      ../../../packages/docs
    ];
  };

  nativeBuildInputs = [
    bun2nix.hook
    nodejs-slim
  ];

  bunDeps = bun2nix.fetchBunDeps {
    bunNix = ../../../bun.nix;
  };

  dontUseBunBuild = true;
  dontUseBunInstall = true;

  buildPhase = ''
    runHook preBuild
    cd packages/docs
    bun run build
    cd ../..
    runHook postBuild
  '';

  # Install layout: payload is the flat dist/ tree at $out (top-level _worker.js/,
  # _astro/, _routes.json, *.html, ...). A normalized wrangler.json is emitted
  # alongside it with paths rewritten to match the flat layout: ./dist/...
  # references in packages/docs/wrangler.jsonc become root-relative (./_worker.js,
  # ".") so `wrangler --config $out/wrangler.json` resolves all artefacts under
  # the same prefix. apps.deploy-sites consumes $out as the materialise source
  # (see modules/apps/deploy-sites/).
  #
  # node is used (over jq) to parse the source wrangler.jsonc because jq's
  # strict-JSON parser rejects // and /* */ comments. The inline JSONC
  # stripper handles the three forms present in the source: block comments,
  # line comments, and trailing commas.
  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -R packages/docs/dist/* $out/
    node -e '
      const fs = require("fs");
      const src = fs.readFileSync("packages/docs/wrangler.jsonc", "utf8");
      const stripped = src
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/(^|[^:])\/\/.*$/gm, "$1")
        .replace(/,(\s*[}\]])/g, "$1");
      const cfg = JSON.parse(stripped);
      cfg.main = "./_worker.js/index.js";
      cfg.assets = cfg.assets || {};
      cfg.assets.directory = ".";
      fs.writeFileSync(process.env.out + "/wrangler.json", JSON.stringify(cfg, null, 2) + "\n");
    '
    runHook postInstall
  '';

  passthru.tests.unit = stdenv.mkDerivation {
    pname = "ironstar-docs-unit";
    version = finalAttrs.version;
    inherit (finalAttrs) src;

    nativeBuildInputs = [
      bun2nix.hook
    ];

    bunDeps = finalAttrs.bunDeps;
    dontUseBunBuild = true;
    dontUseBunInstall = true;
    dontRunLifecycleScripts = true;

    buildPhase = ''
      runHook preBuild
      cd packages/docs
      bun run test:unit
      cd ../..
      runHook postBuild
    '';

    installPhase = ''
      touch $out
    '';

    meta = {
      description = "Vitest unit tests for ironstar-docs";
    };
  };

  passthru.tests.e2e = stdenv.mkDerivation {
    pname = "ironstar-docs-e2e";
    version = finalAttrs.version;
    inherit (finalAttrs) src;

    nativeBuildInputs = [
      bun2nix.hook
      nodejs-slim
    ];

    bunDeps = finalAttrs.bunDeps;
    dontUseBunBuild = true;
    dontUseBunInstall = true;
    dontRunLifecycleScripts = true;
    __darwinAllowLocalNetworking = true;

    env = {
      CI = "true";
      PLAYWRIGHT_BROWSERS_PATH = "${playwrightDriver.browsers}";
      PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
      PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
    };

    buildPhase = ''
      runHook preBuild
      mkdir -p packages/docs/dist
      cp -r ${finalAttrs.finalPackage}/* packages/docs/dist/

      # Add workspace root node_modules/.bin to PATH for serve binary
      export PATH="$PWD/node_modules/.bin:$PATH"

      cd packages/docs
      # Run Playwright via node directly — bun's child_process.fork() IPC
      # is incompatible with Playwright's worker model
      ${nodejs-slim}/bin/node ../../node_modules/@playwright/test/cli.js test
      cd ../..
      runHook postBuild
    '';

    installPhase = ''
      touch $out
    '';

    meta = {
      description = "Playwright E2E tests for ironstar-docs";
    };
  };

  meta = {
    description = "Ironstar documentation site built with Astro Starlight";
    license = lib.licenses.mit;
  };
})
