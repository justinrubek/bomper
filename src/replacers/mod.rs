use crate::error::Result;

pub mod cargo;
pub mod file;
pub mod search;
pub mod simple;

use file::Replacer;

pub trait ReplacementBuilder {
    /// # Errors
    ///
    /// - determined by the implementation
    fn determine_replacements(self) -> Result<Option<Vec<Replacer>>>;
}

#[derive(Clone, Debug)]
pub struct VersionReplacement {
    pub old_version: String,
    pub new_version: String,
}
