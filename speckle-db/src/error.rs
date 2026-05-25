#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error(transparent)]
    Limbo(#[from] limbo::Error),
    #[error("unexpected column value: {0}")]
    UnexpectedValue(String),
    #[error("row not found")]
    NotFound,
}
