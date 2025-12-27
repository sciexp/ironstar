{ pkgs, lib, ... }:

{
  autoWire = [
    "crate"
    "clippy"
    "doc"
  ];
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
