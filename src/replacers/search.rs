use memmap::{Mmap, MmapMut};
use std::ops::Range;
use std::{fs, fs::File, io::prelude::*, path::PathBuf};

use super::file;
use crate::error::{Error, Result};
use crate::replacers::ReplacementBuilder;

/// Replaces all instances of a given value with a new one.
/// This is a somewhat naive implementation, but it works.
/// The area surrounding the value will be checked for matches in the supplied regex
pub struct Replacer {
    path: PathBuf,
    new_data: Vec<u8>,
    regex: regex::bytes::Regex,
    verification_regex: regex::bytes::Regex,
}

impl Replacer {
    /// # Errors
    ///
    /// - `verification_regex` is not a valid regex
    /// - the regex expression compiled from `verification_regex` is larger than the default size limit of `regex::bytes::RegexBuilder`
    pub fn new(
        path: PathBuf,
        old_content: &str,
        verification_regex: &str,
        new_content: &str,
    ) -> Result<Self> {
        let search_text = regex::escape(old_content);
        let expr = format!("((.*(\\n)){{2}}).*({})", &search_text);
        let regex = regex::bytes::RegexBuilder::new(&expr).build()?;
        let verification_regex = regex::bytes::Regex::new(verification_regex)?;

        Ok(Self {
            path,
            regex,
            verification_regex,
            new_data: String::from(new_content).into_bytes(),
        })
    }

    /// Gives the positions in the buffer that need to be replaced.
    fn determine_replacement_locations(&self, source_buf: &Mmap) -> Result<Vec<Range<usize>>> {
        // Find all locations in the file with the version string found
        self.regex
            .captures_iter(source_buf)
            .map(|capture| {
                // Ensure that there is a match of the verification regex before replacing
                let first_capture = capture.get(1).unwrap();
                let result = self.verification_regex.find(first_capture.as_bytes());
                if result.is_none() {
                    return Ok(None);
                }

                // Record the offsets of the portion that is to be replaced
                let last_capture = capture.get(capture.len() - 1);
                if let Some(mtch) = last_capture {
                    Ok(Some(Range {
                        start: mtch.start(),
                        end: mtch.end(),
                    }))
                } else {
                    Ok(None)
                }
            })
            // Filter out any that are None
            .filter_map(|val| match val {
                Ok(Some(val)) => Some(Ok(val)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<_>>>()
    }

    fn get_replacement(
        self,
        source_buf: &Mmap,
        replacement_locations: &[Range<usize>],
        file_permissions: std::fs::Permissions,
    ) -> Result<file::Replacer> {
        let temp_file = tempfile::NamedTempFile::new_in(
            (self.path)
                .parent()
                .ok_or_else(|| Error::InvalidPath(self.path.clone()))?,
        )?;
        let mut file = temp_file.as_file();

        // resize the file to the size of the old one
        let replacers_removal_len = replacement_locations
            .iter()
            .fold(0, |acc, val| acc + val.end - val.start);
        let new_value_len = self.new_data.len() * replacement_locations.len();
        let file_len = source_buf.len() - replacers_removal_len + new_value_len;
        file.set_len(file_len as u64)?;
        file.set_permissions(file_permissions)?;

        let mut target_map = unsafe { MmapMut::map_mut(file)? };

        // use the offset ranges to replace old_content with new_content in source_buf
        match replacement_locations.len() {
            1 => {
                let start = replacement_locations[0].start;
                let end = replacement_locations[0].end;

                let mut writer = &mut *target_map;
                writer.write_all(&source_buf[0..start])?;
                writer.write_all(&self.new_data)?;
                writer.write_all(&source_buf[end..])?;
            }
            val if val > 1 => {
                let mut writer = &mut *target_map;
                let mut prev_end = 0;
                for range in replacement_locations {
                    let start = range.start;
                    let end = range.end;
                    writer.write_all(&source_buf[prev_end..start])?;
                    writer.write_all(&self.new_data)?;
                    prev_end = end;
                }

                writer.write_all(&source_buf[prev_end..])?;
            }
            val => {
                return Err(Error::InvalidReplacementCount(val));
            }
        }

        file.flush()?;

        Ok(file::Replacer {
            path: self.path,
            temp_file,
        })
    }
}

impl ReplacementBuilder for Replacer {
    fn determine_replacements(self) -> Result<Option<Vec<file::Replacer>>> {
        let mut replacers = Vec::new();

        let source_file = File::open(&self.path)?;
        let source_meta = fs::metadata(&self.path)?;
        let source_buf = unsafe { Mmap::map(&source_file)? };

        let offsets = self.determine_replacement_locations(&source_buf)?;
        let replacer = self.get_replacement(&source_buf, &offsets, source_meta.permissions())?;

        drop(source_buf);
        drop(source_file);

        replacers.push(replacer);

        Ok(Some(replacers))
    }
}
