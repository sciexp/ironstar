# Cross-compilation patterns for Rust + Nix

Reference for multi-arch release builds using crane with Nix flakes.
Current state: `nix build .#ironstar-release` produces a single-arch LTO-optimized binary.
Future: Multi-target static binaries for distribution.

## When to implement

Cross-compilation becomes necessary when distribution requirements exceed the current single-arch build.
Typical triggers include GitHub releases with downloadable binaries for multiple platforms, crates.io publishing that benefits from pre-built artifacts, and container image multi-arch manifests for Docker Hub or GitHub Container Registry.

Until these requirements materialize, the current single-arch release build with `lto = true`, `strip = true`, and `opt-level = "z"` remains sufficient.

## Local references

The `nix-cargo-crane` repository at `~/projects/nix-workspace/nix-cargo-crane/examples/` contains four cross-compilation examples that demonstrate progressively complex patterns.

### Static musl linking

The `cross-musl` example produces fully static Linux binaries by targeting musl libc.
This pattern uses `overrideToolchain` to add the musl target, then sets cargo environment variables for static linking.

```nix
craneLib = (crane.mkLib pkgs).overrideToolchain (
  p:
  p.rust-bin.stable.latest.default.override {
    targets = [ "x86_64-unknown-linux-musl" ];
  }
);

my-crate = craneLib.buildPackage {
  src = craneLib.cleanCargoSource ./.;
  strictDeps = true;
  CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
  CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
};
```

The `-C target-feature=+crt-static` flag ensures the C runtime is statically linked, producing a single binary with no dynamic library dependencies.

### pkgs.callPackage splicing

The `cross-rust-overlay` example demonstrates Nix package splicing for cross-compilation with native dependencies.
When building for a different architecture, Nix must distinguish between packages that run during build (nativeBuildInputs) and packages linked into the final binary (buildInputs).

```nix
pkgs = import nixpkgs {
  inherit crossSystem localSystem;
  overlays = [ (import rust-overlay) ];
};

crateExpression =
  { openssl, libiconv, lib, pkg-config, stdenv }:
  craneLib.buildPackage {
    src = craneLib.cleanCargoSource ./.;
    strictDeps = true;
    # pkg-config runs on build platform to locate openssl
    nativeBuildInputs = [ pkg-config ]
      ++ lib.optionals stdenv.buildPlatform.isDarwin [ libiconv ];
    # openssl links into binary for target platform
    buildInputs = [ openssl ];
  };

my-crate = pkgs.callPackage crateExpression { };
```

Using `pkgs.callPackage` allows Nix to automatically splice dependencies for the correct platform, avoiding manual `pkgsBuildHost` and `pkgsHostHost` specifications.

### mingw32 Windows targeting

The `cross-windows` example produces Windows executables from Linux using the mingw32 toolchain.
The crossSystem specification includes both the GNU config triplet and the libc variant.

```nix
pkgs = import nixpkgs {
  overlays = [ (import rust-overlay) ];
  localSystem = system;
  crossSystem = {
    config = "x86_64-w64-mingw32";
    libc = "msvcrt";
  };
};

craneLib = (crane.mkLib pkgs).overrideToolchain (
  p:
  p.rust-bin.stable.latest.default.override {
    targets = [ "x86_64-pc-windows-gnu" ];
  }
);
```

This produces `.exe` binaries that link against the Microsoft Visual C Runtime through mingw32 compatibility layers.

### Nightly build-std

The `build-std` example compiles the Rust standard library from source, enabling optimizations and target features not available in pre-built std.
This requires nightly Rust with the `rust-src` component.

```nix
rustToolchainFor =
  p:
  p.rust-bin.selectLatestNightlyWith (
    toolchain:
    toolchain.default.override {
      extensions = [ "rust-src" ];
      targets = [ "x86_64-unknown-linux-gnu" ];
    }
  );

my-crate = craneLib.buildPackage {
  inherit src;
  strictDeps = true;
  cargoVendorDir = craneLib.vendorMultipleCargoDeps {
    inherit (craneLib.findCargoFiles src) cargoConfigs;
    cargoLockList = [
      ./Cargo.lock
      "${rustToolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/library/Cargo.lock"
    ];
  };
  cargoExtraArgs = "-Z build-std --target x86_64-unknown-linux-gnu";
};
```

The `vendorMultipleCargoDeps` function handles both the project's dependencies and the standard library's dependencies.
This approach requires import-from-derivation (IFD) to read the rustlib Cargo.lock, or manual copying to avoid IFD.

## External exemplars

Several open-source projects demonstrate production-grade cross-compilation patterns worth studying.

