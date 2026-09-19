use axum::{Router, routing::get};
use backend::{AppState, routes::auth};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::sync::{Arc, OnceLock};
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_redoc::{Redoc, Servable};

static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();
#[inline(always)]
pub fn jwt_secret() -> &'static [u8] {
    JWT_SECRET.get_or_init(|| {
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set")
            .into_bytes()
    })
}
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Backend API",
        version = "0.1.0",
        description = "Backend HTTP API"
    ),
    tags(
        (name = "users", description = "User endpoints")
    )
)]
struct ApiDoc;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::with_openapi(ApiDoc::openapi()).merge(auth::router())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let serve_address = std::env::var("SERVE_ADDRESS").expect("SERVE_ADDRESS must be set.");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = Arc::new(AppState {
        pool,
        jwt_secret: jwt_secret(),
    });

    let (app, api) = router().with_state(state).split_for_parts();

    let app = app.merge(Redoc::with_url("/redoc", api));

    let listener = TcpListener::bind(&serve_address)
        .await
        .expect("Failed to bind listener");

    println!("Backend being served at {}", serve_address);

    axum::serve(listener, app).await.unwrap();
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "users",
    responses(
        (status = 200, description = "Service is healthy")
    )
)]
async fn health() -> &'static str {
    "OK"
}
