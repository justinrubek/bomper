use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::error::Result;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTableData {
    pub search_value: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
    pub by_file: Option<HashMap<PathBuf, FileTableData>>,
    #[serde(default)]
    pub cargo: Option<CargoReplaceMode>,
}

impl Config {
    pub fn from_ron(path: &impl AsRef<Path>) -> Result<Config> {
        let file = std::fs::read_to_string(path)?;
        let value: Config = ron::from_str(&file)?;

        Ok(value)
    }
}

/// Reads from the Cargo.lock file to determine which packages to bump versions for.
/// This is more reliable than a simple regex because it matches the exact package names only
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CargoReplaceMode {
    /// automatically determine package names from the Cargo workspace
    Autodetect,
    /// Manually specify package names
    Packages(Vec<String>),
}
