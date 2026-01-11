//! Error handling for Gosh-Fetch

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("download engine error: {0}")]
    Engine(String),

    #[error("engine not initialized")]
    EngineNotInitialized,

    #[error("database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("channel error: {0}")]
    Channel(String),
}

impl From<gosh_dl::EngineError> for Error {
    fn from(err: gosh_dl::EngineError) -> Self {
        match err {
            gosh_dl::EngineError::NotFound(msg) => Error::NotFound(msg),
            gosh_dl::EngineError::InvalidInput { field, message } => {
                Error::InvalidInput(format!("{}: {}", field, message))
            }
            gosh_dl::EngineError::Network { message, .. } => Error::Network(message),
            gosh_dl::EngineError::Storage { message, .. } => Error::Database(message),
            other => Error::Engine(other.to_string()),
        }
    }
}

impl<T> From<async_channel::SendError<T>> for Error {
    fn from(err: async_channel::SendError<T>) -> Self {
        Error::Channel(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
