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
    routes::classes::assignment::{
        AssignmentError,
        models::{AssignAssignmentRequest, Assignment, AssignmentStudent, CreateAssignment},
        services,
    },
};
#[utoipa::path(get, path = "/classes/{class_id}/assignments/create")]
pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateAssignment>,
) -> Json<Result<Assignment, AssignmentError>> {
    Json(services::create_assignment(&state.pool, req, claims.sub).await)
}

#[utoipa::path(get, path = "/classes/{class_id}/assignments")]
pub async fn list_assignments() {}

//This should not rely on user id's to fetch a resource as it complicates the authentication model.
#[utoipa::path(get, path = "/classes/{class_id}/assignments/{assignment_id}")]
pub async fn get_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json((class_id, assignment_id)): Json<(Uuid, Uuid)>,
) -> Json<Result<Assignment, AssignmentError>> {
    Json(services::get_assignment(&state.pool, claims.sub, class_id, assignment_id).await)
}

#[utoipa::path(delete, path = "/classes/{class_id}/assignments/{assignment_id}")]
pub async fn delete_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path((class_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> Json<Result<bool, AssignmentError>> {
    Json(services::delete_assignment(&state.pool, claims.sub, class_id, assignment_id).await)
}
#[utoipa::path(post, path = "/assign")]
pub async fn assign_to_student(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<AssignAssignmentRequest>,
) -> Json<Result<AssignmentStudent, AssignmentError>> {
    Json(
        services::assign_student(
            &state.pool,
            claims.sub,
            req.class_id,
            req.assignment_id,
            req.student_id,
        )
        .await,
    )
}
#[utoipa::path(post, path = "/classes/{class_id}/assignments/{assignment_id}/submit")]
pub async fn submit_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path((class_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> Json<Result<bool, AssignmentError>> {
    Json(services::submit_assignment(&state.pool, assignment_id, claims.sub, class_id).await)
}
#[utoipa::path(post, path = "/assign_class")]
pub async fn assign_to_class(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json((class_id, assignment_id)): Json<(Uuid, Uuid)>,
) -> Json<Result<bool, AssignmentError>> {
    Json(services::assign_to_class(&state.pool, claims.sub, assignment_id, class_id).await)
}
//This should queue some job if it requires some workflow to be executed.
//By default on submission, it'll query if anytthing relevant to grading can be performed.
//If yes queue the job to be executed, otherwise leave as is.
//

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
        .routes(routes!(create_assignment))
        .routes(routes!(get_assignment))
        .routes(routes!(delete_assignment))
        .routes(routes!(assign_to_student))
        .routes(routes!(submit_assignment))
        .routes(routes!(assign_to_class))
}
