{ inputs, ... }:
{
  imports = [
    inputs.nix-unit.modules.flake.default
  ];
}
