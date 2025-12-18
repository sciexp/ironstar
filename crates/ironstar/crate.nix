{ pkgs, lib, ... }:

{
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
