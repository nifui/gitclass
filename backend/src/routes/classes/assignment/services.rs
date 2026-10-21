use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::routes::{
    auth::{models::SystemRole, repository::get_user_role},
    classes::{
        assignment::{
            AssignmentError,
            models::{
                Assignment, AssignmentRepositoryLink, AssignmentStudent, AssignmentStudentStatus,
                CreateAssignment, CreateAssignmentPayload,
            },
            repository::{
                self, change_assignment_status, get_assignment_id, get_assignment_students,
            },
        },
        models::Permissions,
        services::get_permissions,
    },
};

pub async fn create_assignment(
    pool: &PgPool,
    parameters: CreateAssignment,
    user_id: Uuid,
) -> Result<Assignment, AssignmentError> {
    let user_role = get_user_role(pool, user_id)
        .await
        .map_err(|_| AssignmentError::Database)?;
    if user_role != SystemRole::Admin {
        return Err(AssignmentError::Unauthorized);
    }

    let payload = CreateAssignmentPayload {
        class_id: parameters.class_id,
        title: parameters.title,
        description: parameters.description,
        instructions: parameters.instructions,
        assigned_at: OffsetDateTime::now_utc(),
        due_at: parameters.due_at,
        total_points: parameters.total_points,
        created_by: user_id,
    };
    repository::create_assignment(pool, payload)
        .await
        .map_err(|_| AssignmentError::Database)
}

pub async fn get_assignment(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
    assignment_id: Uuid,
) -> Result<Assignment, AssignmentError> {
    //Check if the user has access to the class in which the assignment exists.
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::VIEW_ASSIGNMENTS)
    {
        return Err(AssignmentError::Unauthorized);
    }
    repository::get_assignment(pool, assignment_id)
        .await?
        .ok_or(AssignmentError::DoesNotExist)
}

pub async fn delete_assignment(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
    assignment_id: Uuid,
) -> Result<bool, AssignmentError> {
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::DELETE_ASSIGNMENTS)
    {
        return Err(AssignmentError::Unauthorized);
    }
    Ok(repository::delete_assignment(pool, assignment_id).await?)
}
//Assign an assignment to a student.
pub async fn assign_student(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
    assignment_id: Uuid,
    student_id: Uuid,
) -> Result<AssignmentStudent, AssignmentError> {
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::CREATE_ASSIGNMENTS)
    {
        return Err(AssignmentError::Unauthorized);
    }
    Ok(repository::assign_student(pool, assignment_id, student_id).await?)
}
//Returns info on the assignments and students associated with them.
pub async fn assignment_student_info(
    pool: &PgPool,
    user_id: Uuid,
    class_id: Uuid,
    assignment_name: String,
) -> Result<Vec<AssignmentStudent>, AssignmentError> {
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::MANAGE_CLASS)
    {
        return Err(AssignmentError::Unauthorized);
    }

    let assignment_id = get_assignment_id(pool, &assignment_name, class_id).await?;
    Ok(get_assignment_students(pool, assignment_id).await?)
}

pub async fn submit_assignment(
    pool: &PgPool,
    assignment_id: Uuid,
    student_id: Uuid,
    class_id: Uuid,
) -> Result<bool, AssignmentError> {
    if !get_permissions(pool, student_id, class_id)
        .await?
        .contains(Permissions::SUBMIT_ASSIGNMENT)
    {
        return Err(AssignmentError::Unauthorized);
    }

    repository::update_student_status(
        pool,
        assignment_id,
        student_id,
        AssignmentStudentStatus::Submitted,
    )
    .await?;
    Ok(true)
}
//Assign an assignment to every student.
pub async fn assign_to_class(
    pool: &PgPool,
    user_id: Uuid,
    assignment_id: Uuid,
    class_id: Uuid,
) -> Result<bool, AssignmentError> {
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::ASSIGN_ASSIGNMENT)
    {
        return Err(AssignmentError::Unauthorized);
    }
    let mut tx = pool.begin().await?;
    let createdd = repository::assign_to_class(&mut *tx, assignment_id, class_id).await?;
    tx.commit().await?;
    Ok(createdd)
}

// This does not perform any sort of check of whether the repository matches the template set by the
// teacher. However this does get caught by the workflow/job executor.
// THIS IS NOT A GENERAL TEMPLATE ASSIGNER FOR THE TEACHER.
// THIS ONLY ASSOCIATES A REPOSITORY WITH A STUDENTS ASSIGNMENT FOR SUBMISSION OF.
// Refer to the template creation logic.
pub async fn link_repsitory(
    pool: PgPool,
    repository_id: Uuid,
    assignment_id: Uuid,
    student_id: Option<Uuid>,
) -> Result<AssignmentRepositoryLink, AssignmentError> {
    repository::link_repository(&pool, assignment_id, repository_id, student_id)
        .await
        .map_err(|_| AssignmentError::Database)
}
//General purpsose, requires a higher set of permissions than the above two.
//This is really suboptmal as it requires 3 queries to actually work.
//Suppose it's worth it as failure cases are more clear as to where it occured.
pub async fn set_assignment_status(
    pool: &PgPool,
    user_id: Uuid,
    st_assignment_id: Uuid,
    status: AssignmentStudentStatus,
) -> Result<bool, AssignmentError> {
    let class_id = sqlx::query!(
        r#"
        SELECT class_id 
        FROM class_assignments as a 
        INNER JOIN assignment_students as st 
        ON a.id = st.class_assignment_id 
        WHERE st.id = $1 
        "#,
        st_assignment_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AssignmentError::NotFound)?
    .class_id;
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::MANAGE_CLASS)
    {
        return Err(AssignmentError::Unauthorized);
    }
    change_assignment_status(pool, st_assignment_id, status).await
}
pub async fn update_assignment() {}
