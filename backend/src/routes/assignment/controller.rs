use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    AppState,
    routes::{
        assignment::{AssignmentError, repository::CreateAssignmentPayload},
        auth::AdminUser,
    },
};

pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    AdminUser(claims): AdminUser,
    Json(payload): Json<CreateAssignmentPayload>,
) -> Result<(), AssignmentError> {
    Ok(())
}

pub fn router() -> OpenApiRouter {
    OpenApiRouter::default()
}
