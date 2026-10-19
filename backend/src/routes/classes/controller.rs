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
    routes::classes::{
        ClassError,
        models::{ClassMember, CreateClass, JoinRequest},
        services,
    },
};

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
        .merge(super::assignment::router())
        .merge(super::student::router())
        .routes(routes!(get_member_info))
        .routes(routes!(get_members))
        .routes(routes!(leave_class))
        .routes(routes!(join_class))
        .routes(routes!(create_class))
        .routes(routes!(link_organization))
        .routes(routes!(generate_class_code))
}

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
    path = "/class/{class_id}/leave",
    responses(
        (status = 200, description = "Successfully left the class"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class membership not found")
    ),
    tag = "Classes"
)]
pub async fn leave_class(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(class_id): Path<Uuid>,
) -> Result<(), ClassError> {
    services::leave_class(&state.pool, claims.sub, class_id).await
}

#[utoipa::path(
    post,
    path = "/class/join",
    responses(
        (status = 200, description = "Successfully joined the class", body = bool),
        (status = 400, description = "Invalid class code"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Class not found")
    ),
    tag = "Classes"
)]
pub async fn join_class(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<JoinRequest>,
) -> Result<Json<bool>, ClassError> {
    Ok(Json(
        services::join_class(&state.pool, req.code, claims.sub).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/class/create",
    responses(
        (status = 201, description = "Successfully created the class"),
        (status = 400, description = "Invalid class data"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Classes"
)]
pub async fn create_class(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateClass>,
) -> Result<(), ClassError> {
    Ok(())
}

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
pub async fn generate_class_code(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(class_id): Path<Uuid>,
) -> Result<(), ClassError> {
    services::generate_class_code(&state.pool, claims.sub, class_id).await
}

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
pub async fn link_organization(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<(), ClassError> {
    Ok(())
}
