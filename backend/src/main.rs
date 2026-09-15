use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use axum::Router;
use backend::AppState;
use dotenvy::dotenv;
use sqlx::PgPool;

//This could be changed to a static array as JWT_SECRET tokens should be of constant size.
static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

pub fn jwt_secret() -> &'static [u8] {
    JWT_SECRET.get_or_init(|| {
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set")
            .into_bytes()
    })
}
#[tokio::main]
async fn main() {
    dotenv().ok();
    let host_address = std::env::var("SERVE_ADDRESS").unwrap();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(&db_url).await.unwrap();

    let state = Arc::new(AppState {
        pool,
        jwt_secret: jwt_secret(),
    });
    //For development testing.
    // sqlx::migrate!("./migrations").run(&pool).await?;
    let listener = tokio::net::TcpListener::bind(host_address.clone())
        .await
        .unwrap();
    let app = Router::<()>::new();
    axum::serve(listener, app).await.unwrap();
}
