use std::fs;

use figment::{Error as FigmentError, Figment, Provider};
use rayon::prelude::*;

use bomper::config::Config;
use bomper::file::overwrite_file;
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
        let r: Result<Vec<_>, _> = self.config.files.par_iter().map(|file| {
            overwrite_file(file, &args.old_version, &args.new_version)
        }).collect();

        // Only persist the changes if all operations succeed
        match r {
            Err(e) => Err(e),
            Ok(files) => {
                for replacer in files {
                    replacer.temp_file.persist(fs::canonicalize(replacer.path)?)?;
                }

                Ok(())
            }
        }
    }
}
