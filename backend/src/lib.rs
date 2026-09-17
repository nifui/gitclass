pub mod errors;
pub mod routes;
pub mod utils;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: &'static [u8],
}

//Idea for allowing swapping Git Providers if in the future a self-hosted git storage is desired.
pub trait GitProvider {
    fn create_repository();
    fn delete_repository();
    fn manage_users();
    fn repository_metadata();
}

//Import froma specific file format to make it less of a hassle.
fn import_from_csv(path: &str) {}
