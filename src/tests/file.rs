use anyhow::anyhow;
use parking_lot::Mutex;
use std::{
    fmt::Display,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use tempfile::TempDir;

use crate::error::{Error, Result};

pub struct FileJail {
    _directory: TempDir,
    canonical_path: PathBuf,
    original_cwd: PathBuf,
}

static LOCK: Mutex<()> = parking_lot::const_mutex(());

impl FileJail {
    #[track_caller]
    pub fn expect_with<F: FnOnce(&mut FileJail) -> Result<()>>(f: F) {
        if let Err(e) = FileJail::try_with(f) {
            panic!("failed to create jail: {}", e)
        }
    }

    #[track_caller]
    pub fn try_with<F: FnOnce(&mut FileJail) -> Result<()>>(f: F) -> Result<()> {
        let _lock = LOCK.lock();
        let directory = TempDir::new()?;
        let mut jail = FileJail {
            canonical_path: directory.path().canonicalize()?,
            _directory: directory,
            original_cwd: std::env::current_dir()?,
        };

        std::env::set_current_dir(jail.directory())?;
        f(&mut jail)
    }

    pub fn directory(&self) -> &Path {
        &self.canonical_path
    }

    pub fn create_file<P: AsRef<std::path::Path>>(&self, path: P, contents: &str) -> Result<File> {
        let path = path.as_ref();
        if !path.is_relative() {
            return Err(Error::Other(anyhow!(
                "FileJail::create_file: path must be relative"
            )));
        }

        // Create the parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(self.directory().join(parent))?;
        }

        let file = File::create(self.directory().join(path))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(contents.as_bytes())?;
        let file = writer.into_inner().map_err(as_string_error)?;
        Ok(file)
    }

    /// Returns the path to the file in the jail
    pub fn strip_path(&self, path: PathBuf) -> Result<String> {
        let path = path.canonicalize()?;
        if !path.starts_with(&self.canonical_path) {
            return Err(Error::Other(anyhow!(
                "FileJail::strip_path: path is not in the jail"
            )));
        }
        let path = path.strip_prefix(&self.canonical_path).unwrap();
        let path = path.to_str().ok_or_else(|| {
            Error::Other(anyhow!("FileJail::strip_path: path is not valid unicode"))
        })?;
        Ok(path.to_string())
    }
}

fn as_string_error<S: Display>(s: S) -> Error {
    Error::Other(anyhow!("{}", s.to_string()))
}

impl Drop for FileJail {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original_cwd);
    }
}
