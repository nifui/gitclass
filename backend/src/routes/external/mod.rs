use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

pub mod identities;
pub mod organizations;
pub mod providers;
pub mod repositories;
pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
