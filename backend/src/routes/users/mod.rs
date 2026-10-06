use utoipa_axum::router::OpenApiRouter;

pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;
pub fn router() -> OpenApiRouter {
    OpenApiRouter::default()
}
// Routes for basic user info, like their classes and general grade for each class.
