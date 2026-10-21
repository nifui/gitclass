use crate::routes::classes::assignment::{AssignmentError, models::*};
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
            title, description, instructions,
            total_points, created_by
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING 
            id, title, description, instructions, 
            total_points, 
            status AS "status: AssignmentStatus", 
            created_by, created_at, updated_at

        "#,
        payload.title,
        payload.description,
        payload.instructions,
        payload.total_points,
        payload.created_by,
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
            id, title, description, instructions, 
            total_points, 
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
            INSERT INTO assignment_students (class_assignment_id, student_id)
            VALUES ($1, $2)
            RETURNING 
                id, class_assignment_id, 
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
            started_at = COALESCE(
                started_at, 
                CASE WHEN $1::text = 'STARTED' 
                THEN $2::timestamptz 
                ELSE NULL END
            ),
            submitted_at = COALESCE(
                submitted_at, 
                CASE WHEN $1::text = 'SUBMITTED' 
                THEN $2::timestamptz 
                ELSE NULL END
            ),
            graded_at = COALESCE(
                graded_at, 
                CASE WHEN $1::text = 'GRADED' 
                THEN $2::timestamptz 
                ELSE NULL END
            )
        WHERE class_assignment_id = $3 AND student_id = $4
        RETURNING 
            id, 
            class_assignment_id, 
            status AS "status: AssignmentStudentStatus", 
            assigned_at, 
            started_at, 
            submitted_at, 
            graded_at, 
            created_at, 
            updated_at
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
        INSERT INTO assignment_repositories (class_assignment_id, repository_id, student_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (class_assignment_id, repository_id) 
        DO UPDATE SET student_id = EXCLUDED.student_id
        RETURNING class_assignment_id, repository_id, student_id, created_at
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
            id, class_assignment_id,
            status AS "status: AssignmentStudentStatus", 
            assigned_at, started_at, submitted_at, graded_at, 
            created_at, updated_at
        FROM assignment_students
        WHERE class_assignment_id = $1
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
    class_assignment_id: Uuid,
) -> Result<Uuid, AssignmentError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query!(
        r#"
        SELECT id
        FROM class_assignments
        WHERE id = $1
        "#,
        class_assignment_id,
    )
    .fetch_optional(executor)
    .await?
    .ok_or(AssignmentError::DoesNotExist)?
    .id)
}
//This hides a lot of failure cases. Some of these can be handled by the service built on top of it
//but the nature of this being a fused query obfuscates a lot of errors into one.
//Certain students not being assigned the assignment as they already have it?
//Invalid class_id?
//Invalid assignment_id?
//Will strengthen later.
//Some of the concerns might not be actual problems.

pub async fn assign_to_class<'e, E>(
    executor: &mut E,
    assignment_id: Uuid,
    class_id: Uuid,
) -> Result<bool, AssignmentError>
where
    for<'c> &'c mut E: Executor<'c, Database = Postgres>,
{
    let class_assignment_id = sqlx::query_scalar!(
        r#"
        INSERT INTO class_assignments (
            class_id,
            assignment_id
        )
        VALUES ($1, $2)
        ON CONFLICT (class_id, assignment_id)
        DO NOTHING
        RETURNING id
        "#,
        class_id,
        assignment_id,
    )
    .fetch_optional(&mut *executor)
    .await?;

    let (class_assignment_id, created) = match class_assignment_id {
        Some(id) => (id, true),
        None => {
            let id = sqlx::query_scalar!(
                r#"
                SELECT id
                FROM class_assignments
                WHERE class_id = $1
                  AND assignment_id = $2
                "#,
                class_id,
                assignment_id,
            )
            .fetch_one(&mut *executor)
            .await?;

            (id, false)
        }
    };

    sqlx::query!(
        r#"
        INSERT INTO assignment_students (
            class_assignment_id,
            student_id
        )
        SELECT
            $1,
            cm.user_id
        FROM class_members AS cm
        WHERE cm.class_id = $2
          AND cm.role = 'STUDENT'
        ON CONFLICT (class_assignment_id, student_id)
        DO NOTHING
        "#,
        class_assignment_id,
        class_id,
    )
    .execute(&mut *executor)
    .await?;

    Ok(created)
}

pub async fn change_assignment_status<'e, E>(
    executor: E,
    assignment_id: Uuid,
    status: AssignmentStudentStatus,
) -> Result<bool, AssignmentError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query!(
        r#"
        UPDATE assignment_students
        SET status = $1::assignment_student_status
        WHERE student_id = $2
        "#,
        status as AssignmentStudentStatus,
        assignment_id
    )
    .execute(executor)
    .await?
    .rows_affected()
        > 0)
}
//Teachers and the owner of the assignment can modify it.
pub async fn can_modify_assignment<'e, E>(
    executor: E,
    user_id: Uuid,
    assignment_st_id: Uuid,
) -> Result<bool, AssignmentError>
where
    E: Executor<'e, Database = Postgres>,
{
    let is_owner = sqlx::query!(
        r#"
        SELECT 
        CASE 
        WHEN student_id = $1 AND id = $2 THEN 1
        ELSE 0
        END AS is_owner
        FROM assignment_students
        "#,
        user_id,
        assignment_st_id
    )
    .fetch_one(executor)
    .await?
    .is_owner;

    Ok(false)
}
