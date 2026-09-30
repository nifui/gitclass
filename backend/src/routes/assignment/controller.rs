use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::router::OpenApiRouter;

use crate::{AppState, middleware::auth::AdminUser, routes::assignment::AssignmentError};

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
