use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::{
    classes::{
        ClassError,
        assignment::models::{AssignmentStudent, AssignmentStudentStatus},
        models::Permissions,
        services::get_permissions,
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
    assignment_id: Uuid,
) -> Result<Vec<AssignmentStudent>, ClassError> {
    repository::get_user_assignments(pool, assignment_id).await
}

pub async fn get_user_assignment(
    pool: &PgPool,
    assignment_id: Uuid,
) -> Result<AssignmentStudent, ClassError> {
    repository::get_user_assignment(pool, assignment_id).await
}

pub async fn submit_assignment(pool: &PgPool, assignment_id: Uuid) -> Result<bool, ClassError> {
    repository::change_assignment_status(pool, assignment_id, AssignmentStudentStatus::Submitted)
        .await
}

pub async fn unsubmit_assignment(pool: &PgPool, assignment_id: Uuid) -> Result<bool, ClassError> {
    repository::change_assignment_status(pool, assignment_id, AssignmentStudentStatus::Started)
        .await
}
//General purpsose, requires a higher set of permissions than the above two.
//This is really suboptmal as it requires 3 queries to actually work.
//Suppose it's worth it as failure cases are more clear as to where it occured.
pub async fn set_assignment_status(
    pool: &PgPool,
    user_id: Uuid,
    st_assignment_id: Uuid,
    status: AssignmentStudentStatus,
) -> Result<bool, ClassError> {
    let class_id = sqlx::query!(
        r#"
        SELECT class_id 
        FROM assignments as a 
        INNER JOIN assignment_students as st 
        ON a.id = st.assignment_id 
        WHERE st.assignment_id = $1 
        "#,
        st_assignment_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ClassError::InvalidAssignment)?
    .class_id;
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::MANAGE_CLASS)
    {
        return Err(ClassError::Unauthorized);
    }
    repository::change_assignment_status(pool, st_assignment_id, status).await
}
