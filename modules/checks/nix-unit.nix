{ inputs, self, ... }:
{
  perSystem =
    { system, ... }:
    {
      # All flake inputs must be explicitly passed to nix-unit so it can evaluate
      # tests without network access. Without this, nix-unit runs in a sandbox
      # and fails with SSL/network errors when trying to fetch inputs.
      # All flake inputs must be explicitly passed to nix-unit so it can evaluate
      # tests without network access.
      nix-unit.inputs = {
        inherit (inputs)
          nixpkgs
          systems
          flake-parts
          treefmt-nix
          import-tree
          git-hooks
          rust-overlay
          crane
          bun2nix
          pkgs-by-name-for-flake-parts
          process-compose-flake
          services-flake
          playwright-web-flake
          nix-unit
          ;
        inherit self;
      };

      nix-unit.tests = {
        # Metadata Tests

        # TC-001: Flake Structure Smoke Test
        # Validates flake has required top-level outputs
        # Note: templates is optional (only present when nix-template param is true)
        testMetadataFlakeOutputsExist = {
          expr = (builtins.hasAttr "devShells" self) && (builtins.hasAttr "checks" self);
          expected = true;
        };

        # System-Specific Tests

        # TC-002: System Packages Exist
        # Validates system-specific devShells output exists
        testSystemDevShellsExist = {
          expr = builtins.hasAttr system self.devShells;
          expected = true;
        };

        # TC-003: Default DevShell Exists
        # Validates default devShell is accessible for current system
        testDefaultDevShellExists = {
          expr = builtins.hasAttr "default" self.devShells.${system};
          expected = true;
        };

        # TC-004: System Checks Exist
        # Validates system-specific checks output exists
        testSystemChecksExist = {
          expr = builtins.hasAttr system self.checks;
          expected = true;
        };

        # Feature Tests

        # TC-005: Formatter Available
        # Validates formatter is configured for current system
        testFormatterExists = {
          expr = builtins.hasAttr system self.formatter;
          expected = true;
        };

        # Relational Invariant Tests

        # TC-006: Every package has a corresponding check
        # Validates that all packages (minus explicit exclusions) appear in checks.
        # Exclusions: default (alias), ironstar-release (expensive opt-in),
        # nix-fast-build (passthrough input), frontendAssets (build intermediate),
        # per-crate *-test (redundant with workspace-test),
        # per-crate *-clippy (redundant with workspace-clippy).
        testPackagesHaveChecks =
          let
            packages = self.packages.${system};
            checks = self.checks.${system};
            excluded = [
              "default"
              "ironstar-release"
              "nix-fast-build"
              "frontendAssets"
            ];
            isPerCrateSuffix =
              name: (builtins.match ".*-test$" name != null) || (builtins.match ".*-clippy$" name != null);
            packageNames = builtins.attrNames packages;
            relevantPackages = builtins.filter (
              name: !(builtins.elem name excluded) && !(isPerCrateSuffix name)
            ) packageNames;
          in
          {
            expr = builtins.all (name: builtins.hasAttr name checks) relevantPackages;
            expected = true;
          };
      };
    };
}
