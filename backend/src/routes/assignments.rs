use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssignmentRequest {
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub assigned_at: Option<OffsetDateTime>,
    pub due_at: Option<OffsetDateTime>,
    pub total_points: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Assignment {
    pub id: Uuid,
    pub class_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub assigned_at: Option<OffsetDateTime>,
    pub due_at: Option<OffsetDateTime>,
    pub total_points: i64,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

use crate::{
    AppState,
    routes::auth::{AdminUser, AuthUser},
};

enum Triggers {}

pub struct Workflow {
    triggers: Triggers,
}

pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    AdminUser(claims): AdminUser,
    Json(req): Json<CreateAssignmentRequest>,
) {
}
pub async fn destroy_assignment() {}
pub async fn create_workflow() {}
