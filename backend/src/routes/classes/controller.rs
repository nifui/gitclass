use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    AppState,
    middleware::auth::AuthUser,
    routes::classes::{ClassError, models::ClassMember},
};

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
        .merge(super::assignment::router())
        .merge(super::student::router())
        .routes(routes!(get_member_info))
        .routes(routes!(get_members))
        .routes(routes!(regenerate_class_code))
        .routes(routes!(leave_class))
        .routes(routes!(join_class))
        .routes(routes!(create_class))
        .routes(routes!(set_class_code))
        .routes(routes!(link_organization))
}
use utoipa::path;

#[utoipa::path(
    get,
    path = "/class/{class_id}/members",
    params(
        ("class_id" = u32, Path, description = "ID of the class")
    ),
    responses(
        (status = 200, description = "Successfully returned class members", body = Vec<ClassMember>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class not found")
    ),
    tag = "Classes"
)]
pub async fn get_members(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(class_id): Path<u32>,
) -> Result<Json<Vec<ClassMember>>, ClassError> {
    Ok(Json(Vec::default()))
}

#[utoipa::path(
    get,
    path = "/class/{class_id}/student/{student_id}",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class"),
        ("student_id" = Uuid, Path, description = "ID of the student")
    ),
    responses(
        (status = 200, description = "Successfully returned member information"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class or member not found")
    ),
    tag = "Classes"
)]
pub async fn get_member_info(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path((class_id, student_id)): Path<(Uuid, Uuid)>,
) -> Result<(), ClassError> {
    todo!()
}

#[utoipa::path(
    post,
    path = "/class/{class_id}/regenerate-code",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class")
    ),
    responses(
        (status = 200, description = "Successfully regenerated the class code", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class not found")
    ),
    tag = "Classes"
)]
pub async fn regenerate_class_code(
    State(state): State<Arc<AppState>>,
) -> Result<String, ClassError> {
    Ok("".to_string())
}

#[utoipa::path(
    post,
    path = "/class/leave",
    responses(
        (status = 200, description = "Successfully left the class"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class membership not found")
    ),
    tag = "Classes"
)]
pub async fn leave_class(State(state): State<Arc<AppState>>, AuthUser(claims): AuthUser) {}

#[utoipa::path(
    post,
    path = "/class/join",
    responses(
        (status = 200, description = "Successfully joined the class"),
        (status = 400, description = "Invalid class code"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class not found")
    ),
    tag = "Classes"
)]
pub async fn join_class() {}

#[utoipa::path(
    post,
    path = "/class",
    responses(
        (status = 201, description = "Successfully created the class"),
        (status = 400, description = "Invalid class data"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Classes"
)]
pub async fn create_class() {}

#[utoipa::path(
    put,
    path = "/class/{class_id}/code",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class")
    ),
    responses(
        (status = 200, description = "Successfully updated the class code"),
        (status = 400, description = "Invalid class code"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class not found")
    ),
    tag = "Classes"
)]
pub async fn set_class_code() {}

#[utoipa::path(
    post,
    path = "/class/{class_id}/organization",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class")
    ),
    responses(
        (status = 200, description = "Successfully linked the organization to the class"),
        (status = 400, description = "Invalid organization data"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class or organization not found")
    ),
    tag = "Classes"
)]
pub async fn link_organization() {}
