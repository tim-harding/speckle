#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error("unexpected column value: {0}")]
    UnexpectedValue(String),
    #[error("row not found")]
    NotFound,
}
