use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    AppState,
    middleware::auth::AuthUser,
    routes::classes::{
        ClassError,
        assignment::{
            AssignmentError,
            models::{AssignAssignmentRequest, Assignment, AssignmentStudent, CreateAssignment},
            services,
        },
    },
};

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
        .routes(routes!(create_assignment))
        .routes(routes!(list_assignments))
        .routes(routes!(get_assignment))
        .routes(routes!(delete_assignment))
        .routes(routes!(assign_to_student))
        .routes(routes!(submit_assignment))
        .routes(routes!(assign_to_class))
}

//Top level routes for assignment-relation (move to different routes).
//
// Create assignment
// Link template to assignment
// Remove template to assignment
// Assign assignment to class.
// Unassign assignment to class.
// Modify class assignment
// Delete assignment
// Get class_assignment (assignment + class_assignment as it contains the metadata)
// Get specific student assignment
// Unassign student assignment
// List class assignments
// List student assignments
// Change assignment status.
//
// Repositories
// Set assignment template
// Set assignment status
// Assign assignment to class
// Modify class assignment
// Delete assignment
// Get class assignment
// Get student assignments
//
#[utoipa::path(
    post,
    path = "/classes/{class_id}/assignments",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class")
    ),
    request_body = CreateAssignment,
    responses(
        (status = 201, description = "Assignment created successfully", body = Assignment),
        (status = 400, description = "Invalid request or class parameters"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Class not found"),
        (status = 500, description = "Failed to create assignment")
    ),
    tag = "Assignments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateAssignment>,
) -> Result<Json<Assignment>, AssignmentError> {
    services::create_assignment(&state.pool, req, claims.sub)
        .await
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/classes/{class_id}/assignments",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class")
    ),
    responses(
        (status = 200, description = "Assignments retrieved successfully", body = [Assignment]),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Class not found"),
        (status = 500, description = "Database error")
    ),
    tag = "Assignments",
    security(
        ("bearer_auth" = [])
    )
)]

pub async fn list_assignments(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(class_id): Path<Uuid>,
) -> Result<Json<Vec<Assignment>>, ClassError> {
    Ok(Json(Vec::default()))
}

#[utoipa::path(
    get,
    path = "/classes/{class_id}/assignments/{assignment_id}",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class"),
        ("assignment_id" = Uuid, Path, description = "ID of the assignment")
    ),
    responses(
        (status = 200, description = "Assignment retrieved successfully", body = Assignment),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Assignment or class not found"),
        (status = 500, description = "Database error")
    ),
    tag = "Assignments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path((class_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Assignment>, AssignmentError> {
    services::get_assignment(&state.pool, claims.sub, class_id, assignment_id)
        .await
        .map(Json)
}

#[utoipa::path(
    delete,
    path = "/classes/{class_id}/assignments/{assignment_id}",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class"),
        ("assignment_id" = Uuid, Path, description = "ID of the assignment")
    ),
    responses(
        (status = 204, description = "Assignment deleted successfully"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Assignment or class not found"),
        (status = 500, description = "Database error")
    ),
    tag = "Assignments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path((class_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AssignmentError> {
    services::delete_assignment(&state.pool, claims.sub, class_id, assignment_id)
        .await
        .map(|_| ())
}

#[utoipa::path(
    post,
    path = "/classes/{class_id}/assignments/{assignment_id}/students",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class"),
        ("assignment_id" = Uuid, Path, description = "ID of the assignment")
    ),
    request_body = AssignAssignmentRequest,
    responses(
        (status = 201, description = "Student assigned successfully", body = AssignmentStudent),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Assignment, class, or student not found"),
        (status = 500, description = "Database error")
    ),
    tag = "Assignments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn assign_to_student(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path((class_id, assignment_id)): Path<(Uuid, Uuid)>,
    Json(student_id): Json<Uuid>,
) -> Result<Json<AssignmentStudent>, AssignmentError> {
    services::assign_student(&state.pool, claims.sub, class_id, assignment_id, student_id)
        .await
        .map(Json)
}

#[utoipa::path(
    post,
    path = "/classes/{class_id}/assignments/{assignment_id}/submit",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class"),
        ("assignment_id" = Uuid, Path, description = "ID of the assignment")
    ),
    responses(
        (status = 200, description = "Assignment submitted successfully"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Assignment not found"),
        (status = 500, description = "Database error")
    ),
    tag = "Assignments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn submit_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path((class_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<bool>, AssignmentError> {
    services::submit_assignment(&state.pool, assignment_id, claims.sub, class_id)
        .await
        .map(Json)
}

#[utoipa::path(
    post,
    path = "/classes/{class_id}/assignments/{assignment_id}/assign-class",
    params(
        ("class_id" = Uuid, Path, description = "ID of the class"),
        ("assignment_id" = Uuid, Path, description = "ID of the assignment")
    ),
    responses(
        (status = 200, description = "Assignment associated with the class successfully"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Assignment or class not found"),
        (status = 500, description = "Database error")
    ),
    tag = "Assignments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn assign_to_class(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path((class_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<bool>, AssignmentError> {
    services::assign_to_class(&state.pool, claims.sub, assignment_id, class_id)
        .await
        .map(Json)
}

pub async fn update_assignment() {
    //Requires modifying other tables depending on what was changed in the assignment.
    //
}
