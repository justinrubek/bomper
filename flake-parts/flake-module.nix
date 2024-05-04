localFlake: {
  self,
  lib,
  flake-parts-lib,
  inputs,
  ...
}: {
  options = {
    perSystem = flake-parts-lib.mkPerSystemOption ({
      config,
      inputs',
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
            type = lib.types.package;
            readOnly = true;
            description = "A wrapped version of bomper with configuration already specified.";
          };
        };
      };

      config = let
        configFile = pkgs.writeText "bomper-config" cfg.configuration;
      in
        lib.mkIf cfg.enable {
          bomper.wrappedBomper = localFlake.withSystem system ({config, ...}: (pkgs.writeShellApplication {
            name = "bomper";
            runtimeInputs = [config.packages.cli];
            text = ''exec bomper --config-file ${configFile} "$@"'';
          }));
        };
    });
  };
}
