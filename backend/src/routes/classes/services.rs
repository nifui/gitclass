use rand::distr::{Alphanumeric, SampleString};
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::{
    auth::{models::SystemRole, repository::get_user_role},
    classes::{
        ClassError,
        models::{ClassMember, ClassRole, CreateClass, Permissions},
        repository::{
            self, add_member, find_corresponding_class, get_class_role, is_member, set_course_code,
        },
    },
    external::organizations::organization_exists,
};
//Move this to a config file. Much more convenient than having to dig through the code to modify
//such a trivial number. Use a static OnceLock for this later.
const CODE_SIZE: usize = 6;
pub fn generate_code(n: usize) -> String {
    Alphanumeric.sample_string(&mut rand::rng(), n)
}
//This technically handles permission checking and membership checking in one function as to have a
//role you must be part of the class.
pub async fn get_permissions(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
) -> Result<Permissions, ClassError> {
    let role = get_class_role(pool, user_id, class_id).await?;
    Ok(role.permissions())
}
pub async fn generate_class_code(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
) -> Result<(), ClassError> {
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::MANAGE_CLASS)
    {
        return Err(ClassError::Placeholder);
    }
    set_course_code(pool, generate_code(CODE_SIZE), class_id).await?;
    Ok(())
}
//Denied permissions.
pub async fn create_class(
    pool: &PgPool,
    input: &CreateClass,
    user_id: Uuid,
) -> Result<(), ClassError> {
    //Temporary solution before I fix the error handling .
    if get_user_role(pool, user_id).await.unwrap() != SystemRole::Admin {
        return Err(ClassError::PermissionDenied);
    }
    if let Some(org_id) = input.organization_id
        && !organization_exists(pool, org_id).await.unwrap()
    {
        return Err(ClassError::Placeholder);
    }
    repository::create_class(pool, input).await?;
    Ok(())
    //Validate the input
}
pub async fn student_count(
    pool: &PgPool,
    class_id: Uuid,
    user_id: Uuid,
) -> Result<i64, ClassError> {
    if get_user_role(pool, user_id).await.unwrap() != SystemRole::Admin {
        return Err(ClassError::PermissionDenied);
    }
    repository::student_count(pool, class_id).await
}
pub async fn get_members(
    pool: &PgPool,
    class_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<ClassMember>, ClassError> {
    if !is_member(pool, class_id, user_id).await? {
        return Err(ClassError::NotMember);
    }
    repository::get_members(pool, class_id).await
}

pub async fn join_class(pool: &PgPool, code: String, user_id: Uuid) -> Result<bool, ClassError> {
    let class_id = find_corresponding_class(pool, code).await?;

    if class_id.is_none() {
        return Err(ClassError::InvalidClass);
    }

    add_member(pool, class_id.unwrap(), user_id, ClassRole::Student).await?;

    Ok(true)
}

pub async fn remove_member(pool: &PgPool, user_id: Uuid, class_id: Uuid) -> Result<(), ClassError> {
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::MANAGE_CLASS)
    {
        return Err(ClassError::Placeholder);
    }
    repository::remove_member(pool, user_id, class_id).await?;
    Ok(())
}
pub async fn leave_class(pool: &PgPool, user_id: Uuid, class_id: Uuid) -> Result<(), ClassError> {
    if !is_member(pool, class_id, user_id).await? {
        return Err(ClassError::NotMember);
    }
    repository::remove_member(pool, user_id, class_id).await?;
    Ok(())
}
