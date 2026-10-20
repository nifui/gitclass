use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::{
    classes::ClassError,
    users::{models::ClassMeta, repository},
};

pub async fn get_user_classes(pool: &PgPool, user_id: Uuid) -> Result<Vec<ClassMeta>, ClassError> {
    repository::get_user_classes(pool, user_id).await
}
