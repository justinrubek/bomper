use anyhow::anyhow;
use figment::{Error as FigmentError, Figment, Provider};
use rayon::prelude::*;
use std::fs;
use std::io::Read;

use bomper::config::Config;
use bomper::replacers::Replacer;
use bomper::replacers::file::FileReplacer;
use bomper::replacers::simple::SimpleReplacer;
use bomper::replacers::search::SearchReplacer;
use bomper::error::{Error, Result};

use crate::cli::Args;

pub struct App {
    pub config: Config,
}

impl App {
    pub fn new() -> Result<App> {
        let config = Config::from_file(&String::from("bomp.toml"))?;
        
        Ok(App {
            config,
        })
    }
}

impl App {
    pub fn run(&self, args: &Args) -> Result<()> {
        // self.config.file.clone().par_drain().for_each(|path| {
        let files_to_replace = self.config.file.clone().par_drain().map(|path| {
            let (path, config) = path;

            let replacer = match config.search_value {
                Some(value) => SearchReplacer::new(
                        path,
                        &args.old_version,
                        &value,
                        &args.new_version,
                    )?.overwrite_file()?,
                None => SimpleReplacer::new(
                        path,
                        &args.old_version,
                        &args.new_version,
                    )?.overwrite_file()?,
            };

            Ok(replacer)

        })
        .filter_map(|val| match val {
            Ok(Some(val)) => Some(Ok(val)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<FileReplacer>>>()?;

        /*
        for replacer in files_to_replace {
            replacer.persist()?;
        }
        */

        Ok(())
    }
}
