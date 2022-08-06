use figment::{Error as FigmentError, Figment, Provider};
use rayon::prelude::*;
use std::fs;
use std::io::Read;

use bomper::config::Config;
use bomper::replacers::file::FileReplacer;
use bomper::replacers::simple::bomp_files;
use bomper::replacers::search::SearchReplacer;
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
        let files_to_replace = self.config.files.clone().par_drain().map(|path| {
            let search_replacer = SearchReplacer::new(
                path,
                args.old_version.clone(),
                regex::bytes::Regex::new("bomper")?,
                args.new_version.clone(),
            )?;

            match search_replacer.overwrite_file() {
                Err(e) => Err(e),
                v => v,
            }
        }).collect::<Result<Vec<FileReplacer>>>()?;

        for replacer in files_to_replace {
            replacer.persist()?;
        }

        Ok(())
    }
}
