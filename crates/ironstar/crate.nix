{ pkgs, lib, ... }:

{
  # Disable rust-flake autowiring - using pure crane pattern in modules/rust.nix
  autoWire = [ ];
  crane = {
    args = {
      # System duckdb for dev builds (avoids libduckdb-sys C++ compilation).
      # For production bundled builds, remove duckdb from buildInputs and
      # re-enable "bundled" feature in Cargo.toml.
      buildInputs =
        lib.optionals pkgs.stdenv.isLinux [
          pkgs.openssl
        ]
        ++ [
          pkgs.duckdb
        ];

      nativeBuildInputs = [
        pkgs.pkg-config
      ];
    };
  };
}
