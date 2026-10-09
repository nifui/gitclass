pub mod controller;
pub mod models;
pub mod repository;
pub mod services;
pub use controller::router;
pub use models::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum RubricError {
    FailedToCreate,
    Database,
}
impl From<sqlx::Error> for RubricError {
    fn from(_: sqlx::Error) -> Self {
        Self::Database
    }
}
