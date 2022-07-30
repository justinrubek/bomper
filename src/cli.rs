use clap::Parser;

#[derive(Parser, Debug)]
#[clap()]
pub struct Args {
    pub old_version: String,
    pub new_version: String,
}
