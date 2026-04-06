{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      self',
      system,
      ...
    }:
    let
      bun2nix = inputs.bun2nix.packages.${system}.default;
      playwrightDriver = inputs.playwright-web-flake.packages.${system}.playwright-driver;
      nodejs-slim = pkgs.nodejs-slim;
    in
    {
      checks."ironstar-e2e" = pkgs.stdenv.mkDerivation {
        name = "ironstar-e2e";

        src = pkgs.lib.fileset.toSource {
          root = ../..;
          fileset = pkgs.lib.fileset.unions [
            ../../playwright.config.ts
            ../../e2e
            ../../package.json
            ../../bun.lock
            ../../tsconfig.json
            ../../patches
          ];
        };

        nativeBuildInputs = [
          bun2nix.hook
          nodejs-slim
        ];

        bunDeps = bun2nix.fetchBunDeps {
          bunNix = ../../bun.nix;
        };

        dontUseBunBuild = true;
        dontUseBunInstall = true;
        dontRunLifecycleScripts = true;
        __darwinAllowLocalNetworking = true;

        env = {
          CI = "true";
          PLAYWRIGHT_BROWSERS_PATH = "${playwrightDriver.browsers}";
          PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
          PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
          IRONSTAR_ENABLE_ANALYTICS = "false";
          IRONSTAR_ENABLE_ZENOH = "true";
          IRONSTAR_PORT = "3000";
        };

        buildPhase = ''
          runHook preBuild

          export HOME=$TMPDIR
          export PATH="$PWD/node_modules/.bin:$PATH"

          # Server env: writable database in sandbox tmpdir
          export IRONSTAR_DATABASE_URL="sqlite:$TMPDIR/ironstar.db?mode=rwc"
          export IRONSTAR_BINARY="${self'.packages.ironstar}/bin/ironstar"

          # Run Playwright via node — bun's fork() IPC is incompatible
          # with Playwright's worker model.
          ${nodejs-slim}/bin/node node_modules/@playwright/test/cli.js test

          runHook postBuild
        '';

        installPhase = ''
          touch $out
        '';

        meta = {
          description = "Playwright E2E tests for ironstar application";
        };
      };
    };
}
