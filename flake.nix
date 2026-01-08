{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      rust-overlay,
      nixpkgs,
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f rec {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };

            rust-toolchain = (
              with pkgs.rust-bin;
              [
                (stable.latest.minimal.override {
                  extensions = [
                    "clippy"
                    "rust-src"
                  ];
                })

                nightly.latest.rustfmt
                nightly.latest.rust-analyzer
              ]
            );
          }
        );
    in
    {

      devShells = forAllSystems (
        { pkgs, rust-toolchain }:
        {
          default =
            with pkgs;
            mkShell {
              packages = [
                fzf
                skim
                glow
                ripgrep
              ]
              ++ rust-toolchain;
            };
        }
      );

      formatter = forAllSystems ({ pkgs, ... }: pkgs.nixfmt-rfc-style);
    };
}
