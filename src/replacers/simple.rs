use memmap::{Mmap, MmapMut};
use std::{fs, fs::File, io::prelude::*, ops::DerefMut, path::PathBuf};

use super::{file, ReplacementBuilder};
use crate::error::{Error, Result};

pub struct Replacer {
    path: PathBuf,
    new_data: Vec<u8>,
    regex: regex::bytes::Regex,
}

impl Replacer {
    /// # Errors
    ///
    /// - the regex expression compiled from `old_content` is larger than the default size limit of `regex::bytes::RegexBuilder`
    pub fn new(path: PathBuf, old_content: &str, new_content: &str) -> Result<Self> {
        let search_text = regex::escape(old_content);
        let regex = regex::bytes::RegexBuilder::new(&search_text).build()?;

        Ok(Self {
            path,
            regex,
            new_data: String::from(new_content).into_bytes(),
        })
    }
}

impl ReplacementBuilder for Replacer {
    fn determine_replacements(self) -> Result<Option<Vec<file::Replacer>>> {
        let mut replacers = Vec::new();

        let source_file = File::open(&self.path)?;
        let source_meta = fs::metadata(&self.path)?;
        let source_map = unsafe { Mmap::map(&source_file)? };

        // Replace occurences of old_content with new_content in source_map.
        let replaced = replace(&self.regex, &source_map, &self.new_data);

        let temp_file = tempfile::NamedTempFile::new_in(
            (self.path)
                .parent()
                .ok_or_else(|| Error::InvalidPath(self.path.clone()))?,
        )?;
        let file = temp_file.as_file();
        file.set_len(replaced.len() as u64)?;
        file.set_permissions(source_meta.permissions())?;

        if !replaced.is_empty() {
            let mut target_map = unsafe { MmapMut::map_mut(file)? };
            target_map.deref_mut().write_all(&replaced)?;
            target_map.flush_async()?;
        }

        drop(source_map);
        drop(source_file);

        replacers.push(file::Replacer {
            path: self.path,
            temp_file,
        });

        Ok(Some(replacers))
    }
}

fn replace<'a>(
    regex: &regex::bytes::Regex,
    buf: &'a [u8],
    replace_with: &[u8],
) -> std::borrow::Cow<'a, [u8]> {
    regex.replacen(buf, 0, replace_with)
}
