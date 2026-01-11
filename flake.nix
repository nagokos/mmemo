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
            inherit system;

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
        { pkgs, rust-toolchain, ... }:
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

      packages = forAllSystems (
        { pkgs, ... }:
        let
          mmemo = pkgs.rustPlatform.buildRustPackage {
            pname = "mmemo";
            version = "0.1.0";
            src = self;

            cargoLock.lockFile = ./Cargo.lock;

            buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];

            meta.mainProgram = "mmemo";
          };
        in
        {
          default = mmemo;
          mmemo = mmemo;
        }
      );

      apps = forAllSystems (
        { system, ... }:
        let
          pkg = self.packages.${system}.mmemo;
        in
        {
          default = {
            type = "app";
            program = "${pkg}/bin/mmemo";
          };

          mmemo = {
            type = "app";
            program = "${pkg}/bin/mmemo";
          };
        }
      );

      formatter = forAllSystems ({ pkgs, ... }: pkgs.nixfmt-rfc-style);
    };
}
