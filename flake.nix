{
  description = "bumps versions in files";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
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
    crane,
    fenix,
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
        lib,
        ...
      }: let
        craneLib = crane.lib.${system};
        common-build-args = rec {
          src = lib.cleanSourceWith {
            src = ./.;
          };
          pname = "bomper";
        };
        deps-only = craneLib.buildDepsOnly ({
            pname = "bomper";
          }
          // common-build-args);

        clippy-check = craneLib.cargoClippy ({
            cargoArtifacts = deps-only;
            cargoClippyExtraArgs = "--all-features -- --deny warnings";
          }
          // common-build-args);

        tests-check = craneLib.cargoNextest ({
            cargoArtifacts = deps-only;
            partitions = 1;
            partitionType = "count";
          }
          // common-build-args);

        rustPackage = craneLib.buildPackage ({
            pname = "bomper-cli";
            cargoArtifacts = deps-only;
            cargoExtraArgs = "--bin bomper";
          }
          // common-build-args);

        rust-environment = inputs'.fenix.packages.latest.toolchain;

        bomper-cli = bomper.packages.${system}.cli;
      in rec {
        packages = {
          cli = rustPackage;
          default = packages.cli;
        };
        devShells = {
          default = pkgs.mkShell rec {
            buildInputs = [
              rust-environment 
              pkgs.alejandra
              pkgs.rustfmt 

              pkgs.cocogitto 
              pkgs.bacon

              bomper-cli
            ];
          };
        };
        apps = {
          cli = {
            type = "app";
            program = "${packages.cli}/bin/bomper";
          };
          default = apps.cli;
        };
        checks = {
          build-cli = packages.cli;
          inherit clippy-check tests-check;
        };
      };
      systems = flake-utils.lib.defaultSystems;
    };
}
