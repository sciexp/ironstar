{
  inputs,
  lib,
  nodejs-slim,
  stdenv,
  jq,
  autoPatchelfHook,
  playwright-browsers-nixpkgs,
  ...
}:
let
  bun2nix = inputs.bun2nix.packages.${stdenv.system}.default;

  # Linux nix-build sandbox rejects the prebuilt @cloudflare/workerd-linux-64
  # glibc ELF (PT_INTERP=/lib64/ld-linux-x86-64.so.2 is absent). Rewrite
  # PT_INTERP + RUNPATH against stdenv glibc + libstdc++. Both `astro build`
  # (via @cloudflare/vite-plugin loading miniflare) and `astro preview`
  # (miniflare's webServer) spawn this binary.
  patchBundledWorkerd = lib.optionalString stdenv.isLinux ''
    shopt -s nullglob
    for binary in node_modules/.bun/@cloudflare+workerd-linux-64@*/node_modules/@cloudflare/workerd-linux-64/bin/workerd; do
      # bun's isolated linker ships files r-xr-xr-x; patchelf needs u+w.
      chmod u+w "$binary"
      autoPatchelf "$binary"
    done
  '';
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
      (lib.fileset.maybeMissing ../../../patches)
      ../../../packages/docs
    ];
  };

  nativeBuildInputs = [
    bun2nix.hook
    nodejs-slim
    jq
  ]
  ++ lib.optionals stdenv.isLinux [ autoPatchelfHook ];

  buildInputs = lib.optionals stdenv.isLinux [ stdenv.cc.cc.lib ];

  # Patch the bundled workerd binary manually in buildPhase; $out has no ELFs.
  dontAutoPatchelf = true;

  bunDeps = bun2nix.fetchBunDeps {
    bunNix = ../../../bun.nix;
  };

  dontUseBunBuild = true;
  dontUseBunInstall = true;

  # Skip miniflare's external fetch to workers.cloudflare.com/cf.json during
  # astro build; the placeholder fallback is sufficient for the prerender pass
  # and avoids a TLS warning in hermetic (no-CA-bundle) sandbox builds.
  env.CLOUDFLARE_CF_FETCH_ENABLED = "false";

  buildPhase = ''
    runHook preBuild
    ${patchBundledWorkerd}
    cd packages/docs
    # Use node (not bun) to invoke astro: bun's incomplete `ws` shim causes the
    # @cloudflare/vite-plugin module-init path to hang in the nix build env
    # (cold cache triggers warning-emitting branch that warm host-cache skips).
    node ./node_modules/.bin/astro build
    cd ../..
    runHook postBuild
  '';

  # Ship the @astrojs/cloudflare adapter output verbatim. The deploy-authoritative
  # config is dist/server/wrangler.json emitted by the adapter; the source
  # wrangler.jsonc is preserved alongside for reference (its main field is a
  # module specifier resolved at build time, not a deploy artefact path).
  # apps.deploy-sites consumes $out and points wrangler at $out/dist/server/wrangler.json.
  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -R packages/docs/dist $out/dist
    cp -R packages/docs/.wrangler $out/.wrangler
    cp packages/docs/wrangler.jsonc $out/wrangler.jsonc
    # Reproducibility: rewrite sandbox-specific absolute paths in the emitted
    # wrangler.json to stable relative values. These fields are never dereferenced
    # downstream — wrangler.unstable_readConfig rederives them from the file path
    # it is handed — but stripping the build-time /nix/var/nix/builds/... strings
    # makes the derivation output bit-identical across rebuilds.
    jq '.configPath = "./wrangler.jsonc" | .userConfigPath = "./wrangler.jsonc"' \
      $out/dist/server/wrangler.json > $out/dist/server/wrangler.json.tmp
    mv $out/dist/server/wrangler.json.tmp $out/dist/server/wrangler.json
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
    ]
    ++ lib.optionals stdenv.isLinux [ autoPatchelfHook ];

    buildInputs = lib.optionals stdenv.isLinux [ stdenv.cc.cc.lib ];

    dontAutoPatchelf = true;

    bunDeps = finalAttrs.bunDeps;
    dontUseBunBuild = true;
    dontUseBunInstall = true;
    dontRunLifecycleScripts = true;
    __darwinAllowLocalNetworking = true;

    env = {
      CI = "true";
      CLOUDFLARE_CF_FETCH_ENABLED = "false";
      PLAYWRIGHT_BROWSERS_PATH = "${playwright-browsers-nixpkgs}";
      PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
      PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
    };

    buildPhase = ''
      runHook preBuild
      ${patchBundledWorkerd}
      # Provide pre-built CF Worker bundle for the CI webServer (astro preview →
      # miniflare/workerd). finalPackage has a nested layout ({dist/client/,
      # dist/server/, .wrangler/, wrangler.jsonc}); astro preview reads
      # .wrangler/deploy/config.json to locate dist/server/wrangler.json via
      # @cloudflare/vite-plugin's getWorkerConfigs().
      mkdir -p packages/docs
      cp -r ${finalAttrs.finalPackage}/dist packages/docs/dist
      cp -r ${finalAttrs.finalPackage}/.wrangler packages/docs/.wrangler
      chmod -R u+w packages/docs/dist packages/docs/.wrangler

      cd packages/docs
      # Run Playwright via node — bun's child_process.fork() IPC
      # is incompatible with Playwright's worker model.
      # CI=true: chromium-only projects, playwright manages webServer lifecycle via
      # playwright.config webServer command (bun run preview:ci → astro preview).
      ${nodejs-slim}/bin/node ./node_modules/@playwright/test/cli.js test
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
