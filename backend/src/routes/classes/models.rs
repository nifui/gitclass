use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Eq, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "class_member_role", rename_all = "UPPERCASE")]
pub enum ClassRole {
    Teacher,
    Assistant,
    Student,
}
impl IntoResponse for ClassRole {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "class_status", rename_all = "UPPERCASE")]
pub enum ClassStatus {
    Archived,
    Open,
}

///Basic metadata for a given class.
#[derive(Deserialize, Serialize, FromRow, ToSchema)]
pub struct Class {
    pub id: Uuid,
    pub name: String,
    pub course_code: Option<String>,
    pub status: ClassStatus,
    pub term: Option<String>,
    pub organization_id: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub archived_at: Option<OffsetDateTime>,
}

/// Information detailing a member of a given class.
#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct ClassMember {
    pub role: ClassRole,
    pub user_id: Uuid,
    pub joined_at: OffsetDateTime,
}

// Request to create a class.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClass {
    pub name: String,
    pub term: Option<String>,
    pub organization_id: Option<Uuid>,
}
