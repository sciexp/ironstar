{ pkgs, lib, ... }:

{
  autoWire = [ ];
  crane = {
    args = {
      buildInputs =
        lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.IOKit
          pkgs.darwin.Security
          pkgs.darwin.SystemConfiguration
          pkgs.darwin.CF
        ]
        ++ lib.optionals pkgs.stdenv.isLinux [
          pkgs.openssl
        ];

      nativeBuildInputs = [
        pkgs.pkg-config
      ]
      ++ lib.optionals pkgs.stdenv.isDarwin [
        pkgs.darwin.Cocoa
      ];
    };
  };
}
