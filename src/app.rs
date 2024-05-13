use crate::cli::{BaseArgs, Bump, RawBump};
use bomper::{
    changelog::generate_changelog_entry,
    config::Config,
    error::{Error, Result},
    replacers::{
        cargo::CargoReplacer, file::FileReplacer, search::SearchReplacer, simple::SimpleReplacer,
        Replacer, VersionReplacement,
    },
    versioning::{
        determine_increment, get_commits_since_tag, get_latest_tag, increment_version,
        VersionIncrement,
    },
};
use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use std::{fmt, io::Write};

pub struct App {
    pub args: BaseArgs,
    pub config: Config,
}

impl App {
    pub fn new(args: BaseArgs, config: Config) -> App {
        App { args, config }
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
        let changelog_entry = generate_changelog_entry(&commits, &new_version);

        let replacement = VersionReplacement {
            old_version: tag.version.to_string(),
            new_version: new_version.to_string(),
        };
        let mut file_changes = determine_changes(&self.config, &replacement)?;
        file_changes.push(apply_changelog(changelog_entry)?);
        apply_changes(file_changes, &self.args)?;

        Ok(())
    }

    pub fn raw_bump(&self, opts: &RawBump) -> Result<()> {
        let replacement = VersionReplacement {
            old_version: opts.old_version.clone(),
            new_version: opts.new_version.clone(),
        };
        let file_changes = determine_changes(&self.config, &replacement)?;
        apply_changes(file_changes, &self.args)?;

        Ok(())
    }
}

/// Persist file changes to the filesystem.
/// This function is responsible for respecting the `dry_run` flag, so it will only persist changes
/// if the flag is not set.
fn apply_changes(changes: Vec<FileReplacer>, args: &BaseArgs) -> Result<()> {
    struct Line(Option<usize>);

    impl fmt::Display for Line {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0 {
                None => write!(f, "    "),
                Some(idx) => write!(f, "{:<4}", idx + 1),
            }
        }
    }

    if args.dry_run {
        println!("Dry run, not persisting changes");
        for replacer in changes {
            let original = std::fs::read_to_string(&replacer.path)?;
            let new = std::fs::read_to_string(&replacer.temp_file)?;

            println!("{}", style(replacer.path.display()).cyan());
            let (_, w) = console::Term::stdout().size();
            // write `─` for the width of the terminal
            println!("{:─^1$}", style("─").cyan(), w as usize);

            let diff = TextDiff::from_lines(&original, &new);
            for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
                if idx > 0 {
                    println!("{:-^1$}", "-", 80);
                }
                for op in group {
                    for change in diff.iter_inline_changes(op) {
                        let (sign, s) = match change.tag() {
                            ChangeTag::Delete => ("-", Style::new().red()),
                            ChangeTag::Insert => ("+", Style::new().green()),
                            ChangeTag::Equal => (" ", Style::new().dim()),
                        };
                        print!(
                            "{}{} |{}",
                            style(Line(change.old_index())).dim(),
                            style(Line(change.new_index())).dim(),
                            s.apply_to(sign).bold(),
                        );
                        for (emphasized, value) in change.iter_strings_lossy() {
                            if emphasized {
                                print!("{}", s.apply_to(value).underlined().on_black());
                            } else {
                                print!("{}", s.apply_to(value));
                            }
                        }
                        if change.missing_newline() {
                            println!();
                        }
                    }
                }
            }
        }

        return Ok(());
    } else {
        for replacer in changes {
            replacer.persist()?;
        }
    }

    Ok(())
}

/// Determine the changes to make to the repository to update the version.
fn determine_changes(
    config: &Config,
    replacement: &VersionReplacement,
) -> Result<Vec<FileReplacer>> {
    let mut files_to_replace = Vec::new();

    let by_file = &config.by_file;
    if let Some(by_file) = by_file {
        for (path, config) in by_file {
            let mut replacers = match &config.search_value {
                Some(value) => SearchReplacer::new(
                    path.clone(),
                    &replacement.old_version,
                    value,
                    &replacement.new_version,
                )?
                .determine_replacements()?,
                None => SimpleReplacer::new(
                    path.clone(),
                    &replacement.old_version,
                    &replacement.new_version,
                )?
                .determine_replacements()?,
            };

            // append new replacers to the list
            if let Some(replacers) = &mut replacers {
                files_to_replace.append(replacers);
            }
        }
    }

    let cargo_lock = &config.cargo;
    if let Some(cargo_lock) = cargo_lock {
        let replacer = CargoReplacer::new(replacement.clone(), cargo_lock.clone())?;
        let mut files = replacer.determine_replacements()?;
        if let Some(files) = &mut files {
            files_to_replace.append(files);
        }
    }

    Ok(files_to_replace)
}

/// Stitch together the existing changelog with the new one.
/// This is done using `- - -` as a marker character.
/// The new changelog is composed of the changelog header (everything from the start to the first
/// marker`, the new entry (with a marker on top), and the remaining part of the previous changelog
fn create_changelog(path: &std::path::Path, contents: &str) -> Result<String> {
    const MARKER: &str = "- - -";

    match std::path::Path::try_exists(path) {
        Ok(true) => {
            let original_changelog = std::fs::read_to_string(path)?;
            let start = original_changelog
                .find(MARKER)
                .ok_or(Error::ChangelogMarker)?;

            let header = &original_changelog[..start];
            let rest = &original_changelog[start..];
            Ok(format!("{header}\n{MARKER}\n{contents}\n{rest}"))
        }
        Ok(false) => Ok(format!("# Changelog\n\n{MARKER}\n{contents}")),
        Err(e) => Err(e.into()),
    }
}

fn apply_changelog(entry: String) -> Result<FileReplacer> {
    let path = std::path::PathBuf::from("CHANGELOG.md");
    let new_changelog = create_changelog(&path, &entry)?;

    let temp_file = tempfile::NamedTempFile::new_in(".")?;
    let mut file = temp_file.as_file();
    file.write_all(new_changelog.as_bytes())?;

    Ok(FileReplacer { path, temp_file })
}
