use crate::versioning::Commit;
use conventional_commit_parser::commit::CommitType;
use std::collections::HashMap;

const TEMPLATE: &str = include_str!("templates/changelog_entry.md");

#[derive(Debug, serde::Serialize)]
pub struct ChangelogEntry<'a> {
    pub version: &'a str,
    pub commits: HashMap<String, Vec<ChangelogCommit>>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
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
    commits: I,
    version: &str,
    description: Option<String>,
) -> String {
    let mut env = minijinja::Environment::new();
    env.add_template("changelog_entry", TEMPLATE).unwrap();

    let typed_commits: HashMap<String, Vec<ChangelogCommit>> =
        commits.into_iter().fold(HashMap::new(), |mut acc, commit| {
            let key = display_commit_type(&commit.conventional_commit.commit_type);
            let entry = acc.entry(key).or_default();
            let commit = ChangelogCommit {
                scope: commit.conventional_commit.scope.clone(),
                summary: commit.conventional_commit.summary.clone(),
                hash: commit.commit_id.to_string(),
                author: commit.signature.name.to_string(),
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
        .unwrap()
}
