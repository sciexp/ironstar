{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
    inputs.git-hooks.flakeModule
  ];

  perSystem =
    { config, pkgs, ... }:
    {
      treefmt = {
        projectRootFile = "flake.nix";

        # Nix formatting
        programs.nixfmt.enable = true;

        # Rust formatting - use rustfmt from project toolchain for version consistency
        # This ensures treefmt uses the same rustfmt as cargo fmt (from rust-toolchain.toml)
        # NOTE: edition must match workspace.package.edition in Cargo.toml
        # treefmt-nix doesn't support auto-detection from Cargo.toml (always passes --edition)
        programs.rustfmt = {
          enable = true;
          package = config.ironstar.rustToolchain;
          edition = "2024";
        };

        # TypeScript/JavaScript/JSON via biome
        programs.biome = {
          enable = true;
          includes = [
            # packages: TS/JS/JSON/Astro source
            "packages/**/*.ts"
            "packages/**/*.tsx"
            "packages/**/*.js"
            "packages/**/*.jsx"
            "packages/**/*.json"
            "packages/**/*.astro"
            # web-components: TS/JS/JSON source
            "web-components/**/*.ts"
            "web-components/**/*.js"
            "web-components/**/*.json"
            # root and CI config
            "package.json"
            ".github/**/*.json"
          ];
          excludes = [
            ".github/renovate.json"
            "web-components/vendor/*"
          ];
        };
      };

      pre-commit.settings = {
        package = pkgs.prek;
        hooks.treefmt.enable = true;
        hooks.gitleaks = {
          enable = true;
          name = "gitleaks";
          entry = "${pkgs.gitleaks}/bin/gitleaks protect --staged --verbose --redact";
          language = "system";
          pass_filenames = false;
        };
      };
    };
}
