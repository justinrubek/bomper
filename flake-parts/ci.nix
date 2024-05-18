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
          self'.packages.bomper
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
      };
    };
    packages = {
      bomper = config.bomper.wrappedBomper;
    };
  };
}
