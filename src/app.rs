use crate::cli::{BaseArgs, Bump, RawBump};
use bomper::{
    config::Config,
    error::{Error, Result},
    replacers::{
        cargo::CargoReplacer, search::SearchReplacer, simple::SimpleReplacer, Replacer,
        VersionReplacement,
    },
    versioning::get_latest_tag,
};

pub struct App {
    pub args: BaseArgs,
    pub config: Config,
}

impl App {
    pub fn new(args: BaseArgs) -> Result<App> {
        let config = match &args.config_file {
            Some(path) => Config::from_ron(&path)?,
            None => {
                let base = project_base_directory::get_project_root()
                    .map_err(|_| Error::ProjectBaseDirectory)?;
                match base {
                    Some(base) => Config::from_ron(&base.join("bomp.ron"))?,
                    None => Config::from_ron(&String::from("bomp.ron"))?,
                }
            }
        };

        Ok(App { args, config })
    }
}

impl App {
    pub fn bump(&self, _opts: &Bump) -> Result<()> {
        let repo = gix::discover(".")?;

        let tag = get_latest_tag(&repo)?;
        todo!()
    }

    pub fn raw_bump(&self, opts: &RawBump) -> Result<()> {
        // self.config.file.clone().par_drain().for_each(|path| {
        let versions = VersionReplacement {
            old_version: opts.old_version.clone(),
            new_version: opts.new_version.clone(),
        };
        let mut files_to_replace = Vec::new();

        let by_file = &self.config.by_file;
        if let Some(by_file) = by_file {
            for (path, config) in by_file {
                let mut replacers = match &config.search_value {
                    Some(value) => SearchReplacer::new(
                        path.clone(),
                        &opts.old_version,
                        value,
                        &opts.new_version,
                    )?
                    .determine_replacements()?,
                    None => {
                        SimpleReplacer::new(path.clone(), &opts.old_version, &opts.new_version)?
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

        if self.args.dry_run {
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
