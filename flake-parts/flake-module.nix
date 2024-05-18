localFlake: {
  lib,
  flake-parts-lib,
  ...
}: {
  options.perSystem = flake-parts-lib.mkPerSystemOption ({
    config,
    pkgs,
    system,
    ...
  }: let
    cfg = config.bomper;
  in {
    options = {
      bomper = {
        enable = lib.mkOption {
          type = lib.types.bool;
          default = false;
          description = ''
            Whether to enable generation bomper.
          '';
        };

        configuration = lib.mkOption {
          type = lib.types.lines;
          description = ''
            The contents of the configuration file to pass to bomper.
          '';
        };

        wrappedBomper = lib.mkOption {
          description = "A wrapped version of bomper with configuration already specified.";
          readOnly = true;
          type = lib.types.package;
        };
      };
    };

    config = let
      configFile = pkgs.writeText "bomper-config" cfg.configuration;
    in
      lib.mkIf cfg.enable {
        bomper.wrappedBomper = localFlake.withSystem system ({config, ...}: (pkgs.writeShellApplication {
          name = "bomp";
          runtimeInputs = [config.packages.bomp];
          text = ''exec bomp --config-file ${configFile} "$@"'';
        }));
      };
  });
}
