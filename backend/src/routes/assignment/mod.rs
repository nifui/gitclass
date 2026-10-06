use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use thiserror::Error;

use crate::{errors::ApiError, routes::classes::ClassError};
pub mod controller;
pub mod models;
pub mod repository;
pub mod rubric;
pub mod services;
pub use controller::router;
#[derive(Debug, Clone, Deserialize, Error)]
pub enum AssignmentError {
    #[error("Failed to create assignment")]
    Creation,
    #[error("Failed to query database")]
    Database,
    #[error("Assignment does not exist")]
    DoesNotExist,
    #[error("Not enough permission")]
    Unauthorized,
    #[error("Resource was not found with the provided parameters")]
    NotFound,
    #[error("Transparent")]
    ClassError(#[from] ClassError),
}

impl From<sqlx::Error> for AssignmentError {
    fn from(_: sqlx::Error) -> Self {
        Self::Database
    }
}

impl IntoResponse for AssignmentError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json("a")).into_response()
    }
}

impl From<AssignmentError> for ApiError {
    fn from(_: AssignmentError) -> Self {
        Self::Internal
    }
}
