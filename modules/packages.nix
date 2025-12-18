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
      ...
    }:
    {
      # nix build
      packages = {
        # Docs build package can be added here
        # For now, use `bun run build` in the devshell for building docs
        # set-git-env is available from dev-shell module
      };
    };
}
