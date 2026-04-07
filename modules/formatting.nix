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

        # TOML formatting
        programs.taplo.enable = true;

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
            "packages/eventcatalog/domains/**/*.json"
          ];

          # Biome 2.4.9 falls back to schema 2.1.2 in treefmt-nix's version mapping,
          # causing false validation failures against the actual 2.3+ config format.
          validate.enable = false;

          # Full biome configuration as nix attrset (replaces committed biome.json).
          # The generated config is passed to biome via --config-path by treefmt-nix,
          # and symlinked to the project root via shellHook for IDE integration.
          settings = {
            files = {
              ignoreUnknown = false;
              includes = [
                "packages/*/src/**"
                "packages/*/e2e/**"
                "packages/*/tests/**"
                "packages/*/*.config.mjs"
                "packages/*/*.config.ts"
                "packages/*/package.json"
                "web-components/components/**"
                "web-components/bindings/**"
                "web-components/__tests__/**"
                "web-components/*.ts"
                "web-components/package.json"
                "web-components/tsconfig.json"
                "package.json"
                ".github/**/*.json"
              ];
            };
            formatter = {
              enabled = true;
              indentStyle = "space";
              indentWidth = 2;
              lineWidth = 120;
            };
            assist = {
              actions = {
                source = {
                  organizeImports = {
                    level = "on";
                  };
                };
              };
            };
            linter = {
              enabled = true;
              rules = {
                recommended = true;
                correctness = {
                  useExhaustiveDependencies = "info";
                };
                style = {
                  useImportType = "off";
                  noParameterAssign = "error";
                  useAsConstAssertion = "error";
                  useDefaultParameterLast = "error";
                  useEnumInitializers = "error";
                  useSelfClosingElements = "error";
                  useSingleVarDeclarator = "error";
                  noUnusedTemplateLiteral = "error";
                  useNumberNamespace = "error";
                  noInferrableTypes = "error";
                  noUselessElse = "error";
                };
              };
            };
            overrides = [
              {
                includes = [ "**/*.astro" ];
                linter = {
                  rules = {
                    correctness = {
                      noUnusedVariables = "off";
                    };
                  };
                };
              }
            ];
          };
        };
      };

      # Keep local hooks but don't inject checks.pre-commit into nix flake check —
      # treefmt and gitleaks already run as individual checks with better cache granularity.
      pre-commit.check.enable = false;

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
