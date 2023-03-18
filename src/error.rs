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
    RonDeserialize(#[from] ron::de::SpannedError),
    #[error(transparent)]
    CargoLock(#[from] cargo_lock::Error),
    #[error(transparent)]
    SemverParse(#[from] semver::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error("invalid replacement count: {0}")]
    InvalidReplacementCount(usize),
    
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
