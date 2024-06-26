use std::collections::HashMap;

use crate::error::Result;
use conventional_commit_parser::commit::{CommitType, ConventionalCommit};

#[derive(Clone, Debug, Eq)]
pub struct Tag {
    pub commit_id: gix::ObjectId,
    pub version: semver::Version,
    pub prefix_v: bool,
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd<Tag> for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.version.cmp(&other.version))
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl Tag {
    /// Returns the version of the tag as a string.
    /// If the tag is prefixed with a 'v', the 'v' is included in the string.
    #[must_use]
    pub fn version(&self) -> String {
        if self.prefix_v {
            format!("v{}", self.version)
        } else {
            self.version.to_string()
        }
    }

    /// Create a new `Tag` with the version incremented by the given increment.
    /// Note: this does not change the commit id, only the version.
    #[must_use]
    pub fn increment_version(&self, increment: VersionIncrement) -> Self {
        let mut new = self.clone();
        new.version = increment_version(self.version.clone(), increment);
        new
    }

    /// # Errors
    ///
    /// - `gitoxide` is unable to read the repository references or tags
    pub fn get_version_tags(repo: &gix::Repository) -> Result<Vec<Tag>> {
        // TODO: should we only look for tags that are from the current branch?
        // TODO: should we ignore tags that are not semver?
        let references = repo.references()?;
        let tags = references.tags()?;
        let tags = tags
            .filter_map(|tag| {
                let tag = tag.ok()?;
                let name = tag.name().shorten().to_string();
                let (version, prefix_v) = if let Some(stripped) = name.strip_prefix("v") {
                    (semver::Version::parse(stripped).ok()?, true)
                } else {
                    (semver::Version::parse(&name).ok()?, false)
                };
                let commit_id = tag.id().into();
                Some(Tag {
                    commit_id,
                    version,
                    prefix_v,
                })
            })
            .collect();

        Ok(tags)
    }
}

#[derive(Clone, Debug)]
pub struct Commit {
    pub commit_id: gix::ObjectId,
    pub conventional_commit: ConventionalCommit,
    pub signature: gix::actor::Signature,
}

impl AsRef<ConventionalCommit> for Commit {
    fn as_ref(&self) -> &ConventionalCommit {
        &self.conventional_commit
    }
}

#[derive(Debug)]
pub enum VersionIncrement {
    Manual(semver::Version),
    Major,
    Minor,
    Patch,
}

/// # Errors
///
/// - `gitoxide` is unable to read the repository references or tags
pub fn get_latest_tag(repo: &gix::Repository) -> Result<Option<Tag>> {
    let tag = Tag::get_version_tags(repo)?.into_iter().max();
    Ok(tag)
}

/// # Errors
///
/// - `gitoxide` is unable to read the repository references or tags
pub fn get_tags(
    repo: &gix::Repository,
    versions: &[semver::Version],
) -> Result<HashMap<semver::Version, Tag>> {
    let tags = Tag::get_version_tags(repo)?;
    let tags = tags
        .into_iter()
        .filter(|tag| versions.contains(&tag.version))
        .map(|tag| (tag.version.clone(), tag))
        .collect();
    Ok(tags)
}

/// # Errors
///
/// - the repository has no commits
/// - git HEAD is not a commit
/// - `gitoxide` is unable to traverse the commit history
pub fn get_commits_since_tag(repo: &gix::Repository, tag: &Tag) -> Result<Vec<Commit>> {
    let head = repo.head_commit()?;
    let ancestors = head.ancestors();
    let mut parsed_commits = Vec::new();
    for commit in ancestors.all()? {
        let commit = commit?;
        let object = commit.object()?;
        if commit.id() == tag.commit_id {
            break;
        }
        let message = object.message()?;
        let mut full_message = String::new();
        full_message.push_str(message.title.to_string().trim());
        if let Some(body) = message.body {
            full_message.push_str("\n\n");
            full_message.push_str(&body.to_string());
        }
        let parsed = conventional_commit_parser::parse(&full_message)?;
        parsed_commits.push(Commit {
            commit_id: commit.id().into(),
            conventional_commit: parsed,
            signature: object.author()?.into(),
        });
    }

    Ok(parsed_commits)
}

