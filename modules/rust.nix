{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];

  perSystem =
    {
      config,
      self',
      pkgs,
      lib,
      ...
    }:
    {
      # Configure per-crate crane args via crate.nix files
      rust-project = {
        crateNixFile = "crate.nix";
      };

      # Expose the main package
      packages =
        let
          inherit (config.rust-project) crates;
        in
        {
          ironstar = crates."ironstar".crane.outputs.drv.crate;
          # Don't set default here - let packages.nix handle it
        };
    };
}
