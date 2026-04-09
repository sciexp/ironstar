{
  lib,
  stdenv,
  fetchFromGitHub,
  fetchYarnDeps,
  yarnConfigHook,
  nodejs,
  yarn,
}:
let
  version = "0.0.0-dev";
  signozSrc = fetchFromGitHub {
    owner = "SigNoz";
    repo = "signoz";
    rev = "8bfadbc1978c3acff9777c65f6152a0ec25087b9";
    hash = "sha256-oYJykkOuxBjb5jQmlUbibIz4DmoQDCmK02BNWQJBlDQ=";
  };
  src = "${signozSrc}/frontend";
in
stdenv.mkDerivation {
  pname = "signoz-frontend";
  inherit version;
  inherit src;

  yarnOfflineCache = fetchYarnDeps {
    yarnLock = "${src}/yarn.lock";
    hash = "sha256-zV+p5lVyhVyNZYZ0rrFQHlHhBM02QNqjMXW6Ua/oxdM=";
  };

  nativeBuildInputs = [
    yarnConfigHook
    nodejs
    (yarn.override { inherit nodejs; })
  ];

  buildPhase = ''
    runHook preBuild

    export NODE_OPTIONS="--max-old-space-size=4096"

    # Run postinstall scripts that generate source files needed for the build.
    # yarnConfigHook uses --ignore-scripts, so these must run explicitly.
    # - i18n:generate-hash: creates i18n-translations-hash.json from public/locales
    # - update-registry.cjs: generates src/auto-import-registry.d.ts
    export CI=1
    node i18-generate-hash.cjs
    node scripts/update-registry.cjs

    # Set build-time environment variables.
    # VITE_FRONTEND_API_ENDPOINT uses a relative path so the frontend
    # uses the same origin as the page, suitable for reverse proxy setups.
    export VITE_FRONTEND_API_ENDPOINT="/api"

    yarn --offline build

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    mv build $out
    runHook postInstall
  '';

  # Static assets do not need fixup (no ELF binaries, no shebangs).
  dontFixup = true;

  meta = {
    description = "SigNoz frontend web application (static assets)";
    homepage = "https://github.com/SigNoz/signoz";
    license = lib.licenses.mit;
  };
}
