use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppState, middleware::auth::AuthUser, routes::classes::ClassError};
pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
        .routes(routes!(get_assosciated_classes))
        .routes(routes!(get_grades))
        .routes(routes!(create_class))
        .routes(routes!(get_assignments))
        .routes(routes!(get_assignment))
        .routes(routes!(submit_assignment))
        .routes(routes!(unsubmit_assignment))
}
#[utoipa::path(
    get,
    path = "/user/{user_id}/classes",
    params(
        ("user_id" = Uuid, Path, description = "ID of the user")
    ),
    responses(
        (status = 200, description = "Successfully retrieved associated classes"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn get_assosciated_classes(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<(), ClassError> {
    Ok(())
}

#[utoipa::path(
    get,
    path = "/user/{user_id}/grades",
    params(
        ("user_id" = Uuid, Path, description = "ID of the user")
    ),
    responses(
        (status = 200, description = "Successfully retrieved user grades"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn get_grades() {}

#[utoipa::path(
    post,
    path = "/classes/{class_id}/memberships",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class to join")
    ),
    responses(
        (status = 201, description = "Successfully joined the class"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class not found")
    ),
    tag = "Users"
)]
pub async fn join_class() {}

#[utoipa::path(
    post,
    path = "/classes",
    responses(
        (status = 201, description = "Successfully created a class"),
        (status = 400, description = "Invalid class data"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Users"
)]
pub async fn create_class() {}

#[utoipa::path(
    get,
    path = "/users/{user_id}/assignments",
    responses(
        (status = 200, description = "Successfully retrieved assignments"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Users"
)]
pub async fn get_assignments() {}

#[utoipa::path(
    get,
    path = "/assignments/{assignment_id}",
    params(
        ("assignment_id" = Uuid, Path, description = "ID of the assignment")
    ),
    responses(
        (status = 200, description = "Successfully retrieved the assignment"),
        (status = 404, description = "Assignment not found")
    ),
    tag = "Users"
)]
pub async fn get_assignment() {}

#[utoipa::path(
    patch,
    path = "/user/{user_id}/assignments/{assignment_id}/submit",
    params(
        ("assignment_id" = Uuid, Path, description = "ID of the assignment to submit")
    ),
    responses(
        (status = 200, description = "Successfully submitted the assignment"),
        (status = 400, description = "Invalid submission"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Assignment not found")
    ),
    tag = "Users"
)]
pub async fn submit_assignment() {}

#[utoipa::path(
    patch,
    path = "/user/{user_id}/assignments/{assignment_id}/unsubmit",
    params(
        ("assignment_id" = Uuid, Path, description = "ID of the assignment to unsubmit")
    ),
    responses(
        (status = 200, description = "Successfully unsubmitted the assignment"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Assignment not found")
    ),
    tag = "Users"
)]
pub async fn unsubmit_assignment() {}
