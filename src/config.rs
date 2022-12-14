use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::error::Result;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FileTableData {
    pub search_value: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Config {
    pub file: HashMap<PathBuf, FileTableData>,
}

impl Config {
    pub fn from_file(path: &impl AsRef<Path>) -> Result<Config> {
        let file = std::fs::read_to_string(path)?;
        let value: Config = toml::from_str(&file)?;

        Ok(value)
    }
}
