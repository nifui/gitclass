use axum::{Json, http::StatusCode, response::IntoResponse};

pub mod controller;
pub mod repository;
pub mod service;

pub enum AssignmentError {}
impl IntoResponse for AssignmentError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json("a")).into_response()
    }
}
