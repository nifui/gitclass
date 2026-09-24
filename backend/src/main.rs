use backend::AppState;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::sync::{Arc, OnceLock};
use tokio::net::TcpListener;
use utoipa::{
    OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
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
    OpenApiRouter::with_openapi(ApiDoc::openapi())
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

    let (router, mut openapi) = router()
        .routes(routes!(health))
        .with_state(state)
        .split_for_parts();

    openapi.components.as_mut().unwrap().add_security_scheme(
        "bearerAuth",
        SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
    );

    openapi.info.title = String::from("GitClassroom API");
    openapi.info.description = Some(String::from("Managing repositories for assignments"));
    openapi.info.contact = None;
    openapi.info.version = String::from("1.0.0");
    openapi.info.license = None;

    let app = router.merge(Redoc::with_url("/redoc", openapi));

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
