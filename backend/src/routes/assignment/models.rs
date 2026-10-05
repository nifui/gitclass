use serde::{Deserialize, Serialize};
use sqlx::{self, prelude::FromRow};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "assignment_student_status", rename_all = "UPPERCASE")]
pub enum AssignmentStudentStatus {
    Assigned,
    Started,
    Submitted,
    Graded,
    Exempt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssignment {
    pub class_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub due_at: OffsetDateTime,
    pub total_points: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Assignment {
    pub id: Uuid,
    pub class_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub assigned_at: Option<OffsetDateTime>,
    pub due_at: Option<OffsetDateTime>,
    pub total_points: i32,
    pub status: AssignmentStatus,
    pub created_by: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssignmentStudent {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub student_id: Uuid,
    pub status: AssignmentStudentStatus,
    pub assigned_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub submitted_at: Option<OffsetDateTime>,
    pub graded_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssignmentRepositoryLink {
    pub assignment_id: Uuid,
    pub repository_id: Uuid,
    pub student_id: Option<Uuid>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssignmentPayload {
    pub class_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub assigned_at: OffsetDateTime,
    pub due_at: OffsetDateTime,
    pub total_points: i32,
    pub created_by: Uuid,
}
#[derive(Deserialize, Debug, Clone)]
pub struct AssignAssignmentRequest {
    pub student_id: Uuid,
    pub assignment_id: Uuid,
    pub class_id: Uuid,
}
