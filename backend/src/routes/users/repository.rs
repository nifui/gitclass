use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres, prelude::FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::routes::{
    classes::{
        ClassError,
        assignment::models::{AssignmentStudent, AssignmentStudentStatus},
        models::ClassStatus,
    },
    users::models::ClassMeta,
};

pub async fn get_user_classes<'e, E>(
    executor: E,
    user_id: Uuid,
) -> Result<Vec<ClassMeta>, ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query_as!(
        ClassMeta,
        r#"
        SELECT 
            c.id, 
            c.name, 
            c.status as "status: ClassStatus", 
            c.term
        FROM classes as c
        INNER JOIN class_members as cm 
            ON cm.class_id = c.id 
        WHERE cm.user_id = $1
        "#,
        user_id
    )
    .fetch_all(executor)
    .await?)
}
//Unsure of the scope. Maybe just general
pub async fn get_grades<'e, E>(executor: E, user_id: Uuid) -> Result<Vec<i64>, ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(Vec::default())
}
pub async fn create_class<'e, E>(
    executor: E,
    name: String,
    term: String,
) -> Result<Uuid, ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query!(
        r#"
        INSERT INTO classes
        (name, status ,term)
        VALUES ($1, 'OPEN' , $2)
        RETURNING id
        "#,
        name,
        term
    )
    .fetch_one(executor)
    .await?
    .id)
}

pub async fn get_user_assignment<'e, E>(
    executor: E,
    assignment_id: Uuid,
) -> Result<AssignmentStudent, ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query_as!(
        AssignmentStudent,
        r#"
        SELECT 
            id, 
            assignment_id, 
            status as "status: AssignmentStudentStatus", 
            assigned_at, 
            started_at, 
            submitted_at, 
            graded_at, 
            created_at, 
            updated_at
        FROM assignment_students 
        WHERE assignment_id = $1
        "#,
        assignment_id,
    )
    .fetch_one(executor)
    .await?)
}
pub async fn get_user_assignments<'e, E>(
    executor: E,
    user_id: Uuid,
) -> Result<Vec<AssignmentStudent>, ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query_as!(
        AssignmentStudent,
        r#"
        SELECT 
            id, 
            assignment_id, 
            status as "status: AssignmentStudentStatus", 
            assigned_at, 
            started_at, 
            submitted_at, 
            graded_at, 
            created_at, 
            updated_at
        FROM assignment_students 
        WHERE student_id = $1
        "#,
        user_id
    )
    .fetch_all(executor)
    .await?)
}
