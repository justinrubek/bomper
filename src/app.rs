use figment::{Error, Figment, Provider};

use bomper::config::Config;

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
