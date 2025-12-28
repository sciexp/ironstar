{ pkgs, lib, ... }:

{
  # Disable rust-flake autowiring - using pure crane pattern in modules/rust.nix
  autoWire = [ ];
  crane = {
    args = {
      buildInputs = lib.optionals pkgs.stdenv.isLinux [
        pkgs.openssl
      ];

      nativeBuildInputs = [
        pkgs.pkg-config
      ];
    };
  };
}
