use rand::distr::{Alphanumeric, SampleString};
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::{
    auth::{models::SystemRole, repository::get_user_role},
    classes::{
        ClassError,
        models::{CreateClass, Permissions},
        repository::{self, get_class_role, set_course_code},
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
) -> Result<Permissions, sqlx::Error> {
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
