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

  meta = {
    description = "Ironstar documentation site built with Astro Starlight";
    license = lib.licenses.mit;
  };
}
