use thiserror::Error;

pub type Result<T> = std::result::Result<T, IndexError>;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("index database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Source(#[from] agenteye_core::Error),
}
