use figment::{Error as FigmentError, Figment, Provider};

use bomper::config::Config;
use bomper::replacers::simple::bomp_files;
use bomper::error::Result;

use crate::cli::Args;

pub struct App {
    pub config: Config,
    pub figment: Figment,
}

impl App {
    pub fn new() -> Result<App, FigmentError> {
        App::custom(Config::figment())
    }

    pub fn custom<T: Provider>(provider: T) -> Result<App, FigmentError> {
        let figment = Figment::from(provider);
        Ok(App {
            config: Config::from(&figment)?,
            figment,
        })
    }
}

impl App {
    pub fn run(&self, args: &Args) -> Result<()> {
        bomp_files(self.config.files.clone(), &args.old_version, &args.new_version)
    }
}
