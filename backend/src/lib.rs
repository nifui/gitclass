pub mod errors;
pub mod routes;
pub mod utils;

use sqlx::PgPool;

use crate::errors::AuthError;

pub fn map_sqlx_error(err: sqlx::Error) -> AuthError {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.code().as_deref() == Some("23505")
    {
        return match db_err.constraint() {
            Some("users_email_key") => AuthError::DuplicateEmail,
            Some("users_username_key") => AuthError::DuplicateUsername,
            _ => AuthError::Database(err),
        };
    }
    AuthError::Database(err)
}
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
