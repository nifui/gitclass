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

pub async fn assign_assignment(
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
    Json((class_id, assignment_id)): Json<(Uuid, Uuid)>,
) -> Result<bool, AssignmentError> {
    services::submit_assignment(&state.pool, class_id, assignment_id, claims.sub).await
}

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
