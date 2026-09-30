use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "assignment_status", rename_all = "UPPERCASE")]
pub enum AssignmentStatus {
    Draft,
    Published,
    Closed,
    Archived,
}

#[derive(Debug, Clone, Deserialize)]
pub enum AssignmentError {
    #[error("Failed to create assignment")]
    Creation,
    #[error("Failed to query database")]
    Database,
}
impl IntoResponse for AssignmentError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json("a")).into_response()
    }
}
impl From<sqlx::Error> for AssignmentError {
    fn from(_: sqlx::Error) -> Self {
        Self::AssignmentCreation
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssignmentPayload {
    pub class_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub assigned_at: Option<OffsetDateTime>,
    pub due_at: Option<OffsetDateTime>,
    pub total_points: i32,
    pub created_by: Uuid,
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
    pub status: AssignmentStatus,
    pub created_by: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

use crate::{
    AppState,
    middleware::auth::{AdminUser, AuthUser},
};

enum Triggers {}

pub struct Workflow {
    triggers: Triggers,
}

pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    AdminUser(claims): AdminUser,
    Json(req): Json<CreateAssignmentPayload>,
) -> Result<(), AssignmentError> {
    sqlx::query_as!(
        Assignment,
        r#"
        INSERT INTO assignments (
            class_id, title, description, instructions, 
            assigned_at, due_at, total_points, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING 
            id, class_id, title, description, instructions, 
            assigned_at, due_at, total_points, 
            status AS "status: AssignmentStatus", 
            created_by, created_at, updated_at
        "#,
        req.class_id,
        req.title,
        req.description,
        req.instructions,
        OffsetDateTime::now_utc(),
        req.due_at,
        req.total_points,
        claims.sub
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(())
}
pub struct AssociateRequest {
    repository_link: String,
    assignment_title: String,
}
//Links an external repository to an assignment.
// This is for turning in assignments, and is automatically done when the user starts an assignment
pub async fn associate_repository(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<AssociateRequest>,
) -> Result<(), AssignmentError> {
    //Merge these queries via a join.
    let repository_id = sqlx::query!(
        r#"
        SELECT id
        FROM repositories
        WHERE web_url = $1
        "#,
        req.repository_link,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(AssignmentError::Database)?
    .ok_or(AssignmentError::Database)?
    .id;
    let assignment_id = sqlx::query!(
        r#"
        SELECT id
        FROM assignments
        WHERE title = $1
        "#,
        req.assignment_title,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(AssignmentError::Database)?
    .ok_or(AssignmentError::Database)?
    .id;

    sqlx::query!(
        r#"
        INSERT INTO assignment_repositories ( 
            assignment_id, 
            repository_id, 
            student_id
        )
        VALUES ($1, $2, $3)
        "#,
        assignment_id,
        repository_id,
        claims.sub
    )
    .execute(&state.pool)
    .await
    .map_err(AssignmentError::Database)?;

    Ok(())
}
//Sets a template for an assignment.
pub fn set_template() {}
pub async fn destroy_assignment() {}
pub async fn create_workflow() {}

pub mod repository {}
