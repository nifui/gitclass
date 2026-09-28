use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

// --- Enums ---

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

// --- Models ---

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
    pub assigned_at: Option<OffsetDateTime>,
    pub due_at: Option<OffsetDateTime>,
    pub total_points: i32,
    pub created_by: Uuid,
}

#[derive(Clone)]
pub struct AssignmentRepository {
    pool: PgPool,
}

impl AssignmentRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new assignment
    pub async fn create_assignment(
        &self,
        payload: CreateAssignmentPayload,
    ) -> Result<Assignment, sqlx::Error> {
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
            payload.class_id,
            payload.title,
            payload.description,
            payload.instructions,
            payload.assigned_at,
            payload.due_at,
            payload.total_points,
            payload.created_by
        )
        .fetch_one(&self.pool)
        .await
    }

    /// Fetch a single assignment by ID
    pub async fn get_assignment(
        &self,
        assignment_id: Uuid,
    ) -> Result<Option<Assignment>, sqlx::Error> {
        sqlx::query_as!(
            Assignment,
            r#"
            SELECT 
                id, class_id, title, description, instructions, 
                assigned_at, due_at, total_points, 
                status AS "status: AssignmentStatus", 
                created_by, created_at, updated_at
            FROM assignments
            WHERE id = $1
            "#,
            assignment_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    /// Assign a student to an assignment
    pub async fn assign_student(
        &self,
        assignment_id: Uuid,
        student_id: Uuid,
    ) -> Result<AssignmentStudent, sqlx::Error> {
        sqlx::query_as!(
            AssignmentStudent,
            r#"
            INSERT INTO assignment_students (assignment_id, student_id)
            VALUES ($1, $2)
            RETURNING 
                id, assignment_id, student_id, 
                status AS "status: AssignmentStudentStatus", 
                assigned_at, started_at, submitted_at, graded_at, 
                created_at, updated_at
            "#,
            assignment_id,
            student_id
        )
        .fetch_one(&self.pool)
        .await
    }

    /// Update a student's assignment status (e.g. from ASSIGNED to SUBMITTED)
    /// Update a student's assignment status (e.g. from ASSIGNED to SUBMITTED)
    pub async fn update_student_status(
        &self,
        assignment_id: Uuid,
        student_id: Uuid,
        new_status: AssignmentStudentStatus,
    ) -> Result<AssignmentStudent, sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();
        sqlx::query_as!(
            AssignmentStudent,
            r#"
            UPDATE assignment_students
            SET 
                status = $1::assignment_student_status,
                updated_at = $2,
                started_at = COALESCE(started_at, CASE WHEN $1::text = 'STARTED' THEN $2::timestamptz ELSE NULL END),
                submitted_at = COALESCE(submitted_at, CASE WHEN $1::text = 'SUBMITTED' THEN $2::timestamptz ELSE NULL END),
                graded_at = COALESCE(graded_at, CASE WHEN $1::text = 'GRADED' THEN $2::timestamptz ELSE NULL END)
            WHERE assignment_id = $3 AND student_id = $4
            RETURNING 
                id, assignment_id, student_id, 
                status AS "status: AssignmentStudentStatus", 
                assigned_at, started_at, submitted_at, graded_at, 
                created_at, updated_at
            "#,
            new_status as AssignmentStudentStatus,
            now,
            assignment_id,
            student_id
        )
        .fetch_one(&self.pool)
        .await
    }
    /// Link a repository to an assignment (and optionally to a student)
    pub async fn link_repository(
        &self,
        assignment_id: Uuid,
        repository_id: Uuid,
        student_id: Option<Uuid>,
    ) -> Result<AssignmentRepositoryLink, sqlx::Error> {
        sqlx::query_as!(
            AssignmentRepositoryLink,
            r#"
            INSERT INTO assignment_repositories (assignment_id, repository_id, student_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (assignment_id, repository_id) 
            DO UPDATE SET student_id = EXCLUDED.student_id
            RETURNING assignment_id, repository_id, student_id, created_at
            "#,
            assignment_id,
            repository_id,
            student_id
        )
        .fetch_one(&self.pool)
        .await
    }

    /// Fetch all students and their statuses for a specific assignment
    pub async fn get_assignment_students(
        &self,
        assignment_id: Uuid,
    ) -> Result<Vec<AssignmentStudent>, sqlx::Error> {
        sqlx::query_as!(
            AssignmentStudent,
            r#"
            SELECT 
                id, assignment_id, student_id, 
                status AS "status: AssignmentStudentStatus", 
                assigned_at, started_at, submitted_at, graded_at, 
                created_at, updated_at
            FROM assignment_students
            WHERE assignment_id = $1
            ORDER BY assigned_at DESC
            "#,
            assignment_id
        )
        .fetch_all(&self.pool)
        .await
    }
}
