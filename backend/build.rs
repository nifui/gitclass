use dotenvy::dotenv;
use sqlx::migrate::Migrator;
use std::path::Path;
#[tokio::main]
async fn main() {
    dotenv().ok();
    // Tell Cargo to rerun this script if migration files change
    //println!("cargo:rerun-if-changed=migrations");

    // Read DATABASE_URL from environment
    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for build-time migrations");

    // Connect and run migrations
    let pool = sqlx::PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to database during build");

    let migrator = Migrator::new(Path::new("./migrations"))
        .await
        .expect("Failed to create migrator");

    migrator
        .run(&pool)
        .await
        .expect("Failed to run migrations during build");
}
