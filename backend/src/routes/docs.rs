use std::sync::Arc;

use aide::redoc::Redoc;
use aide::{
    axum::{
        ApiRouter, IntoApiResponse,
        routing::{get, get_with},
    },
    openapi::OpenApi,
};
use axum::{Extension, Json, response::IntoResponse};

use crate::AppState;

pub fn docs_routes(state: Arc<AppState>) -> ApiRouter {
    aide::generate::infer_responses(true);

    let router: ApiRouter = ApiRouter::new()
        .api_route_with(
            "/swagger",
            get_with(
                Redoc::new("/docs/private/api.json")
                    .with_title("Aide Axum")
                    .axum_handler(),
                |op| op.description("This documentation page."),
            ),
            |p| p,
        )
        .route("/private/api.json", get(serve_docs))
        .with_state(state);
    aide::generate::infer_responses(false);

    router
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api).into_response()
}
