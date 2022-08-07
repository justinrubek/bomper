use crate::error::Result;

pub mod file;
pub mod simple;
pub mod search;

use file::FileReplacer;

pub trait Replacer {
    fn overwrite_file(&self) -> Result<Option<FileReplacer>>;
}
