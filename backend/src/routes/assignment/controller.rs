use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    AppState,
    middleware::auth::AuthUser,
    routes::assignment::{
        AssignmentError,
        models::{Assignment, CreateAssignment},
        services,
    },
};

pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateAssignment>,
) -> Result<Assignment, AssignmentError> {
    services::create_assignment(&state.pool, req, claims.sub).await
}

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
