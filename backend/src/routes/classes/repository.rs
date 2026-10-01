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
    let role = sqlx::query_as!(
        ClassRole,
        r#"
        SELECT cm.role as "class_member_role: ClassRole"
        FROM class_members AS cm
        JOIN classes AS c
            ON c.id = cm.class_id
        WHERE c.id = $1
        AND cm.user_id = $2
        "#,
        user_id,
        class_id
    )
    .fetch_one(pool)
    .await?;

    Ok(ClassRole::Teacher)
}
//Creates a class.
pub async fn create_class(pool: &PgPool) -> Result<Class, ClassError> {
    todo!()
}
//Archives a given class.
pub async fn archive_clsas(pool: &PgPool, class_id: Uuid) -> Result<(), ClassError> {
    sqlx::query!(
        r#"
        UPDATE classes
        SET status = 'ARCHIVED' 
        WHERE id = $1
        "#,
        class_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
//Deletes a given class.
pub async fn delete_class() {}
//Links an organization to the given class. There can only be one link at a time.
//Default behavior is to overwrite the prior organization link.
// Because of this orphaned rows can exist.
pub async fn link_organization() {}

//Gets a list of members(Uuid)
pub async fn get_members() {}

//Given a class code it'll attempt to insert it into the given class.
//If a code exists it will overwrite said prior code and indicate as such.
pub async fn insert_class_code() {}
