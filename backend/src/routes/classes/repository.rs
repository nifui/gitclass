use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use crate::routes::classes::{
    ClassError,
    models::{Class, ClassMember, ClassRole, CreateClass},
};

/// Gets the classes that a user is currently a member of.
pub async fn get_associated_classes<'e, E>(
    executor: E,
    user_id: Uuid,
) -> Result<Vec<Class>, ClassError>
where
    E: Executor<'e, Database = Postgres>,
{
    let classes = sqlx::query_as!(
        Class,
        r#"
        SELECT
            c.id,
            c.name,
            c.course_code,
            c.status as "status: _",
            c.term,
            c.organization_id,
            c.created_at,
            c.updated_at,
            c.archived_at
        FROM classes AS c
        INNER JOIN class_members AS cm
            ON cm.class_id = c.id
        WHERE cm.user_id = $1
        ORDER BY c.created_at DESC
        "#,
        user_id
    )
    .fetch_all(executor)
    .await?;

    Ok(classes)
}

/// Gets the role that a user has in a class.
pub async fn get_class_role(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
) -> Result<ClassRole, ClassError> {
    let role = sqlx::query!(
        r#"
        SELECT cm.role as "class_member_role: ClassRole"
        FROM class_members AS cm
        WHERE cm.class_id = $1
          AND cm.user_id = $2
        "#,
        class_id,
        user_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| ClassError::NotMember)?
    .class_member_role;

    Ok(role)
}
/// Creates a class.
pub async fn create_class(pool: &PgPool, input: &CreateClass) -> Result<Class, ClassError> {
    let class = sqlx::query_as!(
        Class,
        r#"
        INSERT INTO classes (
            name,
            term,
            organization_id,
            status
        )
        VALUES ($1, $2, $3, 'OPEN')
        RETURNING
            id,
            name,
            course_code,
            status as "status: _",
            term,
            organization_id,
            created_at,
            updated_at,
            archived_at
        "#,
        input.name,
        input.term,
        input.organization_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(class)
}

/// Permanently deletes a class.
pub async fn delete_class(pool: &PgPool, class_id: Uuid) -> Result<(), ClassError> {
    sqlx::query!(
        r#"
        DELETE FROM classes
        WHERE id = $1
        "#,
        class_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Links an organization to a class.
///
/// Passing `None` removes the current organization link.
pub async fn link_organization(
    pool: &PgPool,
    class_id: Uuid,
    organization_id: Option<Uuid>,
) -> Result<(), ClassError> {
    sqlx::query!(
        r#"
        UPDATE classes
        SET
            organization_id = $2,
            updated_at = now()
        WHERE id = $1
        "#,
        class_id,
        organization_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
/// Adds a user to a class.
pub async fn add_member(
    pool: &PgPool,
    class_id: Uuid,
    user_id: Uuid,
    role: ClassRole,
) -> Result<(), ClassError> {
    sqlx::query!(
        r#"
        INSERT INTO class_members (
            class_id,
            user_id,
            role
        )
        VALUES ($1, $2, $3)
        "#,
        class_id,
        user_id,
        role as ClassRole,
    )
    .execute(pool)
    .await?;

    Ok(())
}
//Retrieves all members from a given class.
pub async fn get_members(pool: &PgPool, class_id: Uuid) -> Result<Vec<ClassMember>, ClassError> {
    let members = sqlx::query_as!(
        ClassMember,
        r#"
        SELECT
            user_id,
            role as "role: ClassRole",
            joined_at
        FROM class_members
        WHERE class_id = $1
        ORDER BY joined_at
        "#,
        class_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(members)
}
pub async fn set_course_code(
    pool: &PgPool,
    course_code: String,
    class_id: Uuid,
) -> Result<(), ClassError> {
    sqlx::query!(
        r#"
        UPDATE classes 
        SET course_code = $1
        WHERE id = $2
        "#,
        course_code,
        class_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(())
}

pub async fn student_count(pool: &PgPool, class_id: Uuid) -> Result<i64, ClassError> {
    Ok(sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM classes 
        WHERE id = $1
        "#,
        class_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or_default())
}
