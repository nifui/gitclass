use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::routes::{
    classes::{ClassError, models::ClassStatus},
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
pub async fn get_grades<'e, E>(executor: E, user_id: Uuid) -> Result<Vec<i64>, ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(Vec::default())
}
pub async fn create_class<'e, E>(executor: E, user_id: Uuid) -> Result<(), ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(())
}

pub async fn get_user_assignments<'e, E>(executor: E, user_id: Uuid) -> Result<(), ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(())
}

pub async fn get_user_assignment<'e, E>(executor: E, user_id: Uuid) -> Result<(), ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(())
}

pub async fn change_assignment_status<'e, E>(executor: E, user_id: Uuid) -> Result<(), ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(())
}
