use bitflags::bitflags;
use rand::distr::{Alphanumeric, SampleString};
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::classes::{
    ClassError,
    models::ClassRole,
    repository::{get_class_role, set_course_code},
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
    //Is the user a teacher?

    set_course_code(pool, generate_code(CODE_SIZE), class_id).await?;
    Ok(())
}
