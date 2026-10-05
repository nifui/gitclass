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
        repository::{self, get_assignment_id, get_assignment_students},
    },
    auth::{
        models::SystemRole,
        repository::{self, get_user_role},
    },
    classes::{
        models::Permissions,
        services::{get_permissions, permissions},
    },
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
) -> Result<(), AssignmentError> {
    if !get_permissions(pool, user_id, class_id)
        .await?
        .contains(Permissions::DELETE_ASSIGNMENTS)
    {
        return Err(AssignmentError::Unauthorized);
    }
    Ok(repository::delete_assingment(pool, assignment_id).await?)
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
    pool: PgPool,
    class_id: Uuid,
    assignment_name: String,
) -> Result<Vec<AssignmentStudent>, AssignmentError> {
    let assignment_id = get_assignment_id(&pool, &assignment_name, class_id).await?;
    Ok(get_assignment_students(&pool, assignment_id).await?)
}

pub async fn submit_assignment(
    pool: &PgPool,
    class_id: Uuid,
    assignment_id: Uuid,
    student_id: Uuid,
) -> Result<(), AssignmentError> {
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Range<T> {
    pub lower: T,
    pub upper: T,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Condition<T> {
    Range(Range<T>),
    EqualTo(T),
    LessThan(T),
    GreaterThan(T),
}
impl<T: PartialOrd> Condition<T> {
    pub fn matches(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        match self {
            Self::Range(range) => value >= &range.lower && value <= &range.upper,
            Self::EqualTo(expected) => value == expected,
            Self::LessThan(bound) => value < bound,
            Self::GreaterThan(bound) => value > bound,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StudentFilter {
    pub grade: Option<Condition<i32>>,
    pub name: Option<Condition<String>>,
}

//Add a filter option via queries to allow teacher to set students who should recieve the
//assignment.
pub async fn assign_to_student() {}
