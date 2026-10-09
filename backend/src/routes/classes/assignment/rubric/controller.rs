use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
