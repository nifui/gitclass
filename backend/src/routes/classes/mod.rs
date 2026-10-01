use thiserror::Error;

pub mod models;
pub mod repository;

#[derive(Debug, Error)]
pub enum ClassError {
    #[error("placeholder")]
    Placeholder,
}
