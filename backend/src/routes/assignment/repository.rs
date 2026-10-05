use crate::routes::assignment::{AssignmentError, models::*};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

/// Create a new assignment
pub async fn create_assignment<'e, E>(
    executor: E,
    payload: CreateAssignmentPayload,
) -> Result<Assignment, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
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
    .fetch_one(executor)
    .await
}
//This should definitely inspect PgQueryResult
pub async fn delete_assignment<'e, E>(executor: E, assignment_id: Uuid) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query!(
        r#"
        DELETE FROM assignments 
        WHERE id = $1
        "#,
        assignment_id
    )
    .execute(executor)
    .await?;

    Ok(result.rows_affected() > 0)
}
/// Fetch a single assignment by ID
pub async fn get_assignment<'e, E>(
    executor: E,
    assignment_id: Uuid,
) -> Result<Option<Assignment>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
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
    .fetch_optional(executor)
    .await
}

/// Assign a student to an assignment
pub async fn assign_student<'e, E>(
    executor: E,
    assignment_id: Uuid,
    student_id: Uuid,
) -> Result<AssignmentStudent, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
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
    .fetch_one(executor)
    .await
}

/// Update a student's assignment status (e.g. from ASSIGNED to SUBMITTED)
pub async fn update_student_status<'e, E>(
    executor: E,
    assignment_id: Uuid,
    student_id: Uuid,
    new_status: AssignmentStudentStatus,
) -> Result<AssignmentStudent, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
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
    .fetch_one(executor)
    .await
}
/// Link a repository to an assignment (and optionally to a student)
pub async fn link_repository<'e, E>(
    executor: E,
    assignment_id: Uuid,
    repository_id: Uuid,
    student_id: Option<Uuid>,
) -> Result<AssignmentRepositoryLink, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
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
    .fetch_one(executor)
    .await
}

/// Fetch all students and their statuses for a specific assignment
pub async fn get_assignment_students<'e, E>(
    executor: E,
    assignment_id: Uuid,
) -> Result<Vec<AssignmentStudent>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
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
    .fetch_all(executor)
    .await
}

//No duplicate assignments can exist in a class so we query with class_id to act as a unique id.
pub async fn get_assignment_id<'e, E>(
    executor: E,
    assignment_name: &str,
    class_id: Uuid,
) -> Result<Uuid, AssignmentError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query!(
        r#"
        SELECT id
        FROM assignments
        WHERE class_id = $1
        AND title = $2
        "#,
        class_id,
        assignment_name,
    )
    .fetch_optional(executor)
    .await?
    .ok_or(AssignmentError::DoesNotExist)?
    .id)
}
pub async fn assign_to_class<'e, E>(
    executor: E,
    assignment_name: &str,
    class_id: Uuid,
) -> Result<Uuid, AssignmentError>
where
    E: Executor<'e, Database = Postgres>,
{
}
