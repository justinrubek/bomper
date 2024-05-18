_: {
  perSystem = _: {
    bomper = {
      enable = true;
      configuration = ''
        (
            cargo: Some(Autodetect),
            authors: Some({
                "Justin Rubek": "justinrubek"
            }),
        )
      '';
    };
  };
}
