use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::{
    classes::{
        ClassError,
        assignment::{
            models::{AssignmentStudent, AssignmentStudentStatus},
            repository::change_assignment_status,
        },
    },
    users::{
        models::ClassMeta,
        repository::{self},
    },
};

pub async fn get_user_classes(pool: &PgPool, user_id: Uuid) -> Result<Vec<ClassMeta>, ClassError> {
    repository::get_user_classes(pool, user_id).await
}

pub async fn get_user_assignments(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AssignmentStudent>, ClassError> {
    repository::get_user_assignments(pool, user_id).await
}

pub async fn get_user_assignment(
    pool: &PgPool,
    assignment_id: Uuid,
) -> Result<AssignmentStudent, ClassError> {
    repository::get_user_assignment(pool, assignment_id).await
}

pub async fn submit_assignment(pool: &PgPool, assignment_id: Uuid) -> Result<bool, ClassError> {
    change_assignment_status(pool, assignment_id, AssignmentStudentStatus::Submitted)
        .await
        .map_err(|_| ClassError::Placeholder)
}

pub async fn unsubmit_assignment(pool: &PgPool, assignment_id: Uuid) -> Result<bool, ClassError> {
    change_assignment_status(pool, assignment_id, AssignmentStudentStatus::Started)
        .await
        .map_err(|_| ClassError::Placeholder)
}
