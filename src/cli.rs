use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
pub struct Args {
    pub old_version: String,
    pub new_version: String,
}
