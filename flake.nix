{
  description = "bumps versions in files";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    bomper = {
      url = "github:justinrubek/bomper";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    flake-parts,
    gitignore,
    rust-overlay,
    pre-commit-hooks,
    bomper,
    ...
  }:
    flake-parts.lib.mkFlake {inherit self;} {
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        inherit (gitignore.lib) gitignoreSource;
        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = gitignoreSource ./.;
          hooks = {
            alejandra.enable = true;
            rustfmt.enable = true;
          };
        };

        opkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };
        rust = opkgs.rust-bin.stable.latest.default;
        rustPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "bomper";
          version = "0.5.1";

          src = gitignoreSource ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [rust];
        };

        bomper-cli = bomper.packages.${system}.cli;
      in rec {
        packages = {
          cli = rustPackage;
          default = packages.cli;
        };
        devShells = {
          default = pkgs.mkShell rec {
            buildInputs = with pkgs; [rust rustfmt cocogitto bomper-cli];
            inherit (pre-commit-check) shellHook;
          };
        };
        apps = {
          cli = {
            type = "app";
            program = "${packages.cli}/bin/bomper";
          };
          default = apps.cli;
        };
      };
      systems = flake-utils.lib.defaultSystems;
    };
}
