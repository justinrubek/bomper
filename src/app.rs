use figment::{Error, Figment, Provider};
use rayon::prelude::*;

use bomper::config::Config;
use bomper::file::overwrite_file;

use crate::cli::Args;

pub struct App {
    pub config: Config,
    pub figment: Figment,
}

impl App {
    pub fn new() -> Result<App, Error> {
        App::custom(Config::figment())
    }

    pub fn custom<T: Provider>(provider: T) -> Result<App, Error> {
        let figment = Figment::from(provider);
        Ok(App {
            config: Config::from(&figment)?,
            figment,
        })
    }
}

impl App {
    pub fn run(&self, args: &Args) -> Result<(), Error> {
        self.config.files.par_iter().for_each(|file| {
            overwrite_file(file, &args.old_version, &args.new_version);
        });

        Ok(())
    }
}
