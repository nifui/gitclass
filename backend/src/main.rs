use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::Extension;
use backend::{AppState, routes::docs::docs_routes};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::sync::{Arc, OnceLock};
use tokio::net::TcpListener;

//This could be changed to a static array as JWT_SECRET tokens should be of constant size.
static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

#[inline(always)]
pub fn jwt_secret() -> &'static [u8] {
    JWT_SECRET.get_or_init(|| {
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set")
            .into_bytes()
    })
}

#[tokio::main]
async fn main() {
    //Load vars first. If no ENV vars theres no point continuing.
    dotenv().ok();
    let serve_address = std::env::var("SERVE_ADDRESS").expect("SERVE_ADDRESS must be set.");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    aide::generate::on_error(|error| {
        println!("{error}");
    });
    aide::generate::extract_schemas(true);

    let mut api = OpenApi::default();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let state = Arc::new(AppState {
        pool,
        jwt_secret: jwt_secret(),
    });

    let app = ApiRouter::new()
        .nest_api_service("/docs", docs_routes(state.clone()))
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)))
        .with_state(state);

    let listener = TcpListener::bind(&serve_address).await.unwrap();

    println!("Backend being served at {:?}", serve_address);

    axum::serve(listener, app).await.unwrap();
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Git Classroom")
        .summary(
            "Replacement for Github classroom for managing student grades across code repositories.",
        )
        .description(include_str!("../../README.md"))
}
