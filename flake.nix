{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };

        ciRustPackages = with pkgs; [
          (rust-bin.stable.latest.minimal.override {
            extensions = [ "clippy" ];
          })
          rust-bin.nightly.latest.rustfmt
        ];

        devRustPackages = with pkgs; [
          (rust-bin.stable.latest.minimal.override {
            extensions = [
              "clippy"
              "rust-analyzer"
              "rust-src"
            ];
          })
          rust-bin.nightly.latest.rustfmt
        ];

        ciCommonPackages = with pkgs; [
          cargo-nextest
          just
          nixfmt
          taplo
        ];

        devCommonPackages =
          with pkgs;
          [
            cargo-audit
            cargo-expand
            commitlint
            just-lsp
            nil
            nixd
          ]
          ++ ciCommonPackages;
      in
      {
        devShells = {
          ci = pkgs.mkShell {
            packages = ciRustPackages ++ ciCommonPackages;
          };

          default = pkgs.mkShell {
            packages = devRustPackages ++ devCommonPackages;
          };
        };

        apps.direnv = flake-utils.lib.mkApp {
          drv = pkgs.direnv;
        };

        formatter = pkgs.nixfmt-tree;
      }
    );
}
