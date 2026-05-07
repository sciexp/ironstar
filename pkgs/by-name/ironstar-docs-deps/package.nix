{
  inputs,
  lib,
  stdenv,
  ...
}:
let
  bun2nix = inputs.bun2nix.packages.${stdenv.system}.default;
in
stdenv.mkDerivation {
  pname = "ironstar-docs-deps";
  version = "0.0.0";

  src = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../package.json
      ../../../bun.lock
      ../../../bun.nix
      ../../../packages/docs/package.json
    ];
  };

  nativeBuildInputs = [ bun2nix.hook ];

  bunDeps = bun2nix.fetchBunDeps {
    bunNix = ../../../bun.nix;
  };

  # Skip lifecycle (postinstall) scripts. wrangler and the @cloudflare/* tooling
  # consumed at runtime are pure JS — no native build is required for the
  # node_modules tree this derivation materialises.
  dontRunLifecycleScripts = true;

  # The bun2nix hook materialises node_modules via bunNodeModulesInstallPhase
  # using the default --linker=isolated layout. That layout places real packages
  # under the monorepo-root node_modules/.bun/ and links direct deps (including
  # workspace-visible dev deps) into the monorepo-root node_modules/ via relative
  # symlinks. Preserving the symlink structure (plain cp -R) keeps Node.js module
  # resolution intact for tools invoked via node_modules/.bin/.
  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    runHook preInstall
    if [ ! -d packages/docs/node_modules ]; then
      echo "error: packages/docs/node_modules not populated by bun install; aborting" >&2
      exit 1
    fi
    mkdir -p $out/packages/docs
    cp -R node_modules $out/node_modules
    cp -R packages/docs/node_modules $out/packages/docs/node_modules
    runHook postInstall
  '';

  meta = {
    description = "Hermetic node_modules tree for packages/docs (wrangler runtime)";
    license = lib.licenses.mit;
  };
}
