use std::fs;
use std::path::PathBuf;

use crate::error::Result;

/// A replacer that contains a temporary file and a path it may be persisted to
#[derive(Debug)]
pub struct Replacer {
    pub path: PathBuf,
    pub temp_file: tempfile::NamedTempFile,
}

impl Replacer {
    /// Persists the pending changes to the file, overwriting its contents
    ///
    /// # Errors
    ///
    /// - `self.path` does not exist
    /// - a non-final component of `self.path` is not a directory
    /// - if `self.temp_file` cannot be persisted to `self.path`
    pub fn persist(self) -> Result<()> {
        let path = fs::canonicalize(&self.path)?;
        self.temp_file.persist(path)?;

        Ok(())
    }
}
