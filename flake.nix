{
  description = "bumps versions in files";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    nix-filter.url = "github:numtide/nix-filter";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} ({
      flake-parts-lib,
      withSystem,
      ...
    }: let
      inherit (flake-parts-lib) importApply;

      flakeModules.bomper = importApply ./flake-parts/flake-module.nix {inherit withSystem;};
    in {
      systems = ["x86_64-linux" "aarch64-linux"];
      imports = [
        inputs.pre-commit-hooks.flakeModule
        flakeModules.bomper

        ./flake-parts/bomper.nix
        ./flake-parts/cargo.nix
        ./flake-parts/ci.nix
        ./flake-parts/formatting.nix
        ./flake-parts/pre-commit.nix
        ./flake-parts/rust-toolchain.nix
      ];

      flake = {
        inherit flakeModules;
      };
    });
}
