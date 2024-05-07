use crate::error::{Error, Result};
use conventional_commit_parser::commit::{CommitType, ConventionalCommit};

#[derive(Clone, Debug, Eq)]
pub struct Tag {
    pub commit_id: gix::ObjectId,
    pub version: semver::Version,
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

#[derive(Debug)]
pub enum VersionIncrement {
    Manual(semver::Version),
    Major,
    Minor,
    Patch,
}

pub fn get_latest_tag(repo: &gix::Repository) -> Result<Tag> {
    // TODO: should we only look for tags that are from the current branch?
    // TODO: should we ignore tags that are not semver?
    let references = repo.references()?;
    let tags = references.tags()?;
    let tag = tags
        .filter_map(|tag| {
            let tag = tag.ok()?;
            let name = tag.name().shorten().to_string();
            let version = semver::Version::parse(&name).unwrap();
            let commit_id = tag.id().into();
            Some(Tag { version, commit_id })
        })
        .max()
        .ok_or_else(|| Error::TagError)?;

    Ok(tag)
}

pub fn get_commits_since_tag(repo: &gix::Repository, tag: &Tag) -> Result<Vec<ConventionalCommit>> {
    let head = repo.head_commit()?;
    let ancestors = head.ancestors();
    let mut parsed_commits = Vec::new();
    for commit in ancestors.all()? {
        let commit = commit.unwrap();
        let object = commit.object().unwrap();
        if commit.id() == tag.commit_id {
            break;
        }
        let message = object.message().unwrap();
        let mut full_message = String::new();
        full_message.push_str(message.title.to_string().trim());
        if let Some(body) = message.body {
            full_message.push_str("\n\n");
            full_message.push_str(&body.to_string());
        }
        let parsed = conventional_commit_parser::parse(&full_message)?;
        parsed_commits.push(parsed);
    }

    Ok(parsed_commits)
}

pub fn determine_increment(
    commits: &[ConventionalCommit],
    current_version: &semver::Version,
) -> VersionIncrement {
    let has_breaking = commits.iter().any(|commit| commit.is_breaking_change);
    if has_breaking {
        match current_version.major {
            0 => VersionIncrement::Minor,
            _ => VersionIncrement::Major,
        }
    } else {
        let has_feature = commits
            .iter()
            .any(|commit| commit.commit_type == CommitType::Feature);
        if has_feature {
            VersionIncrement::Minor
        } else {
            VersionIncrement::Patch
        }
    }
}

pub fn increment_version(
    mut version: semver::Version,
    increment: VersionIncrement,
) -> semver::Version {
    match increment {
        VersionIncrement::Manual(version) => version,
        VersionIncrement::Major => {
            version.major += 1;
            version
        }
        VersionIncrement::Minor => {
            version.minor += 1;
            version
        }
        VersionIncrement::Patch => {
            version.patch += 1;
            version
        }
    }
}
