{
  description = "Rust プログラム実行用環境";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        lib = pkgs.lib;
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        llvm = pkgs.llvmPackages_16;
      in
      {
        devShells.default = pkgs.mkShell {
          name = "rustDevEnv";
          stdenv = llvm.stdenv; # TODO: 不要?
          nativeBuildInputs = [
            # Mold Linker for faster builds (only on Linux)
            (lib.optionals pkgs.stdenv.isLinux pkgs.mold)
            llvm.libclang.lib
            llvm.libcxxClang
            toolchain
          ];
          buildInputs = [
          ];
          packages = [];

          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          LIBCLANG_PATH = "${llvm.libclang.lib}/lib";
        };
      }
    );
}
