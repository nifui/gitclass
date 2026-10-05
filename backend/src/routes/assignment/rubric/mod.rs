pub mod models;
pub use models::*;

use sqlx::{Executor, Postgres};
use uuid::Uuid;

/// Create a rubric for an assignment.
pub async fn create_rubric<'e, E>(executor: E, input: CreateRubric) -> Result<Rubric, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        Rubric,
        r#"
        INSERT INTO rubrics (
            assignment_id,
            name,
            description,
            max_points,
            order_index
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            assignment_id,
            name,
            description,
            max_points,
            order_index,
            created_at
        "#,
        input.assignment_id,
        input.name,
        input.description,
        input.max_points,
        input.order_index
    )
    .fetch_one(executor)
    .await
}

/// Get a rubric by ID.
pub async fn get_rubric<'e, E>(executor: E, rubric_id: Uuid) -> Result<Option<Rubric>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        Rubric,
        r#"
        SELECT
            id,
            assignment_id,
            name,
            description,
            max_points,
            order_index,
            created_at
        FROM rubrics
        WHERE id = $1
        "#,
        rubric_id
    )
    .fetch_optional(executor)
    .await
}

/// List all rubrics for an assignment in display order.
pub async fn list_rubrics_by_assignment<'e, E>(
    executor: E,
    assignment_id: Uuid,
) -> Result<Vec<Rubric>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        Rubric,
        r#"
        SELECT
            id,
            assignment_id,
            name,
            description,
            max_points,
            order_index,
            created_at
        FROM rubrics
        WHERE assignment_id = $1
        ORDER BY order_index ASC, created_at ASC
        "#,
        assignment_id
    )
    .fetch_all(executor)
    .await
}

/// Update a rubric's editable fields.
pub async fn update_rubric<'e, E>(
    executor: E,
    rubric_id: Uuid,
    input: UpdateRubric,
) -> Result<Option<Rubric>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        Rubric,
        r#"
        UPDATE rubrics
        SET
            name = $2,
            description = $3,
            max_points = $4,
            order_index = $5
        WHERE id = $1
        RETURNING
            id,
            assignment_id,
            name,
            description,
            max_points,
            order_index,
            created_at
        "#,
        rubric_id,
        input.name,
        input.description,
        input.max_points,
        input.order_index
    )
    .fetch_optional(executor)
    .await
}

/// Delete a rubric. Associated grades are deleted by CASCADE.
pub async fn delete_rubric<'e, E>(executor: E, rubric_id: Uuid) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query!(
        r#"
        DELETE FROM rubrics
        WHERE id = $1
        "#,
        rubric_id
    )
    .execute(executor)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Calculate the maximum possible score across an assignment's rubrics.
pub async fn get_assignment_max_points<'e, E>(
    executor: E,
    assignment_id: Uuid,
) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(max_points), 0)::BIGINT AS "total!"
        FROM rubrics
        WHERE assignment_id = $1
        "#,
        assignment_id
    )
    .fetch_one(executor)
    .await?;

    Ok(result.total)
}

// --------------------------------------------------------
// Rubric grades
// --------------------------------------------------------

/// Create or update a student's grade for a rubric.
pub async fn upsert_grade<'e, E>(
    executor: E,
    input: UpsertRubricGrade,
) -> Result<RubricGrade, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        RubricGrade,
        r#"
        INSERT INTO rubric_grades (
            rubric_id,
            student_id,
            points,
            feedback,
            graded_by
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (rubric_id, student_id)
        DO UPDATE SET
            points = EXCLUDED.points,
            feedback = EXCLUDED.feedback,
            graded_by = EXCLUDED.graded_by,
            updated_at = now()
        RETURNING
            rubric_id,
            student_id,
            points,
            feedback,
            graded_by,
            created_at,
            updated_at
        "#,
        input.rubric_id,
        input.student_id,
        input.points,
        input.feedback,
        input.graded_by
    )
    .fetch_one(executor)
    .await
}

/// Get one student's grade for a rubric.
pub async fn get_grade<'e, E>(
    executor: E,
    rubric_id: Uuid,
    student_id: Uuid,
) -> Result<Option<RubricGrade>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        RubricGrade,
        r#"
        SELECT
            rubric_id,
            student_id,
            points,
            feedback,
            graded_by,
            created_at,
            updated_at
        FROM rubric_grades
        WHERE rubric_id = $1
          AND student_id = $2
        "#,
        rubric_id,
        student_id
    )
    .fetch_optional(executor)
    .await
}

/// List all grades for a particular rubric.
pub async fn list_grades_by_rubric<'e, E>(
    executor: E,
    rubric_id: Uuid,
) -> Result<Vec<RubricGrade>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        RubricGrade,
        r#"
        SELECT
            rubric_id,
            student_id,
            points,
            feedback,
            graded_by,
            created_at,
            updated_at
        FROM rubric_grades
        WHERE rubric_id = $1
        ORDER BY created_at ASC
        "#,
        rubric_id
    )
    .fetch_all(executor)
    .await
}

/// List a student's rubric grades for an assignment.
pub async fn list_student_assignment_grades<'e, E>(
    executor: E,
    assignment_id: Uuid,
    student_id: Uuid,
) -> Result<Vec<RubricGradeDetails>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        RubricGradeDetails,
        r#"
        SELECT
            r.id AS rubric_id,
            r.assignment_id,
            r.name,
            r.description,
            r.max_points,
            r.order_index,
            g.student_id,
            g.points,
            g.feedback,
            g.graded_by,
            g.created_at,
            g.updated_at
        FROM rubrics r
        INNER JOIN rubric_grades g
            ON g.rubric_id = r.id
        WHERE r.assignment_id = $1
          AND g.student_id = $2
        ORDER BY r.order_index ASC, r.created_at ASC
        "#,
        assignment_id,
        student_id
    )
    .fetch_all(executor)
    .await
}

/// Calculate a student's total points for an assignment.
/// Rubrics without a grade contribute zero.
pub async fn get_student_assignment_total<'e, E>(
    executor: E,
    assignment_id: Uuid,
    student_id: Uuid,
) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(g.points), 0)::BIGINT AS "total!"
        FROM rubrics r
        LEFT JOIN rubric_grades g
            ON g.rubric_id = r.id
           AND g.student_id = $2
        WHERE r.assignment_id = $1
        "#,
        assignment_id,
        student_id
    )
    .fetch_one(executor)
    .await?;

    Ok(result.total)
}

/// Delete a student's grade for a rubric.
pub async fn delete_grade<'e, E>(
    executor: E,
    rubric_id: Uuid,
    student_id: Uuid,
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query!(
        r#"
        DELETE FROM rubric_grades
        WHERE rubric_id = $1
          AND student_id = $2
        "#,
        rubric_id,
        student_id
    )
    .execute(executor)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// List all rubric grades for every student in an assignment.
pub async fn list_assignment_grades<'e, E>(
    executor: E,
    assignment_id: Uuid,
) -> Result<Vec<RubricGradeDetails>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        RubricGradeDetails,
        r#"
        SELECT
            r.id AS rubric_id,
            r.assignment_id,
            r.name,
            r.description,
            r.max_points,
            r.order_index,
            g.student_id,
            g.points,
            g.feedback,
            g.graded_by,
            g.created_at,
            g.updated_at
        FROM rubrics r
        INNER JOIN rubric_grades g
            ON g.rubric_id = r.id
        WHERE r.assignment_id = $1
        ORDER BY
            r.order_index ASC,
            g.student_id ASC
        "#,
        assignment_id
    )
    .fetch_all(executor)
    .await
}
