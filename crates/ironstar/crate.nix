{ pkgs, lib, ... }:

{
  autoWire = [ ];
  crane = {
    args = {
      buildInputs =
        lib.optionals pkgs.stdenv.isDarwin (
          with pkgs.apple-sdk.frameworks;
          [
            IOKit
            Security
            SystemConfiguration
            CoreFoundation
          ]
        )
        ++ lib.optionals pkgs.stdenv.isLinux [
          pkgs.openssl
        ];

      nativeBuildInputs = [
        pkgs.pkg-config
      ]
      ++ lib.optionals pkgs.stdenv.isDarwin [
        pkgs.apple-sdk.frameworks.Cocoa
      ];
    };
  };
}
