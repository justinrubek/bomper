#[derive(thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FileIo(#[from] std::io::Error),
    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("file to replace has no parent: {0}")]
    InvalidPath(std::path::PathBuf),
    #[error("failed to replace file with tempfile: {0}")]
    TempFilePersist(#[from] tempfile::PersistError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
