use std::fs;
use std::path::PathBuf;

use crate::error::Result;

/// A replacer that contains a temporary file and a path it may be persisted to
#[derive(Debug)]
pub struct FileReplacer {
    pub path: PathBuf,
    pub temp_file: tempfile::NamedTempFile,
}

impl FileReplacer {
    /// Persists the pending changes to the file, overwriting its contents
    pub fn persist(self) -> Result<()> {
        let path = fs::canonicalize(&self.path)?;
        self.temp_file.persist(path)?;

        Ok(())
    }
}
