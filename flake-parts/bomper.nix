_: {
  perSystem = {...}: {
    bomper = {
      enable = true;
      configuration = ''
        (
            cargo: Some(Autodetect),
        )
      '';
    };
  };
}
