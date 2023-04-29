{
  inputs,
  self,
  ...
}: {
  perSystem = {pkgs, ...}: {
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
            entry = "${pkgs.treefmt}/bin/treefmt";
          };
        };
      };
    };
  };
}
