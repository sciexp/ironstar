# Structural invariant: every relevant package has a corresponding check.
#
# This carries forward the relational invariant previously expressed as
# TC-006 in modules/checks/nix-unit.nix (testPackagesHaveChecks). The
# remaining nix-unit smoke tests (TC-001..TC-005) were tautologies
# guaranteed by import-tree and the flake-parts module type system; only
# the package-set <-> check-set coupling carries non-trivial semantic
# content and is preserved here.
#
# Implemented as runCommand JSON-diff (via flake.lib.mkStructuralCheck)
# rather than nix-unit because the assertion target is a pure
# attribute-name list computed at outer eval time. nix-unit's
# expression-evaluation harness adds no value over `diff -u` here, while
# imposing a network-isolation constraint that was the proximate cause of
# the obh.10 hercules-ci-effects input-discipline failure.
#
# Exclusions (carried forward from TC-006, plus deps build intermediates):
#   - default: alias for ironstar
#   - ironstar-release: expensive opt-in, not a check target
#   - nix-fast-build: passthrough input, not a build artifact
#   - frontendAssets: build intermediate, exercised via ironstar
#   - ironstar-docs-deps: build intermediate, exercised via ironstar-docs
#   - ironstar-eventcatalog-deps: build intermediate, exercised via
#       ironstar-eventcatalog
#   - per-crate *-test: redundant with workspace-test
#   - per-crate *-clippy: redundant with workspace-clippy
#
# Failure mode: the diff shows package names present in `actual` but
# absent from `expected` (i.e., packages that lack a corresponding check).
#
# Pattern adapted from vanixiets commit 467c538ed
# (modules/checks/structure/flake-shape.nix).
{ self, lib, ... }:
{
  perSystem =
    {
      pkgs,
      system,
      ...
    }:
    let
      mkCheck = self.lib.mkStructuralCheck pkgs;
      excluded = [
        "default"
        "ironstar-release"
        "nix-fast-build"
        "frontendAssets"
        "ironstar-docs-deps"
        "ironstar-eventcatalog-deps"
      ];
      isPerCrateSuffix =
        name: (builtins.match ".*-test$" name != null) || (builtins.match ".*-clippy$" name != null);
      packages = self.packages.${system};
      checks = self.checks.${system};
      relevantPackageNames = lib.naturalSort (
        builtins.filter (name: !(builtins.elem name excluded) && !(isPerCrateSuffix name)) (
          builtins.attrNames packages
        )
      );
      packagesWithChecks = builtins.filter (name: builtins.hasAttr name checks) relevantPackageNames;
    in
    {
      checks.structure-package-set-invariant = mkCheck {
        name = "package-set-invariant";
        actual = relevantPackageNames;
        expected = packagesWithChecks;
      };
    };
}
