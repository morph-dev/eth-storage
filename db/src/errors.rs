use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("General DB Error")]
    Error,
}
