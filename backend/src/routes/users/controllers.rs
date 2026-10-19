use std::sync::Arc;

use axum::extract::State;
use utoipa_axum::router::OpenApiRouter;

use crate::{AppState, middleware::auth::AuthUser, routes::classes::ClassError};

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}

pub async fn get_assosciated_classes(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<(), ClassError> {
    Ok(())
}

pub async fn get_grades() {}
pub async fn leave_class(State(state): State<Arc<AppState>>, AuthUser(claims): AuthUser) {}
pub async fn join_class() {}
pub async fn create_class() {}
pub async fn get_assignments() {}
pub async fn get_assignment() {}
//Refer to local assignment_student.id
#[utoipa::path(
    get, 
    path = "/user/assignments/{assignment_id}",
    responses(
        (status = 200, description = "Successfully grabbed the assignments ")
    )
)]
pub async fn submit_assignment() {
    //Simple change in type.
}
