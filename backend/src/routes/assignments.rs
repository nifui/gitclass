use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    AppState,
    routes::auth::{AdminUser, AuthUser},
};

enum Triggers {}
pub struct Workflow {
    triggers: Triggers,
}

#[derive(Serialize, Deserialize)]
pub struct AssignmentRequest {
    title: String,
    due_date: OffsetDateTime,
    total_points: f32,
}
pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    AdminUser(claims): AdminUser,
    Json(req): Json<AssignmentRequest>,
) {
}
pub async fn destroy_assignment() {}
pub async fn create_workflow() {}
