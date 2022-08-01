use std::path::PathBuf;

pub struct FileReplacer {
    pub path: PathBuf,
    pub temp_file: tempfile::NamedTempFile,
}
