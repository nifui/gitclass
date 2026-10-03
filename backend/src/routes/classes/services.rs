use bitflags::bitflags;
use rand::distr::{Alphanumeric, SampleString};
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::{
    auth::{models::SystemRole, repository::get_user_role},
    classes::{
        ClassError,
        models::{ClassRole, CreateClass},
        repository::{self, get_class_role, set_course_code},
    },
    external::organizations::organization_exists,
};

const CODE_SIZE: usize = 6;
pub fn generate_code(n: usize) -> String {
    Alphanumeric.sample_string(&mut rand::rng(), n)
}

bitflags! {
    pub struct Permissions: u8 {}
}
//Replace with something more generic like has permission or something.
//helps when adding other roles with less permissions than a teacher or something.
pub async fn permissions(pool: &PgPool, user_id: Uuid, class_id: Uuid) -> Result<bool, ClassError> {
    let role = get_class_role(pool, user_id, class_id).await?;
    Ok(role == ClassRole::Teacher || role == ClassRole::Assistant)
}

pub async fn generate_class_code(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
) -> Result<(), ClassError> {
    if !permissions(pool, user_id, class_id).await? {
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
