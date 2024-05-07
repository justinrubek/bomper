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
