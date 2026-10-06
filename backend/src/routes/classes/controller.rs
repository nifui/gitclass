use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    routing::get,
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
        .route("/class/{class_id}/students", get(get_members))
        .merge(super::assignment::router())
        .merge(super::student::router())
        .routes(routes!(get_member_info))
}
pub async fn get_members(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(class_id): Path<u32>,
) -> Result<Json<Vec<ClassMember>>, ClassError> {
    Ok(Json(Vec::default()))
}

#[utoipa::path(get, path = "/class/{class_id}/student/{student_id}", 
    responses(
        (status = 200, description = "Successfully returned member info"),
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

pub async fn regenerate_class_code(
    State(state): State<Arc<AppState>>,
) -> Result<String, ClassError> {
    Ok("".to_string())
}
