use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
pub struct Args {
    #[clap(short, long)]
    pub config_file: Option<PathBuf>,
    #[clap(short, long)]
    pub dry_run: bool,

    pub old_version: String,
    pub new_version: String,
}
