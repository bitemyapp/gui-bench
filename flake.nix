{
  description = "gui-bench: a shell that builds and runs every adapter on NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      }));
    in
    {
      devShells = forAll (
        pkgs:
        let
          # The channel in rust-toolchain.toml is the one the adapters are
          # compared on; read it, so the two cannot drift apart.
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          # Linked at build time by one adapter or another.
          libraries = with pkgs; [
            alsa-lib
            fontconfig
            freetype
            libdrm
            libgbm
            libva
            libxkbcommon
            openssl
            wayland
            libx11
            libxcb
            zlib
          ];
          # Opened with dlopen at run time: the Vulkan loader for wgpu,
          # the window systems' client libraries, fontconfig.
          runtime = with pkgs; [
            libGL
            libxcursor
            libxi
            libxrandr
            vulkan-loader
          ];
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              toolchain
              pkg-config
              cmake
              rustPlatform.bindgenHook
              # zgui's style engine (stylo) generates its properties with it.
              python3
              # The runner's host description.
              pciutils
            ];
            buildInputs = libraries;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (libraries ++ runtime);
          };
        }
      );
    };
}
