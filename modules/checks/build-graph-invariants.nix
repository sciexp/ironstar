# Graph-drift regulator: assert the committed build-graph snapshot stays within
# the accepted ceilings.
#
# CCV framing: build-graph-snapshot.json is the operating ENVELOPE (the realized
# build-graph shape, generated outside the sandbox by `nix run
# .#build-graph-snapshot` because a flake check cannot invoke recursive nix);
# build-graph-baseline.json holds the accepted ceilings; this check is the
# REGULATOR that fails when the snapshot leaves the envelope. It is a sibling of
# cargo-nix-lock-sync: that one gates the crate SET (every Cargo.lock package
# appears in Cargo.nix), this one gates the crate-set's DUPLICATION STRUCTURE and
# workspace-member presence, which the set-level check cannot see.
#
# Inputs are content-addressed on exactly the snapshot + baseline via
# builtins.path with fixed names (the same discipline as cargo-nix-lock-sync), so
# this check rebuilds only when those two files change, never on store-hash churn
# or unrelated commits. No IFD and no recursive nix: both files are read as plain
# store paths and validated with python3.
#
# Ceilings are upper bounds: growth fails, planned shrinkage (the dev/release and
# per-member duplication, the vendor monolith) passes. Member presence is an exact
# floor. nixpkgs stdenv-stage bootstrap variance is intentionally not gated.
{ self, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      snapshot = builtins.path {
        path = self + "/modules/checks/build-graph-snapshot.json";
        name = "build-graph-invariants-snapshot";
      };
      baseline = builtins.path {
        path = self + "/modules/checks/build-graph-baseline.json";
        name = "build-graph-invariants-baseline";
      };
      validator = ./build-graph-invariants.py;
    in
    {
      checks.build-graph-invariants =
        pkgs.runCommand "build-graph-invariants"
          {
            nativeBuildInputs = [ pkgs.python3 ];
            inherit snapshot baseline;
          }
          ''
            python3 ${validator} "$snapshot" "$baseline"
            touch $out
          '';
    };
}
