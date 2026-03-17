#[derive(Debug, thiserror::Error)]
pub enum LatuiError {
    #[error("Database error: {0}")]
    Database(#[from] crate::tracking::database::DatabaseError),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("Search error: {0}")]
    Search(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Application error: {0}")]
    App(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("IO error during cache execution: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}
