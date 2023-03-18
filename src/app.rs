use bomper::replacers::cargo::CargoReplacer;

use bomper::config::Config;
use bomper::error::Result;
use bomper::replacers::search::SearchReplacer;
use bomper::replacers::simple::SimpleReplacer;
use bomper::replacers::{Replacer, VersionReplacement};

use crate::cli::Args;

pub struct App {
    pub config: Config,
}

impl App {
    pub fn new() -> Result<App> {
        let ron_config = Config::from_ron(&String::from("bomp.ron"))?;

        Ok(App { config: ron_config })
    }
}

impl App {
    pub fn run(&self, args: &Args) -> Result<()> {
        // self.config.file.clone().par_drain().for_each(|path| {
        let versions = VersionReplacement {
            old_version: args.old_version.clone(),
            new_version: args.new_version.clone(),
        };
        let mut files_to_replace = Vec::new();

        let by_file = &self.config.by_file;
        if let Some(by_file) = by_file {
            for (path, config) in by_file {
                let mut replacers = match &config.search_value {
                    Some(value) => SearchReplacer::new(
                        path.clone(),
                        &args.old_version,
                        value,
                        &args.new_version,
                    )?
                    .determine_replacements()?,
                    None => {
                        SimpleReplacer::new(path.clone(), &args.old_version, &args.new_version)?
                            .determine_replacements()?
                    }
                };

                // append new replacers to the list
                if let Some(replacers) = &mut replacers {
                    files_to_replace.append(replacers);
                }
            }
        }

        let cargo_lock = &self.config.cargo;
        if let Some(cargo_lock) = cargo_lock {
            let replacer = CargoReplacer::new(versions, cargo_lock.clone())?;
            let mut files = replacer.determine_replacements()?;
            if let Some(files) = &mut files {
                files_to_replace.append(files);
            }
        }

        if args.dry_run {
            println!("Dry run, not persisting changes");
            for replacer in files_to_replace {
                println!("Would have replaced: {}", replacer.path.display());
            }

            return Ok(());
        } else {
            for replacer in files_to_replace {
                replacer.persist()?;
            }
        }

        Ok(())
    }
}
