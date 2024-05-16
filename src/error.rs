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
    Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error(transparent)]
    CargoLock(#[from] cargo_lock::Error),
    #[error(transparent)]
    CargoToml(#[from] cargo_toml::Error),
    #[error(transparent)]
    SemverParse(#[from] semver::Error),
    #[error(transparent)]
    GixCommit(#[from] gix::commit::Error),
    #[error(transparent)]
    GixDiscover(#[from] gix::discover::Error),
    #[error(transparent)]
    GixDecode(#[from] gix::worktree::object::decode::Error),
    #[error(transparent)]
    GixFindExisting(#[from] gix::object::find::existing::Error),
    #[error(transparent)]
    GixObjectCommit(#[from] gix::object::commit::Error),
    #[error(transparent)]
    GixObjectWrite(#[from] gix::object::write::Error),
    #[error(transparent)]
    GixRef(#[from] gix::reference::iter::Error),
    #[error(transparent)]
    GixRefInit(#[from] gix::reference::iter::init::Error),
    #[error(transparent)]
    GixRefEdit(#[from] gix::reference::edit::Error),
    #[error(transparent)]
    GixReferenceHeadId(#[from] gix::reference::head_id::Error),
    #[error(transparent)]
    GixRemoteFindExisting(#[from] gix::remote::find::existing::Error),
    #[error(transparent)]
    GixHeadCommit(#[from] gix::reference::head_commit::Error),
    #[error(transparent)]
    GixWalk(#[from] gix::revision::walk::Error),
    #[error(transparent)]
    ConventialCommitParse(#[from] conventional_commit_parser::error::ParseError),
    #[error(transparent)]
    MiniJinja(#[from] minijinja::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    StdPathStripPrefix(#[from] std::path::StripPrefixError),
    #[error(transparent)]
    StdStrUtf8Error(#[from] std::str::Utf8Error),
    #[error("invalid version: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("invalid toml: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("EDITOR environment variable not set")]
    EditorNotSet,
    #[error("invalid replacement count: {0}")]
    InvalidReplacementCount(usize),
    #[error("invalid cargo.toml: {0}")]
    InvalidCargoToml(cargo_metadata::camino::Utf8PathBuf),
    #[error("unable to determine project base directory")]
    ProjectBaseDirectory,
    #[error("unable to determine the most recent tag")]
    TagError,
    #[error("version '{0}' was not found")]
    VersionNotFound(semver::Version),
    #[error("changelog does not contain marker character")]
    ChangelogMarker,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
