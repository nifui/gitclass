use std::sync::Arc;

use axum::extract::State;

use crate::{AppState, middleware::auth::AuthUser, routes::classes::ClassError};

pub async fn get_assosciated_classes(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<(), ClassError> {
    Ok(())
}

pub async fn get_grades() {}
pub async fn leave_class() {}
pub async fn join_class() {}
pub async fn create_class() {}
