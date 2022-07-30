#[derive(thiserror::Error, Debug)]
pub enum Error {
    FileIoError(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