**rustshop/flakebox** at [github.com/rustshop/flakebox](https://github.com/rustshop/flakebox) provides a higher-level crane wrapper with multi-target support spanning Linux, Darwin, Android, and iOS.
It includes CI workflow generation and simplifies common cross-compilation scenarios.

**syncom/rust-cross-build-nix** at [github.com/syncom/rust-cross-build-nix](https://github.com/syncom/rust-cross-build-nix) focuses on bit-for-bit reproducible cross-compilation.
The repository compares pure Nix approaches against cross-rs and demonstrates static OpenSSL linking patterns.

**mediocregopher's eachCrossSystem** documented at [mediocregopher.com/posts/x-compiling-rust-with-nix](https://mediocregopher.com/posts/x-compiling-rust-with-nix) presents a complete matrix build utility.
The pattern covers x86_64, i686, aarch64, and armv6l Linux targets plus Windows:

```nix
buildTargets = {
  "aarch64-linux" = {
    crossSystemConfig = "aarch64-unknown-linux-musl";
    rustTarget = "aarch64-unknown-linux-musl";
  };
  "x86_64-linux" = {
    crossSystemConfig = "x86_64-unknown-linux-musl";
    rustTarget = "x86_64-unknown-linux-musl";
  };
  "x86_64-windows" = {
    crossSystemConfig = "x86_64-w64-mingw32";
    rustTarget = "x86_64-pc-windows-gnu";
  };
};
```

**helix-editor/helix** at [github.com/helix-editor/helix/blob/master/flake.nix](https://github.com/helix-editor/helix/blob/master/flake.nix) demonstrates a production text editor with a mature multi-system flake.
Their approach includes Cachix integration for binary caching and overlay composition for toolchain management.

**srid/rust-nix-template** at [github.com/srid/rust-nix-template](https://github.com/srid/rust-nix-template) uses juspay/rust-flake, a crane-based template system.
It demonstrates dual CI approaches and provides a minimal starting point for new Rust projects.

## Key patterns

### eachCrossSystem matrix builds

For releasing to multiple targets simultaneously, define a build target matrix that maps output names to cross-compilation configurations.
This pattern iterates over targets, producing platform-specific packages in the flake outputs.

The mediocregopher approach wraps `flake-utils.lib.eachDefaultSystem` with an inner loop over target configurations, producing outputs like `packages.x86_64-linux.aarch64-linux-binary` for each host-target combination.

### Static musl linking

Musl libc produces fully self-contained Linux binaries suitable for distribution across different Linux distributions.
The resulting binaries have no runtime dependencies beyond the kernel syscall interface.

Key considerations include larger binary sizes compared to glibc-linked binaries, potential DNS resolution differences (musl uses a simpler resolver), and compatibility with some C libraries that assume glibc extensions.
For server applications like ironstar, these tradeoffs favor portability.

### craneLib.overrideToolchain

The `overrideToolchain` function replaces crane's default Rust toolchain without affecting the broader nixpkgs Rust derivations.
This scoped approach avoids rebuilding unrelated packages that depend on Rust.

The function accepts either a toolchain derivation directly or a function `pkgs -> toolchain`, allowing lazy evaluation when the toolchain depends on overlays applied to pkgs.

### pkgs.callPackage splicing

For crates with C dependencies, `callPackage` enables Nix to automatically handle the distinction between build-time and run-time dependencies.
Without splicing, cross-compilation requires explicit package set selection (`pkgsBuildHost`, `pkgsHostTarget`), which is error-prone.

The pattern defines the build expression as a function accepting dependencies, then calls it through `pkgs.callPackage`.
Nix's cross-compilation infrastructure ensures each dependency comes from the appropriate package set.

## Ironstar roadmap

The current release build at `nix build .#ironstar-release` in `modules/rust.nix` uses a standard release profile with LTO and stripping enabled.
Cargo.toml configures `[profile.release]` with `lto = true`, `strip = true`, and `opt-level = "z"` for size optimization.

When multi-arch distribution becomes necessary, the implementation path follows this sequence.
First, add `x86_64-unknown-linux-musl` as a second target using the static musl pattern from the local examples.
Second, if Windows builds are required, add the mingw32 cross-system configuration.
Third, for releases requiring optimized standard library, add the nightly build-std pattern.

The flakebox wrapper may simplify this progression if its abstractions align with ironstar's needs.
Evaluate against the raw crane patterns before adopting the higher-level wrapper.

## References

Local examples:
- `~/projects/nix-workspace/nix-cargo-crane/examples/cross-musl/flake.nix`
- `~/projects/nix-workspace/nix-cargo-crane/examples/cross-rust-overlay/flake.nix`
- `~/projects/nix-workspace/nix-cargo-crane/examples/cross-windows/flake.nix`
- `~/projects/nix-workspace/nix-cargo-crane/examples/build-std/flake.nix`

External repositories:
- [rustshop/flakebox](https://github.com/rustshop/flakebox)
- [syncom/rust-cross-build-nix](https://github.com/syncom/rust-cross-build-nix)
- [mediocregopher's cross-compilation guide](https://mediocregopher.com/posts/x-compiling-rust-with-nix)
- [helix-editor/helix flake.nix](https://github.com/helix-editor/helix/blob/master/flake.nix)
- [srid/rust-nix-template](https://github.com/srid/rust-nix-template)
