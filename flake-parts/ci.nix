_: {
  perSystem = {
    config,
    pkgs,
    self',
    ...
  }: {
    devShells = {
      ci = pkgs.mkShell rec {
        packages = [
          self'.packages.bomp
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
      };
    };
    packages = {
      bomp = config.bomper.wrappedBomper;
    };
  };
}
