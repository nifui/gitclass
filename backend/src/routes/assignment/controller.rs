use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use crate::{
    AppState,
    middleware::auth::AuthUser,
    routes::assignment::{
        AssignmentError,
        models::{AssignAssignmentRequest, Assignment, AssignmentStudent, CreateAssignment},
        services,
    },
};

pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateAssignment>,
) -> Result<Assignment, AssignmentError> {
    services::create_assignment(&state.pool, req, claims.sub).await
}

pub async fn get_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json((class_id, assignment_id)): Json<(Uuid, Uuid)>,
) -> Result<Assignment, AssignmentError> {
    services::get_assignment(&state.pool, claims.sub, class_id, assignment_id).await
}

pub async fn delete_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json((class_id, assignment_id)): Json<(Uuid, Uuid)>,
) -> Result<bool, AssignmentError> {
    services::delete_assignment(&state.pool, claims.sub, class_id, assignment_id).await
}

pub async fn assign_to_student(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<AssignAssignmentRequest>,
) -> Result<AssignmentStudent, AssignmentError> {
    services::assign_student(
        &state.pool,
        claims.sub,
        req.class_id,
        req.assignment_id,
        req.student_id,
    )
    .await
}

pub async fn submit_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(assignment_id): Json<Uuid>,
) -> Result<bool, AssignmentError> {
    services::submit_assignment(&state.pool, assignment_id, claims.sub).await
}

pub async fn assign_to_class(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json((class_id, assignment_id)): Json<(Uuid, Uuid)>,
) -> Result<bool, AssignmentError> {
    services::assign_to_class(&state.pool, claims.sub, assignment_id, class_id).await
}

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
