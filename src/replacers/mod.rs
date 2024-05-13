use crate::error::Result;

pub mod cargo;
pub mod file;
pub mod search;
pub mod simple;

use file::FileReplacer;

pub trait Replacer {
    fn determine_replacements(self) -> Result<Option<Vec<FileReplacer>>>;
}

#[derive(Clone, Debug)]
pub struct VersionReplacement {
    pub old_version: String,
    pub new_version: String,
}
