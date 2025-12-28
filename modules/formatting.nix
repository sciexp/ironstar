{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
    (inputs.git-hooks + /flake-module.nix)
  ];

  perSystem =
    { pkgs, ... }:
    {
      treefmt = {
        projectRootFile = "flake.nix";

        # Nix formatting
        programs.nixfmt.enable = true;

        # Rust formatting (uses rustfmt from rust-toolchain.toml)
        programs.rustfmt.enable = true;

        # TypeScript/JavaScript/JSON via biome
        programs.biome = {
          enable = true;
          includes = [
            "*.ts"
            "*.tsx"
            "*.js"
            "*.jsx"
            "*.json"
            "*.astro"
          ];
        };
      };

      pre-commit.settings = {
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
