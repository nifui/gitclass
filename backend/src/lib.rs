pub mod errors;
pub mod routes;
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: &'static [u8],
}
//Import froma specific file format to make it less of a hassle.
fn import_from_csv(path: &str) {}
