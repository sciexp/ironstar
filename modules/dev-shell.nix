{
  inputs,
  ...
}:
{
  perSystem =
    {
      config,
      self',
      pkgs,
      lib,
      system,
      ...
    }:
    let
      # Playwright driver from versioned flake (synced with package.json)
      playwrightDriver = inputs.playwright-web-flake.packages.${system}.playwright-driver;

      # Git environment setup script (integrated from git-env.nix)
      set-git-env = pkgs.writeShellApplication {
        name = "set-git-env";
        runtimeInputs = with pkgs; [
          git
          coreutils
          gnused
          gnugrep
        ];
        text = ''
          #!/usr/bin/env bash

          set -euo pipefail

          update_or_append() {
              local var_name=$1
              local var_value=$2
              local file=".env"

              # Create .env file if it doesn't exist
              if [ ! -f "$file" ]; then
                  touch "$file"
              fi

              if [ -s "$file" ] && [ "$(tail -c1 "$file"; echo x)" != $'\nx' ]; then
                  echo >> "$file"
              fi

              if grep -q "^$var_name=" "$file"; then
                  # Use sed with backup extension for compatibility
                  sed -i.bak "s/^$var_name=.*/$var_name=$var_value/" "$file"
                  # Remove backup file
                  rm -f "$file.bak"
              else
                  echo "$var_name=$var_value" >> "$file"
              fi
          }

          GIT_REPO_NAME=$(basename -s .git "$(git config --get remote.origin.url 2>/dev/null || echo "unknown-repo")")
          update_or_append "GIT_REPO_NAME" "$GIT_REPO_NAME"

          GIT_REF=$(git symbolic-ref -q --short HEAD 2>/dev/null || git rev-parse HEAD 2>/dev/null || echo "unknown-ref")
          update_or_append "GIT_REF" "$GIT_REF"

          GIT_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown-sha")
          update_or_append "GIT_SHA" "$GIT_SHA"

          GIT_SHA_SHORT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
          update_or_append "GIT_SHA_SHORT" "$GIT_SHA_SHORT"
        '';
      };

      # Typst with pre-fetched packages for reproducible builds
      # Transitive deps must be listed explicitly (typst.withPackages doesn't auto-resolve)
      typstWithPackages = pkgs.typst.withPackages (
        ps: with ps; [
          oxifmt_0_2_1 # required by cetz 0.3.4
          fletcher # diagrams with nodes and arrows (pulls cetz)
          chronos # sequence diagrams
          polylux # presentations
        ]
      );
    in
    {
      devShells = {
        default = pkgs.mkShell {
          name = "ironstar-dev";
          inputsFrom = [ config.pre-commit.devShell ];
          packages = with pkgs; [
            # Rust toolchain (version defined in rust.nix)
            config.rust-project.toolchain
            cargo-edit
            cargo-nextest

            # Core development tools
            bun
            nodejs
            just
            git

            # Secrets management
            age
            sops
            ssh-to-age
            gitleaks

            # CI/CD tools
            gh
            act
            cachix

            # Specification and diagrams
            idris2
            d2

            # SVG rendering and PNG optimization
            resvg
            oxipng

            # E2E testing browsers from playwright-web-flake (pinned to 1.56.1)

            # Git environment setup
            set-git-env

            # Document typesetting
            typstWithPackages
          ];

          # Make fonts available to typst for consistent rendering across environments
          TYPST_FONT_PATHS = pkgs.lib.concatStringsSep ":" [
            "${pkgs.inter}/share/fonts/truetype"
            "${pkgs.lmodern}/share/fonts"
            "${pkgs.newcomputermodern}/share/fonts"
          ];

          shellHook = ''
            export REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
            set-git-env

            # Playwright browser configuration (version-locked via flake input)
            export PLAYWRIGHT_BROWSERS_PATH="${playwrightDriver.browsers}"
            export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
            export PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS=true

            printf "\n$GIT_REPO_NAME $GIT_REF $GIT_SHA_SHORT\n\n"
          '';
        };
      };
    };
}
