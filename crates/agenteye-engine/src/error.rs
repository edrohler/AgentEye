use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Index(#[from] agenteye_index::IndexError),

    #[error("an index sync is already running")]
    AlreadySyncing,
}
