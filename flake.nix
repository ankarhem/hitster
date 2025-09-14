{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    let
      supportedSystems =
        [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (system:
          f rec {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.rust-overlay.overlays.default
                inputs.self.overlays.default
              ];
            };
            naerskLib = pkgs.callPackage inputs.naersk { };
          });
    in {
      overlays.default = final: prev: {
        rustToolchain = let rust = prev.rust-bin;
        in if builtins.pathExists ./rust-toolchain.toml then
          rust.fromRustupToolchainFile ./rust-toolchain.toml
        else if builtins.pathExists ./rust-toolchain then
          rust.fromRustupToolchainFile ./rust-toolchain
        else
          rust.stable.latest.default.override {
            extensions = [ "rust-src" "rustfmt" ];
          };
      };

      packages = forEachSupportedSystem ({ pkgs, naerskLib }: rec {
        default = naerskLib.buildPackage {
          pname = "hitster";
          src = ./.;
          buildInputs = with pkgs; [ rustToolchain openssl sqlx-cli ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };

        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "hitster";
          tag = "latest";
          created = "now";

          contents = [
            # Runtime dependencies
            pkgs.cacert
            pkgs.openssl
            pkgs.sqlite
          ];

          config = {
            Entrypoint = [ "${default}/bin/hitster" ];
            ExposedPorts = { "3000/tcp" = { }; };
            Env = [ "RUST_LOG=info" "DATABASE_URL=sqlite:///data/hitster.db" ];
            WorkingDir = "/data";
          };
        };
      });

      devShells = forEachSupportedSystem ({ pkgs, ... }: {
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
          ];

          shellHook = ''
            export DATABASE_URL="sqlite://./db/hitster.db"
            cargo sqlx prepare
          '';

          env = {
            # Required by rust-analyzer
            RUST_SRC_PATH =
              "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
      });
    };
}
