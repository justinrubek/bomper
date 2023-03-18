use crate::error::Result;

pub mod cargo;
pub mod file;
pub mod search;
pub mod simple;

use file::FileReplacer;

pub trait Replacer {
    fn overwrite_file(self) -> Result<Option<FileReplacer>>;
}

pub struct VersionReplacement {
    pub old_version: String,
    pub new_version: String,
}
