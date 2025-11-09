use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

impl StorageError {
    pub fn is_unique_violation(&self) -> bool {
        matches!(
            self,
            StorageError::Database(sqlx::Error::Database(e))
                if e.code().as_deref() == Some("23505")
        )
    }

    pub fn is_foreign_key_violation(&self) -> bool {
        matches!(
            self,
            StorageError::Database(sqlx::Error::Database(e))
                if e.code().as_deref() == Some("23503")
        )
    }
}
