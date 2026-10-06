use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Rubric {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub max_points: i32,
    pub order_index: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct RubricGrade {
    pub rubric_id: Uuid,
    pub student_id: Uuid,
    pub points: i32,
    pub feedback: Option<String>,
    pub graded_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// Useful when displaying a student's grade alongside rubric details.
#[derive(Debug, Clone, FromRow)]
pub struct RubricGradeDetails {
    pub rubric_id: Uuid,
    pub assignment_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub max_points: i32,
    pub order_index: i32,
    pub student_id: Uuid,
    pub points: i32,
    pub feedback: Option<String>,
    pub graded_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// ============================================================
// Input DTOs
// ============================================================

#[derive(Debug, Clone)]
pub struct CreateRubric {
    pub assignment_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub max_points: i32,
    pub order_index: i32,
}

#[derive(Debug, Clone)]
pub struct UpdateRubric {
    pub name: String,
    pub description: Option<String>,
    pub max_points: i32,
    pub order_index: i32,
}

#[derive(Debug, Clone)]
pub struct UpsertRubricGrade {
    pub rubric_id: Uuid,
    pub student_id: Uuid,
    pub points: i32,
    pub feedback: Option<String>,
    pub graded_by: Option<Uuid>,
}
