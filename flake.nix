{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs =
    inputs:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f rec {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.rust-overlay.overlays.default
                inputs.self.overlays.default
              ];
            };
            naerskLib = pkgs.callPackage inputs.naersk { };
            pre-commit-hooks = inputs.git-hooks.lib.${system}.run {
              src = ./.;
              hooks = {
                nixfmt-rfc-style.enable = true;

                cargo-check = {
                  enable = true;
                };
                rustfmt = {
                  enable = true;
                  packageOverrides.cargo = pkgs.cargo;
                  packageOverrides.rustfmt = pkgs.rustfmt;
                };
                clippy = {
                  enable = true;
                  packageOverrides.cargo = pkgs.cargo;
                  packageOverrides.clippy = pkgs.clippy;
                };
              };
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rustfmt"
              ];
            };
      };

      packages = forEachSupportedSystem (
        { pkgs, naerskLib, ... }:
        rec {
          hitster = naerskLib.buildPackage {
            pname = "hitster";
            src = ./.;
            buildInputs = with pkgs; [
              rustToolchain
              openssl
              sqlx-cli
            ];
            nativeBuildInputs = with pkgs; [ pkg-config ];
          };
          default = hitster;

          dockerImage = pkgs.dockerTools.buildLayeredImage {
            name = "hitster";
            tag = "latest";
            created = "now";

            contents = [
              # Runtime dependencies
              pkgs.cacert
              pkgs.openssl
              pkgs.sqlite
              # Application
              hitster
            ];

            config = {
              Entrypoint = [ "${hitster}/bin/hitster" ];
              ExposedPorts = {
                "3000/tcp" = { };
              };
              Env = [
                "HITSTER_DATABASE__PATH=/data/db/hitster.db"
                "HITSTER_CONFIG_DIR=/config"
              ];
              WorkingDir = "/data";
              Volumes = {
                "/data" = { };
              };
            };
          };
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs, pre-commit-hooks, ... }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              openssl
              pkg-config
              cargo-deny
              cargo-edit
              cargo-watch
              rust-analyzer

              sqlx-cli
              dive
            ];

            buildInputs = [ pre-commit-hooks.enabledPackages ];
            shellHook = ''
              ${pre-commit-hooks.shellHook}
            '';

            env = {
              DATABASE_URL = "sqlite://./db/hitster.db";
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            };
          };
        }
      );
    };
}
