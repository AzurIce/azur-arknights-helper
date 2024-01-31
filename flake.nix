{
  description = "demo-iced";

  nixConfig = {
    extra-substituters = [
      "https://mirrors.ustc.edu.cn/nix-channels/store"
    ];
    trusted-substituters = [
      "https://mirrors.ustc.edu.cn/nix-channels/store"
    ];
  };


  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    nur.url = github:nix-community/NUR;
  };

  outputs = { self, nur, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) (nur.overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-tools = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          # RUST_SRC_PATH = "${rust-tools}/lib/rustlib/src/rust/library";
          DYLD_LIBRARY_PATH="${pkgs.libclang.lib}/lib";

          buildInputs = with pkgs; [
            # clang
            libclang
            # llvmPackages_16.bintools
            # openssl
            # pkg-config
            opencv
          ] ++ [
            rust-tools
          ];
        };
      }
    );
}
