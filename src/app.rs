use crate::cli::{BaseArgs, Bump, RawBump};
use bomper::{
    changelog::generate_changelog_entry,
    config::Config,
    error::{Error, Result},
    replacers::{
        cargo::CargoReplacer, search::SearchReplacer, simple::SimpleReplacer, Replacer,
        VersionReplacement,
    },
    versioning::{
        determine_increment, get_commits_since_tag, get_latest_tag, increment_version,
        VersionIncrement,
    },
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
    pub fn bump(&self, opts: &Bump) -> Result<()> {
        let repo = gix::discover(".")?;

        let tag = get_latest_tag(&repo)?;
        let commits = get_commits_since_tag(&repo, &tag)?;

        let increment = match &opts.options.version {
            Some(version) => VersionIncrement::Manual(semver::Version::parse(version)?),
            None if opts.options.automatic => {
                let conventional_commits = commits.iter().map(|c| c.as_ref());
                determine_increment(conventional_commits, &tag.version)
            }
            None if opts.options.major => VersionIncrement::Major,
            None if opts.options.minor => VersionIncrement::Minor,
            None if opts.options.patch => VersionIncrement::Patch,
            _ => unreachable!(),
        };
        let new_version = increment_version(tag.version.clone(), increment);
        println!("New version: {}", new_version);
        let changelog_entry = generate_changelog_entry(&commits, &new_version);
        println!("{}", changelog_entry);
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