/// # Errors
///
/// - the repository has no commits
/// - git HEAD is not a commit
/// - `gitoxide` is unable to traverse the commit history
/// - a commit message is found that is not a valid conventional commit
pub fn get_commits_since_initial_commit(repo: &gix::Repository) -> Result<Vec<Commit>> {
    let head = repo.head_commit()?;
    let ancestors = head.ancestors();
    let mut parsed_commits = Vec::new();
    for commit in ancestors.all()? {
        let commit = commit?;
        let object = commit.object()?;
        let message = object.message()?;
        let mut full_message = String::new();
        full_message.push_str(message.title.to_string().trim());
        if let Some(body) = message.body {
            full_message.push_str("\n\n");
            full_message.push_str(&body.to_string());
        }
        let parsed = conventional_commit_parser::parse(&full_message)?;
        parsed_commits.push(Commit {
            commit_id: commit.id().into(),
            conventional_commit: parsed,
            signature: object.author()?.into(),
        });
    }

    Ok(parsed_commits)
}

/// # Errors
///
/// - the repository has no commits
/// - git HEAD is not a commit
/// - `gitoxide` is unable to traverse the commit history
/// - a commit message is found that is not a valid conventional commit
pub fn get_commits_between_tags(
    repo: &gix::Repository,
    from: &Tag,
    to: &Tag,
) -> Result<Vec<Commit>> {
    let start = repo.find_object(to.commit_id)?.into_commit();
    let ancestors = start.ancestors();
    let mut parsed_commits = Vec::new();
    for commit in ancestors.all()? {
        let commit = commit?;
        let object = commit.object()?;
        if commit.id() == from.commit_id {
            break;
        }
        let message = object.message()?;
        let mut full_message = String::new();
        full_message.push_str(message.title.to_string().trim());
        if let Some(body) = message.body {
            full_message.push_str("\n\n");
            full_message.push_str(&body.to_string());
        }
        let parsed = conventional_commit_parser::parse(&full_message)?;
        parsed_commits.push(Commit {
            commit_id: commit.id().into(),
            conventional_commit: parsed,
            signature: object.author()?.into(),
        });
    }

    Ok(parsed_commits)
}

pub fn determine_increment<'a, I: IntoIterator<Item = &'a ConventionalCommit>>(
    commits: I,
    current_version: &semver::Version,
) -> VersionIncrement {
    let (has_breaking, has_feature) =
        commits
            .into_iter()
            .fold((false, false), |(has_breaking, has_feature), commit| {
                (
                    has_breaking || commit.is_breaking_change,
                    has_feature || commit.commit_type == CommitType::Feature,
                )
            });
    if has_breaking {
        match current_version.major {
            0 => VersionIncrement::Minor,
            _ => VersionIncrement::Major,
        }
    } else if has_feature {
        VersionIncrement::Minor
    } else {
        VersionIncrement::Patch
    }
}

#[must_use]
pub fn increment_version(
    mut version: semver::Version,
    increment: VersionIncrement,
) -> semver::Version {
    match increment {
        VersionIncrement::Manual(version) => version,
        VersionIncrement::Major => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
            version.build = semver::BuildMetadata::EMPTY;
            version.pre = semver::Prerelease::EMPTY;
            version
        }
        VersionIncrement::Minor => {
            version.minor += 1;
            version.patch = 0;
            version.build = semver::BuildMetadata::EMPTY;
            version.pre = semver::Prerelease::EMPTY;
            version
        }
        VersionIncrement::Patch => {
            version.patch += 1;
            version.build = semver::BuildMetadata::EMPTY;
            version.pre = semver::Prerelease::EMPTY;
            version
        }
    }
}
