{
  description = "Alex222222222222 configuration for arm64 nix os";

  nixConfig = {
    experimental-features =
      [ "nix-command" "flakes" "auto-allocate-uids" "configurable-impure-env" ];
    substituters = [
      "https://cache.nixos.org/"
      "https://mirrors.bfsu.edu.cn/nix-channels/store"
    ];

    # nix community's cache server
    extra-substituters =
      [ "https://nix-community.cachix.org" "https://hyprland.cachix.org" ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "hyprland.cachix.org-1:a7pgxzMz7+chwVL3/pzj6jIBMioiJM7ypFP8PwtkuGc="
    ];
  };

  inputs = {
    # NixOS 官方软件源，这里使用 nixos-unstable 分支
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system: {
      devShells =
        let
          inherit (pkgs.stdenv) isLinux;
          pkgs = import nixpkgs {
            system = system;
            config.allowUnfree = true;
            overlays = [ rust-overlay.overlay ];
          };

          toolchain = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" "rust-std" ];
            targets = [ "wasm32-unknown-unknown" ];
          };

          buildInputs = with pkgs; [
            unzip
            git
            curl
            wget
            ncdu
            neovim
            htop
            nixfmt
            sqlite
            tokei
            mosh
            vscode
            nil
            nodejs-18_x
            nodePackages.npm
            pkg-config
            nodePackages_latest.tailwindcss
          ];
          rust-packages-linux = with pkgs; [
            protobuf
            binaryen
            wasm-pack
            trunk
            wasm-bindgen-cli
            rustfmt
            clippy
            rust-analyzer
            cargo-tauri
            toolchain
            gtk3
            webkitgtk
            libayatana-appindicator.dev
            alsa-lib.dev
          ];
          rust-packages-darwin = with pkgs; [
            # libiconv
            # darwin.apple_sdk.frameworks.Security
            # darwin.apple_sdk.frameworks.CoreServices
            # darwin.apple_sdk.frameworks.CoreFoundation
            # darwin.apple_sdk.frameworks.Foundation
            # darwin.apple_sdk.frameworks.AppKit
            # darwin.apple_sdk.frameworks.WebKit
            # darwin.apple_sdk.frameworks.Cocoa
          ];
          rust-packages =
            if isLinux then rust-packages-linux else rust-packages-darwin;
          rust-packages-with-my-packages = buildInputs ++ rust-packages;
          shellHook = ''
            export EDITOR=nvim
            export VISUAL=nvim
            export PATH=$PATH:$HOME/.local/bin
          '';
        in
        {
          default = pkgs.mkShell {
            buildInputs = rust-packages-with-my-packages;
            shellHook = shellHook;
          };
        };

      formatter = nixpkgs.legacyPackages.${system}.nixpkgs-fmt;
    });
}
