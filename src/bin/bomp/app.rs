use crate::cli::{Bump, Changelog, RawBump};
use bomper::{
    changelog::generate_changelog_entry,
    config::Config,
    error::{Error, Result},
    replacers::{cargo, file, search, simple, ReplacementBuilder, VersionReplacement},
    versioning::{get_commits_between_tags, get_commits_since_tag, get_latest_tag, Commit, Tag},
};
use console::{style, Style};
use gix::refs::transaction::PreviousValue;
use similar::{ChangeTag, TextDiff};
use std::{fmt, io::Write, path::PathBuf, process::Command};

pub struct App {
    pub config: Config,
}

impl App {
    pub fn new(config: Config) -> App {
        App { config }
    }
}

impl App {
    pub fn bump(&self, opts: &Bump) -> Result<()> {
        let repo = gix::discover(".")?;

        let (tag, commits) = changelog_commits(&repo)?;

        let increment = opts.options.determine_increment(&commits, &tag.version)?;
        let new_tag = tag.increment_version(increment);
        let version_description = if opts.comment {
            if let Some(description) = prompt_for_description()? {
                Some(description)
            } else {
                println!("Aborting bump due to empty description");
                return Ok(());
            }
        } else {
            None
        };
        let new_version_string = new_tag.version();
        let changelog_entry = generate_changelog_entry(
            &repo,
            &commits,
            &new_version_string,
            version_description,
            self.config.authors.as_ref(),
        )?;

        let replacement = VersionReplacement {
            old_version: tag.version.to_string(),
            new_version: new_tag.version.to_string(),
        };
        let mut file_changes = determine_changes(&self.config, &replacement)?;
        file_changes.push(apply_changelog(&changelog_entry)?);
        if let Some(changes) = apply_changes(file_changes, opts.dry_run)? {
            let object_id = prepare_commit(&repo, &changes)?;
            let commit = repo.commit(
                "HEAD",
                format!("chore(version): {}", new_tag.version),
                object_id,
                vec![repo.head_id()?],
            )?;
            repo.tag_reference(new_version_string, commit, PreviousValue::MustNotExist)?;
        }

        Ok(())
    }

    pub fn changelog(&self, opts: &Changelog) -> Result<()> {
        let repo = gix::discover(".")?;
        if let Some(version) = &opts.at {
            let mut tags = Tag::get_version_tags(&repo)?;
            tags.sort();
            tags.reverse();
            let version_range = tags
                .windows(2)
                .find(|tags| {
                    let [first, _] = tags else { unreachable!() };
                    first.version.eq(version)
                })
                .ok_or_else(|| Error::VersionNotFound(version.clone()))?;
            let commits = get_commits_between_tags(&repo, &version_range[1], &version_range[0])?;
            let changelog_entry = generate_changelog_entry(
                &repo,
                &commits,
                &version.to_string(),
                None,
                self.config.authors.as_ref(),
            )?;
            println!("{changelog_entry}");
        } else {
            let (_, commits) = changelog_commits(&repo)?;
            let changelog_entry = generate_changelog_entry(
                &repo,
                &commits,
                "unreleased",
                None,
                self.config.authors.as_ref(),
            )?;
            let path = std::path::PathBuf::from("CHANGELOG.md");
            if opts.no_decorations {
                if opts.only_current_version {
                    println!("{changelog_entry}");
                } else {
                    let new_changelog = create_changelog(&path, &changelog_entry)?;
                    println!("{new_changelog}");
                }
            } else {
                let old_changelog = std::fs::read_to_string(&path).unwrap_or_default();
                let new_changelog = create_changelog(&path, &changelog_entry)?;
                print_diff(&old_changelog, &new_changelog, path.display().to_string());
            }
        }

        Ok(())
    }

    pub fn raw_bump(&self, opts: &RawBump) -> Result<()> {
        let replacement = VersionReplacement {
            old_version: opts.old_version.clone(),
            new_version: opts.new_version.clone(),
        };
        let file_changes = determine_changes(&self.config, &replacement)?;
        apply_changes(file_changes, opts.dry_run)?;

        Ok(())
    }
}

