{
  inputs,
  self,
  ...
} @ part-inputs: {
  imports = [];

  perSystem = {
    config,
    pkgs,
    lib,
    system,
    inputs',
    self',
    ...
  }: let
    devTools = [
      # rust tooling
      self'.packages.rust-toolchain
      pkgs.cargo-audit
      pkgs.cargo-udeps
      pkgs.bacon
      # version control
      pkgs.cocogitto
      config.bomper.wrappedBomper
      # formatting
      self'.packages.treefmt
      # misc
    ];

    # packages required for building the rust packages
    extraPackages = [
      pkgs.pkg-config
    ];
    withExtraPackages = base: base ++ extraPackages;

    craneLib = (inputs.crane.mkLib pkgs).overrideToolchain self'.packages.rust-toolchain;

    common-build-args = rec {
      src = inputs.nix-filter.lib {
        root = ../.;
        include = [
          "src"
          "Cargo.toml"
          "Cargo.lock"
        ];
      };

      pname = "bomper";

      nativeBuildInputs = withExtraPackages [];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
    };

    deps-only = craneLib.buildDepsOnly ({} // common-build-args);

    packages = {
      default = packages.bomp;
      bomp = craneLib.buildPackage ({
          pname = "bomp";
          cargoArtifacts = deps-only;
          meta.mainProgram = "bomp";
        }
        // common-build-args);
      cargo-doc = craneLib.cargoDoc ({
          cargoArtifacts = deps-only;
        }
        // common-build-args);
    };

    checks = {
      clippy = craneLib.cargoClippy ({
          cargoArtifacts = deps-only;
          cargoClippyExtraArgs = "--all-features -- --deny warnings";
        }
        // common-build-args);

      rust-fmt = craneLib.cargoFmt ({
          inherit (common-build-args) src;
        }
        // common-build-args);

      rust-tests = craneLib.cargoNextest ({
          cargoArtifacts = deps-only;
          partitions = 1;
          partitionType = "count";
        }
        // common-build-args);
    };
  in rec {
    inherit packages checks;

    devShells.default = pkgs.mkShell rec {
      packages = withExtraPackages devTools;
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;

      shellHook = ''
        ${config.pre-commit.installationScript}
      '';
    };

    apps = {
      bomp = {
        type = "app";
        program = pkgs.lib.getBin self'.packages.bomp;
      };
      default = apps.bomp;
    };
  };
}
