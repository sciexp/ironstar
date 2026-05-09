{
  inputs,
  lib,
  stdenv,
  chromium,
  ffmpeg,
  makeWrapper,
  makeFontsConf,
  runCommand,
}:
# Playwright browsers wrapped from nixpkgs chromium for nix-sandbox
# compatibility.
#
# playwright-web-flake (post-PR#18) provisions chromium via pre-built
# Chrome for Testing (CFT) glibc binaries. Inside a Linux nix build
# sandbox those binaries crash on page.goto(): the browser process
# launches and accepts CDP commands (viewport set, context create) but
# the renderer dies the moment navigation begins, surfacing as
# "Target page, context or browser has been closed".
#
# On Linux, wrap nixpkgs chromium with the directory layout Playwright
# expects (chrome-linux64 + chrome-headless-shell-linux64 keyed by
# browsersJSON revision so the path matches what playwright-driver
# advertises). On darwin, CFT works fine — passthrough the original
# playwrightDriver.browsers.
#
# Consumers set PLAYWRIGHT_BROWSERS_PATH to the result of this
# derivation. Three call sites currently:
#   - pkgs/by-name/ironstar-docs/package.nix       (passthru.tests.e2e)
#   - pkgs/by-name/ironstar-eventcatalog/package.nix (passthru.tests.e2e)
#   - modules/checks/e2e.nix                        (ironstar-e2e)
let
  playwrightDriver = inputs.playwright-web-flake.packages.${stdenv.system}.playwright-driver;
in
if stdenv.isLinux then
  let
    browsersJSON = playwrightDriver.passthru.browsersJSON;
    chromiumRevision = browsersJSON.chromium.revision;
    ffmpegRevision = browsersJSON.ffmpeg.revision;
    fontconfigFile = makeFontsConf { fontDirectories = [ ]; };
    # Playwright EXECUTABLE_PATHS differ by arch.
    chromiumDir = if stdenv.hostPlatform.isx86_64 then "chrome-linux64" else "chrome-linux";
    headlessShellDir =
      if stdenv.hostPlatform.isx86_64 then
        "chrome-headless-shell-linux64"
      else
        "chrome-headless-shell-linux";
  in
  runCommand "playwright-browsers-nixpkgs"
    {
      nativeBuildInputs = [ makeWrapper ];
      passthru = { inherit playwrightDriver; };
      meta = {
        description = "Playwright browsers wrapped from nixpkgs chromium for nix-sandbox compatibility";
        platforms = lib.platforms.linux ++ lib.platforms.darwin;
      };
    }
    ''
      # Chromium
      mkdir -p $out/chromium-${chromiumRevision}/${chromiumDir}
      makeWrapper ${chromium}/bin/chromium \
        $out/chromium-${chromiumRevision}/${chromiumDir}/chrome \
        --set SSL_CERT_FILE /etc/ssl/certs/ca-bundle.crt \
        --set FONTCONFIG_FILE ${fontconfigFile}

      # Chromium headless shell
      mkdir -p $out/chromium_headless_shell-${chromiumRevision}/${headlessShellDir}
      makeWrapper ${chromium}/bin/chromium \
        $out/chromium_headless_shell-${chromiumRevision}/${headlessShellDir}/chrome-headless-shell \
        --set SSL_CERT_FILE /etc/ssl/certs/ca-bundle.crt \
        --set FONTCONFIG_FILE ${fontconfigFile}

      # ffmpeg (for video capture in e2e tests)
      mkdir -p $out/ffmpeg-${ffmpegRevision}
      ln -s ${ffmpeg}/bin/ffmpeg $out/ffmpeg-${ffmpegRevision}/ffmpeg-linux
    ''
else
  playwrightDriver.browsers
