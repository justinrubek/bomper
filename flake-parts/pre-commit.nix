{
  inputs,
  self,
  ...
}: {
  perSystem = {
    pkgs,
    lib,
    ...
  }: let
    formatters = [
      pkgs.alejandra
      pkgs.rustfmt
    ];

    # makeWrapper treefmt to provide the correct PATH with all formatters
    treefmt = pkgs.stdenv.mkDerivation {
      name = "treefmt";
      buildInputs = [pkgs.makeWrapper];
      buildCommand = ''
        makeWrapper \
          ${pkgs.treefmt}/bin/treefmt \
          $out/bin/treefmt \
          --prefix PATH : ${lib.makeBinPath formatters}
      '';
    };
  in {
    packages = {
      inherit treefmt;
    };

    pre-commit = {
      check.enable = true;

      settings = {
        src = ../.;
        hooks = {
          treefmt = {
            enable = true;
            name = "treefmt";
            description = "format the code";
            types = ["file"];
            pass_filenames = true;
            entry = "${treefmt}/bin/treefmt";
          };
        };
      };
    };
  };
}
