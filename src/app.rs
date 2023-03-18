use bomper::replacers::cargo::CargoLockReplacer;

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
                let replacer = match &config.search_value {
                    Some(value) => {
                        SearchReplacer::new(path.clone(), &args.old_version, value, &args.new_version)?
                            .overwrite_file()?
                    }
                    None => SimpleReplacer::new(path.clone(), &args.old_version, &args.new_version)?
                        .overwrite_file()?,
                };

                if let Some(replacer) = replacer {
                    files_to_replace.push(replacer);
                }
            }
        }

        let cargo_lock = &self.config.cargo_lock;
        if let Some(cargo_lock) = cargo_lock {
            let replacer = CargoLockReplacer::new(versions, cargo_lock.clone())?;
            let file = replacer.overwrite_file()?.expect("Cargo.lock replacer failed");
            files_to_replace.push(file);
        }

        if args.dry_run {
            println!("Dry run, not persisting changes");
            for replacer in files_to_replace {
                println!("Would have replaced: {}", replacer.path.display());

                println!("{:#?}", replacer);
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
