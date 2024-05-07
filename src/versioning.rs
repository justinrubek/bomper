use crate::error::{Error, Result};
use conventional_commit_parser::commit::ConventionalCommit;

#[derive(Clone, Debug, Eq)]
pub struct Tag {
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
            Some(Tag { version })
        })
        .max()
        .ok_or_else(|| Error::TagError)?;

    Ok(tag)
}

pub fn determine_increment(_commits: Vec<ConventionalCommit>) -> VersionIncrement {
    todo!()
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
