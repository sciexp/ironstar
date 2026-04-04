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

  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -R packages/docs/dist/* $out/
    runHook postInstall
  '';

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
