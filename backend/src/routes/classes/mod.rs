use thiserror::Error;

pub mod models;
pub mod repository;

#[derive(Debug, Error)]
pub enum ClassError {
    #[error("placeholder")]
    Placeholder,
}

impl From<sqlx::Error> for ClassError {
    fn from(value: sqlx::Error) -> Self {
        map_db_error(value)
    }
}

pub fn map_db_error(err: sqlx::Error) -> ClassError {
    ClassError::Placeholder
}
