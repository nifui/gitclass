use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use crate::routes::classes::{
    ClassError,
    models::{Class, ClassRole},
};
//Gets the class information associated with a given user.
pub async fn get_associated_classes<'e, E>(
    executor: E,
    user_id: Uuid,
) -> Result<Vec<Class>, ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(Vec::default())
}
//Gets the class role associated with a user.
pub async fn get_class_role(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
) -> Result<ClassRole, ClassError> {
    sqlx::query!(
        r#"
    SELECT cm.role as "class_member_role: ClassRole"
    FROM class_members AS cm
    JOIN classes AS c
        ON c.id = cm.class_id
    "#
    )
    .fetch_optional(pool)
    .await;

    Ok(ClassRole::Teacher)
}
