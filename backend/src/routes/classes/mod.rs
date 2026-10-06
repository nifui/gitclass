use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod assignment;
pub mod controller;
pub mod models;
pub mod repository;
pub mod services;
pub mod student;
pub use controller::router;

#[derive(Clone, Serialize, Deserialize, Debug, Error)]
pub enum ClassError {
    #[error("placeholder")]
    Placeholder,
    #[error("Not enough permission")]
    PermissionDenied,
    #[error("Not a member of class")]
    NotMember,
}

impl From<sqlx::Error> for ClassError {
    fn from(value: sqlx::Error) -> Self {
        map_db_error(value)
    }
}

pub fn map_db_error(err: sqlx::Error) -> ClassError {
    ClassError::Placeholder
}
impl IntoResponse for ClassError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, "").into_response()
    }
}
