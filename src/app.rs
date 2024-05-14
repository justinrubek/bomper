use crate::cli::{BaseArgs, Bump, Changelog, RawBump};
use bomper::{
    changelog::generate_changelog_entry,
    config::Config,
    error::{Error, Result},
    replacers::{
        cargo::CargoReplacer, file::FileReplacer, search::SearchReplacer, simple::SimpleReplacer,
        Replacer, VersionReplacement,
    },
    versioning::{get_commits_since_tag, get_latest_tag, increment_version},
};
use console::{style, Style};
use gix::refs::transaction::PreviousValue;
use similar::{ChangeTag, TextDiff};
use std::{fmt, io::Write, path::PathBuf};

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

        let increment = opts.options.determine_increment(&commits, &tag.version)?;
        let new_version = increment_version(tag.version.clone(), increment);
        let changelog_entry = generate_changelog_entry(&commits, &new_version);

        let replacement = VersionReplacement {
            old_version: tag.version.to_string(),
            new_version: new_version.to_string(),
        };
        let mut file_changes = determine_changes(&self.config, &replacement)?;
        file_changes.push(apply_changelog(changelog_entry)?);
        if let Some(changes) = apply_changes(file_changes, &self.args)? {
            let new_tree = prepare_commit(&repo, changes)?;
            let object_id = repo.write_object(&new_tree)?;
            let commit = repo.commit(
                "HEAD",
                format!("chore(version): {new_version}"),
                object_id,
                vec![repo.head_id()?],
            )?;
            repo.tag_reference(new_version.to_string(), commit, PreviousValue::MustNotExist)?;
        }

        Ok(())
    }

    pub fn changelog(&self, opts: &Changelog) -> Result<()> {
        let repo = gix::discover(".")?;
        let tag = get_latest_tag(&repo)?;
        let commits = get_commits_since_tag(&repo, &tag)?;
        let increment = opts.options.determine_increment(&commits, &tag.version)?;
        let new_version = increment_version(tag.version.clone(), increment);
        let changelog_entry = generate_changelog_entry(&commits, &new_version);
        let path = std::path::PathBuf::from("CHANGELOG.md");
        let old_changelog = std::fs::read_to_string(&path).unwrap_or_default();
        let new_changelog = create_changelog(&path, &changelog_entry)?;
        if opts.no_decorations {
            match opts.only_current_version {
                true => println!("{}", changelog_entry),
                false => println!("{}", new_changelog),
            }
        } else {
            print_diff(old_changelog, new_changelog, path.display().to_string());
        }
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
fn apply_changes(changes: Vec<FileReplacer>, args: &BaseArgs) -> Result<Option<Vec<PathBuf>>> {
    if args.dry_run {
        println!("Dry run, not persisting changes");
        for replacer in changes {
            let original = std::fs::read_to_string(&replacer.path)?;
            let new = std::fs::read_to_string(&replacer.temp_file)?;

            print_diff(original, new, replacer.path.display().to_string())
        }

        Ok(None)
    } else {
        let replaced_files: Vec<PathBuf> = changes.iter().map(|r| r.path.clone()).collect();
        for replacer in changes {
            replacer.persist()?;
        }

        Ok(Some(replaced_files))
    }
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
            Ok(format!("{header}{MARKER}\n\n{contents}\n{rest}"))
        }
        Ok(false) => Ok(format!("# Changelog\n\n{MARKER}\n\n{contents}\n\n{MARKER}\n\ngenerated by [bomper](https://github.com/justinrubek/bomper)")),
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

fn prepare_commit(
    repo: &gix::Repository,
    changes: Vec<PathBuf>,
) -> Result<gix::worktree::object::Tree> {
    let head = repo.head_commit()?;
    let tree: gix::worktree::object::Tree = head.tree()?.decode()?.into();
    let new_tree = rewrite_tree(repo, &tree.clone(), &changes)?;
    Ok(new_tree)
}

/// Creates a modified git tree with the provided path's entries changed to match their new
/// contents.
///
/// TODO: remove this once `gix` supports a better way to create changes
fn rewrite_tree(
    repo: &gix::Repository,
    tree: &gix::worktree::object::Tree,
    changes: &Vec<PathBuf>,
) -> Result<gix::worktree::object::Tree> {
    let mut new_entries = vec![];

    for entry in &tree.entries {
        let object: gix::Object = repo.find_object(entry.oid)?;
        match &object.kind {
            gix::object::Kind::Tree => {
                let old_tree = object.clone().into_tree().decode()?.into();
                let new_tree = rewrite_tree(repo, &old_tree, changes)?;
                let new_id = repo.write_object(&new_tree)?;

                new_entries.push(gix::worktree::object::tree::Entry {
                    filename: entry.filename.clone(),
                    mode: entry.mode,
                    oid: new_id.into(),
                });
            }
            gix::object::Kind::Blob => {
                let file_path: PathBuf = entry.filename.clone().to_string().into();
                if let Some(new_path) = changes.iter().find(|p| **p == file_path) {
                    println!("replacing {:?}", new_path);
                    let new_id = repo.write_blob_stream(std::fs::File::open(new_path)?)?;

                    new_entries.push(gix::worktree::object::tree::Entry {
                        filename: entry.filename.clone(),
                        mode: entry.mode,
                        oid: new_id.into(),
                    });
                } else {
                    new_entries.push(entry.clone());
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(gix::worktree::object::Tree {
        entries: new_entries,
    })
}

fn print_diff(original: String, new: String, context: String) {
    struct Line(Option<usize>);

    impl fmt::Display for Line {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0 {
                None => write!(f, "    "),
                Some(idx) => write!(f, "{:<4}", idx + 1),
            }
        }
    }

    println!("\n{}", style(context).cyan());
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
