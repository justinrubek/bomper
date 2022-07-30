use memmap::{Mmap, MmapMut};
use std::{fs, fs::File, io::prelude::*, path::Path, ops::DerefMut};

use crate::error::{Error, Result};

pub fn overwrite_file(path: &Path, old_content: &str, new_content: &str) -> Result<()> {
    let source_file = File::open(path)?;
    let source_meta = fs::metadata(path)?;
    let source_map  = unsafe { Mmap::map(&source_file)? };

    let data = String::from(new_content).into_bytes();

    // Replace occurences of old_content with new_content in source_map.
    let search_text = regex::escape(old_content);
    let regex = regex::bytes::RegexBuilder::new(&search_text).build()?;
    let replaced = replace(&regex, &source_map, data);

    let temp_file = tempfile::NamedTempFile::new_in(
        path.parent()
        .ok_or_else(|| Error::InvalidPath(path.to_path_buf()))?,
    )?;
    let file = temp_file.as_file();
    file.set_len(replaced.len() as u64)?;
    file.set_permissions(source_meta.permissions())?;

    if replaced.is_empty() == false {
        let mut target_map = unsafe { MmapMut::map_mut(&file)? };
        target_map.deref_mut().write_all(&replaced)?;
        target_map.flush_async()?;
    }

    drop(source_map);
    drop(source_file);

    temp_file.persist(fs::canonicalize(path)?)?;

    Ok(())
}

fn replace<'a>(
    regex: &regex::bytes::Regex,
    buf: &'a [u8],
    replace_with: Vec<u8>,
) -> std::borrow::Cow<'a, [u8]> {
    regex.replacen(&buf, 0, &*replace_with)
}
