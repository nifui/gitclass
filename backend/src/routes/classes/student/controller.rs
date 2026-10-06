use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

// /class/{class_id}/students/{student_id} = fetch general student information.
// /class/{class_id}/students/{student_id}/grades = fetch student grades.
// /class/{class_id}/students/{student_id}/assignments
pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
