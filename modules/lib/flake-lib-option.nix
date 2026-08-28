# Declares `flake.lib` so that more than one module can contribute a helper.
#
# flake-parts treats an undeclared flake output attribute as unique per
# definition, so a second module defining `flake.lib.<name>` is reported as a
# conflicting definition of `flake.lib` rather than merged into it. Declaring
# the attribute as a lazy attrset restores per-attribute contribution, which
# is what `self.lib.mkStructuralCheck` and `self.lib.mkEffectRunContext`
# already assume at their use sites.
{ lib, flake-parts-lib, ... }:
{
  options.flake = flake-parts-lib.mkSubmoduleOptions {
    lib = lib.mkOption {
      type = lib.types.lazyAttrsOf lib.types.raw;
      default = { };
      description = "repository-local nix helpers, exposed as `self.lib`";
    };
  };
}
