use bomper::error::Result;
use bomper::versioning::{determine_increment, Commit, VersionIncrement};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, about, version)]
pub struct Args {
    #[clap(flatten)]
    pub base_args: BaseArgs,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(clap::Args, Debug)]
pub(crate) struct BaseArgs {
    #[arg(short, long)]
    pub config_file: Option<PathBuf>,
    #[arg(short, long)]
    pub dry_run: bool,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum Commands {
    /// bump versions in files without doing anything else
    RawBump(RawBump),
    /// bump versions in files and commit the changes
    Bump(Bump),
    /// generate a changelog
    Changelog(Changelog),
}

#[derive(clap::Args, Debug)]
pub(crate) struct RawBump {
    pub old_version: String,
    pub new_version: String,
}

#[derive(clap::Args, Debug)]
pub(crate) struct Bump {
    #[clap(flatten)]
    pub options: BumpOptions,
}

#[derive(clap::Args, Debug)]
pub(crate) struct Changelog {
    /// the version to start generate the changelog for.
    #[arg(long)]
    pub at: Option<semver::Version>,

    /// output the changelog in plain style, with no decorations.
    #[arg(short, long)]
    pub no_decorations: bool,
    /// only include entries for commits since the last version.
    /// only valid when `no_decorations` is set.
    #[arg(short, long, requires = "no_decorations")]
    pub only_current_version: bool,
}

#[derive(clap::Args, Debug)]
#[command(group = clap::ArgGroup::new("bump-type").required(true))]
pub(crate) struct BumpOptions {
    #[arg(short, long, group = "bump-type")]
    pub version: Option<String>,
    #[arg(short, long, group = "bump-type")]
    pub automatic: bool,
    #[arg(short = 'M', long, group = "bump-type")]
    pub major: bool,
    #[arg(short, long, group = "bump-type")]
    pub minor: bool,
    #[arg(short, long, group = "bump-type")]
    pub patch: bool,
}

impl BumpOptions {
    pub(crate) fn determine_increment<'a, I: IntoIterator<Item = &'a Commit>>(
        &self,
        commits: I,
        current_version: &semver::Version,
    ) -> Result<VersionIncrement> {
        match &self.version {
            Some(version) => Ok(VersionIncrement::Manual(semver::Version::parse(version)?)),
            None if self.automatic => {
                let conventional_commits = commits.into_iter().map(|c| c.as_ref());
                Ok(determine_increment(conventional_commits, current_version))
            }
            None if self.major => Ok(VersionIncrement::Major),
            None if self.minor => Ok(VersionIncrement::Minor),
            None if self.patch => Ok(VersionIncrement::Patch),
            _ => unreachable!(),
        }
    }
}
