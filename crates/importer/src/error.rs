use thiserror::Error;

pub type Result<T> = std::result::Result<T, ImporterError>;

#[derive(Error, Debug)]
pub enum ImporterError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Storage error: {0}")]
    StorageError(#[from] storage::error::StorageError),

    #[error("Data transformation error: {0}")]
    TransformationError(String),

    #[error("Import error: {0}")]
    ImportError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}