/// Persist file changes to the filesystem.
/// This function is responsible for respecting the `dry_run` flag, so it will only persist changes
/// if the flag is not set.
fn apply_changes(changes: Vec<file::Replacer>, dry_run: bool) -> Result<Option<Vec<PathBuf>>> {
    if dry_run {
        println!("Dry run, not persisting changes");
        for replacer in changes {
            let original = std::fs::read_to_string(&replacer.path).unwrap_or_default();
            let new = std::fs::read_to_string(&replacer.temp_file)?;

            print_diff(&original, &new, replacer.path.display().to_string());
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
) -> Result<Vec<file::Replacer>> {
    let mut files_to_replace = Vec::new();

    let by_file = &config.by_file;
    if let Some(by_file) = by_file {
        for (path, config) in by_file {
            let mut replacers = match &config.search_value {
                Some(value) => search::Replacer::new(
                    path.clone(),
                    &replacement.old_version,
                    value,
                    &replacement.new_version,
                )?
                .determine_replacements()?,
                None => simple::Replacer::new(
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
        let replacer = cargo::Replacer::new(replacement.clone(), cargo_lock.clone());
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
/// marker, the new entry (with a marker on top), and the remaining part of the previous changelog
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

fn apply_changelog(entry: &str) -> Result<file::Replacer> {
    let path = std::path::PathBuf::from("CHANGELOG.md");
    let new_changelog = create_changelog(&path, entry)?;

    let temp_file = tempfile::NamedTempFile::new_in(".")?;
    let mut file = temp_file.as_file();
    file.write_all(new_changelog.as_bytes())?;

    Ok(file::Replacer { path, temp_file })
}

fn prepare_commit(repo: &gix::Repository, changes: &[PathBuf]) -> Result<gix::ObjectId> {
    let head = repo.head_commit()?;
    let mut editor = repo.edit_tree(head.id)?;
    for file_path in changes {
        let gix_path = file_path.to_str().unwrap();
        let new_id = repo.write_blob_stream(std::fs::File::open(file_path)?)?;
        editor.upsert(gix_path, gix::object::tree::EntryKind::Blob, new_id)?;
    }
    let new_tree = editor.write()?;
    Ok(new_tree.detach())
}

fn print_diff(original: &str, new: &str, context: String) {
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

    let diff = TextDiff::from_lines(original, new);
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

const DESCRIPTION_HELP: &[u8] = br#"
# Please enter a description for this version change"
# All lines starting with '#' will be ignored
# The contents of this file will be inserted into the changelog markdown"#;

fn prompt_for_description() -> Result<Option<String>> {
    let mut file = tempfile::NamedTempFile::new_in(".")?;
    file.write_all(DESCRIPTION_HELP)?;

    let editor = std::env::var("EDITOR").map_err(|_| Error::EditorNotSet)?;
    Command::new(editor)
        .arg(file.path())
        .status()
        .expect("failed to edit changelog description file");

    let description = std::fs::read_to_string(file.path())?;
    let description = description
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    if description.is_empty() {
        Ok(None)
    } else {
        // check if the file only contains whitespace
        let description = description.trim();
        if description.is_empty() {
            Ok(None)
        } else {
            Ok(Some(description.to_string()))
        }
    }
}

/// Retrieve all the commits that should be included in a new changelog entry.
/// This will start at the current head commit and walk back to the latest tag.
/// The latest tag is determined by the highest semver tag in the repository.
/// If no tags are found, the root commit will be used as the starting point and a version of `0.0.0` will be used.
fn changelog_commits(repo: &gix::Repository) -> Result<(Tag, Vec<Commit>)> {
    let tag = if let Some(tag) = get_latest_tag(repo)? {
        tag
    } else {
        let head = repo.head_commit()?;
        let ancestors = head.ancestors();
        let root_commit = ancestors.all()?.last();
        Tag {
            version: semver::Version::new(0, 0, 0),
            commit_id: root_commit.unwrap().unwrap().id().into(),
            prefix_v: false,
        }
    };
    let commits = get_commits_since_tag(repo, &tag)?;
    Ok((tag, commits))
}
