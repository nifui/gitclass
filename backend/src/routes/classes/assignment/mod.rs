use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{errors::ApiError, routes::classes::ClassError};
pub mod controller;
pub mod models;
pub mod repository;
pub mod rubric;
pub mod services;
pub use controller::router;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum AssignmentError {
    #[error("Failed to create assignment")]
    Creation,

    #[error("Database error occurred")]
    Database,

    #[error("Assignment does not exist")]
    DoesNotExist,

    #[error("Not enough permissions")]
    Unauthorized,

    #[error("Resource was not found with the provided parameters")]
    NotFound,

    #[error(transparent)]
    ClassError(#[from] ClassError),
}
impl IntoResponse for AssignmentError {
    fn into_response(self) -> axum::response::Response {
        // Map each error variant to an appropriate HTTP status code
        let status = match &self {
            Self::Creation => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DoesNotExist | Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::FORBIDDEN,
            Self::ClassError(_) => StatusCode::BAD_REQUEST, // Adjust depending on ClassError' nature
        };

        // Create a clean JSON body containing the error message
        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status, body).into_response()
    }
}
//Hide internal errors from users of the API.
impl From<AssignmentError> for ApiError {
    fn from(_: AssignmentError) -> Self {
        Self::Internal
    }
}
impl From<sqlx::Error> for AssignmentError {
    fn from(_: sqlx::Error) -> Self {
        Self::Database
    }
}
