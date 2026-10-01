use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::routes::{
    assignment::{
        AssignmentError,
        models::{
            Assignment, AssignmentRepositoryLink, AssignmentStudent, CreateAssignment,
            CreateAssignmentPayload,
        },
        repository::{self, get_assignment_students, get_assingment_id},
    },
    auth::{models::SystemRole, repository::get_user_role},
};
//Convert to a payload.
pub async fn create_assignment(
    pool: &PgPool,
    parameters: CreateAssignment,
    user_id: Uuid,
) -> Result<Assignment, AssignmentError> {
    //Check if the operation can be performed.
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
pub async fn delete_assignment() {}

//Returns info on the assignments and students associated with them.
pub async fn assignment_student_info(
    pool: PgPool,
    class_id: Uuid,
    assignment_name: String,
) -> Result<Vec<AssignmentStudent>, AssignmentError> {
    let assignment_id = get_assingment_id(&pool, &assignment_name, class_id).await?;
    Ok(get_assignment_students(&pool, assignment_id).await?)
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
pub struct ClassFilter {}
//Same idea as below but a diff scope.
pub async fn assign_to_class() {}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Range<T> {
    pub upper: T,
    pub lower: T,
}
//Rust has this somewhat but not as an enum.
#[derive(Eq, Debug, Serialize, Deserialize, PartialEq)]
pub enum Condition<T>
where
    T: Eq + PartialEq,
{
    Range(Range<T>),
    EqualTo(T),
    LessThan(T),
    GreaterThan(T),
}
//The Eq restricts the types to disclude floating types.
pub struct StudentFilter<T>
where
    T: Eq + PartialEq,
{
    pub grade_range: Condition<T>,
}

//Add a filter option via queries to allow teacher to set students who should recieve the
//assignment.
pub async fn assign_to_student() {}
