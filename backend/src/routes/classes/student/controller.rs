use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

// /classes/{class_id}/students/{student_id} = fetch general student information.
// /classes/{class_id}/students/{student_id}/grades = fetch student grades (might remove as this can
// be calculated by the frontend)
// /classes/{class_id}/students/{student_id}/assignments = fetch all assignments belonging to a
// student.
// /classes/{class_id}/students/{student_id}/assignments/{assignment_id} = returns info on a specific
// assignment that belongs to a student.
// /classes/{class_id}/students/{student_id}/remove = removes a student from the given class
pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
