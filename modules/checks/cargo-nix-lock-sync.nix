# Staleness regulator: assert the committed Cargo.nix is in lockstep with Cargo.lock.
#
# A pure, no-network derivation that derives the package set (name@version pairs)
# independently from Cargo.lock and from Cargo.nix and fails on any mismatch. It is
# the belt-and-suspenders for the CI auto-regen in regenerate-lock-files.yaml: a
# developer who bumps Cargo.lock locally without running `just regenerate-cargo-nix`
# (on a PR the CI trigger paths do not catch) is caught here.
#
# Inputs are content-addressed on exactly the two files via builtins.path with fixed
# names (the same discipline as modules/rust.nix's ironstar-cargo-lock pin), so this
# check rebuilds only when Cargo.lock or Cargo.nix content changes, not on unrelated
# commits. No IFD: both files are read as plain store paths, not derivation outputs.
#
# Parsing is package-scoped, not a raw `version =` line count. Cargo.lock carries one
# pre-package format header (`version = 4`) outside any [[package]] block, and a naive
# count yields 577 rather than 576; anchoring on the strictly-alternating name/version
# pairs after the first `name =` line excludes the header. Cargo.nix pairs each
# crateName with the version on its immediately-following line; crate2nix dependency
# entries use packageId (never version), so no stray version fields interleave.
{ self, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      cargoLock = builtins.path {
        path = self + "/Cargo.lock";
        name = "cargo-nix-lock-sync-cargo-lock";
      };
      cargoNix = builtins.path {
        path = self + "/Cargo.nix";
        name = "cargo-nix-lock-sync-cargo-nix";
      };
    in
    {
      checks.cargo-nix-lock-sync =
        pkgs.runCommand "cargo-nix-lock-sync"
          {
            nativeBuildInputs = [ pkgs.coreutils ];
            inherit cargoLock cargoNix;
          }
          ''
            set -euo pipefail

            # Cargo.lock: each [[package]] yields name= then version= in TOML order.
            # Drop everything before the first top-level `name =` line so the
            # pre-package `version = N` lockfile-format header is excluded, then pair
            # the strictly-alternating name/version lines.
            sed -n '/^name = /,$p' "$cargoLock" \
              | grep -E '^(name|version) = ' \
              | paste - - \
              | sed -E 's/^name = "([^"]+)"\tversion = "([^"]+)"$/\1@\2/' \
              | LC_ALL=C sort -u > lock-set.txt

            # Cargo.nix: each crateName line is immediately followed by its version
            # line, both at crate-level indentation. Dependency entries carry packageId,
            # not version, so the alternation is exact.
            grep -E '^[[:space:]]*(crateName|version) = ' "$cargoNix" \
              | sed -E 's/^[[:space:]]+//' \
              | paste - - \
              | sed -E 's/^crateName = "([^"]+)";\tversion = "([^"]+)";$/\1@\2/' \
              | LC_ALL=C sort -u > nix-set.txt

            lock_count=$(wc -l < lock-set.txt)
            nix_count=$(wc -l < nix-set.txt)
            echo "Cargo.lock package set: $lock_count entries"
            echo "Cargo.nix  crate set:  $nix_count entries"

            if ! diff -u lock-set.txt nix-set.txt > set.diff; then
              echo "" >&2
              echo "error: Cargo.nix is out of sync with Cargo.lock." >&2
              echo "" >&2
              echo "The name@version package sets differ (- only in Cargo.lock, + only in Cargo.nix):" >&2
              echo "" >&2
              sed 's/^/    /' set.diff >&2
              echo "" >&2
              echo "Run \`just regenerate-cargo-nix\` and commit the updated Cargo.nix." >&2
              exit 1
            fi

            echo "Cargo.nix is in lockstep with Cargo.lock ($lock_count packages)."
            touch $out
          '';
    };
}
