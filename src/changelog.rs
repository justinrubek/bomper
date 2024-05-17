use crate::{error::Result, versioning::Commit};
use conventional_commit_parser::commit::CommitType;
use std::collections::HashMap;

const TEMPLATE: &str = include_str!("templates/changelog_entry.md");

#[derive(Debug, serde::Serialize)]
pub struct ChangelogEntry<'a> {
    pub version: &'a str,
    pub commits: HashMap<String, Vec<ChangelogCommit>>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ChangelogCommit {
    pub scope: Option<String>,
    pub summary: String,
    pub hash: String,
    pub author: String,
}

impl std::fmt::Display for ChangelogCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.scope {
            Some(ref scope) => write!(
                f,
                "**({0})** {1} - ({2}) - {3}",
                scope, self.summary, self.hash, self.author
            ),
            None => write!(f, "{0} - ({1}) - {2}", self.summary, self.hash, self.author),
        }
    }
}

impl serde::Serialize for ChangelogCommit {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

pub fn display_commit_type(commit_type: &CommitType) -> String {
    match commit_type {
        CommitType::Feature => "features".to_string(),
        CommitType::BugFix => "bug fixes".to_string(),
        CommitType::Refactor => "refactors".to_string(),
        CommitType::Chore => "chores".to_string(),
        CommitType::Documentation => "documentation".to_string(),
        CommitType::Style => "style".to_string(),
        CommitType::Test => "tests".to_string(),
        CommitType::Build => "build system".to_string(),
        CommitType::Revert => "reverts".to_string(),
        CommitType::Ci => "continuous integration".to_string(),
        CommitType::Performances => "performance".to_string(),
        CommitType::Custom(custom) => custom.to_string(),
    }
}

pub fn generate_changelog_entry<'a, I: IntoIterator<Item = &'a Commit>>(
    repo: &gix::Repository,
    commits: I,
    version: &str,
    description: Option<String>,
    authors: &Option<HashMap<String, String>>,
) -> Result<String> {
    let mut env = minijinja::Environment::new();
    env.add_template("changelog_entry", TEMPLATE).unwrap();

    let url = gix_repo_url(repo)?;
    let version = match &url {
        Some((host, path)) => &format!("[{version}](https://{host}/{path}/releases/tag/{version})"),
        None => version,
    };

    let typed_commits: HashMap<String, Vec<ChangelogCommit>> =
        commits.into_iter().fold(HashMap::new(), |mut acc, commit| {
            let key = display_commit_type(&commit.conventional_commit.commit_type);
            let entry = acc.entry(key).or_default();
            let author = author_name(commit.signature.name.to_string(), authors, &url);
            let commit_id = commit.commit_id.to_string();
            let hash = match &url {
                Some((host, path)) => format!(
                    "[{}](https://{host}/{path}/commit/{commit_id})",
                    &commit_id[..7]
                ),
                None => commit_id,
            };

            let commit = ChangelogCommit {
                scope: commit.conventional_commit.scope.clone(),
                summary: commit.conventional_commit.summary.clone(),
                hash,
                author,
            };
            entry.push(commit);
            acc
        });
    let entry = ChangelogEntry {
        version,
        commits: typed_commits,
        description,
    };

    let template = env.get_template("changelog_entry").unwrap();
    template
        .render(minijinja::context!(
            entry => entry,
        ))
        .map_err(Into::into)
}

fn gix_repo_url(repo: &gix::Repository) -> Result<Option<(String, String)>> {
    let remote = match repo.find_default_remote(gix::remote::Direction::Push) {
        Some(remote) => remote?,
        None => return Ok(None),
    };

    match remote.url(gix::remote::Direction::Push) {
        Some(url) => {
            let host = url.host_argument_safe();
            let path = url.path_argument_safe();
            match (host, path) {
                (Some(host), Some(path)) => Ok(Some((
                    host.to_string(),
                    remove_suffix(&path.to_string(), ".git").to_string(),
                ))),
                _ => Ok(None),
            }
        }
        None => Ok(None),
    }
}

fn remove_suffix<'a>(input: &'a str, suffix: &str) -> &'a str {
    if let Some(stripped) = input.strip_suffix(suffix) {
        stripped
    } else {
        input
    }
}

fn author_name(
    commit_author: String,
    authors: &Option<HashMap<String, String>>,
    url: &Option<(String, String)>,
) -> String {
    match url {
        Some((host, _)) => {
            if let Some(authors) = authors {
                authors
                    .get(&commit_author)
                    .cloned()
                    .map(|author| format!("[@{author}](https://{host}/{author})"))
                    .unwrap_or(commit_author)
            } else {
                commit_author
            }
        }
        None => commit_author,
    }
}
