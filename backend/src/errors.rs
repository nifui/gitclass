use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ApiError {
    // Authentication
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("authentication required")]
    Unauthorized,

    #[error("invalid token")]
    InvalidToken,

    #[error("token expired")]
    TokenExpired,

    #[error("missing authorization header")]
    MissingAuthHeader,

    // Authorization
    #[error("forbidden")]
    Forbidden,

    // Validation
    #[error("validation failed: {0}")]
    Validation(String),

    // Resources
    #[error("repository not found")]
    RepositoryNotFound,

    #[error("repository already exists")]
    RepositoryExists,

    #[error("file not found")]
    FileNotFound,

    // Operations
    #[error("filesystem operation failed")]
    Filesystem,

    #[error("git operation failed")]
    Git,

    #[error("database operation failed")]
    Database,

    #[error("internal server error")]
    Internal,

    // Specific cases
    #[error("commit listing failed")]
    CommitListing,

    #[error("unsupported file type")]
    UnsupportedFileType,
}

impl ApiError {
    const fn status(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials
            | Self::Unauthorized
            | Self::InvalidToken
            | Self::TokenExpired => StatusCode::UNAUTHORIZED,

            Self::MissingAuthHeader | Self::Validation(_) | Self::CommitListing => {
                StatusCode::BAD_REQUEST
            }

            Self::Forbidden => StatusCode::FORBIDDEN,

            Self::RepositoryNotFound | Self::FileNotFound => StatusCode::NOT_FOUND,

            Self::RepositoryExists => StatusCode::CONFLICT,

            Self::UnsupportedFileType => StatusCode::UNSUPPORTED_MEDIA_TYPE,

            Self::Filesystem | Self::Git | Self::Database | Self::Internal => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    const fn code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "invalid_credentials",
            Self::Unauthorized => "unauthorized",
            Self::InvalidToken => "invalid_token",
            Self::TokenExpired => "token_expired",
            Self::MissingAuthHeader => "missing_auth_header",

            Self::Forbidden => "forbidden",

            Self::Validation(_) => "validation_error",

            Self::RepositoryNotFound => "repository_not_found",
            Self::RepositoryExists => "repository_exists",

            Self::FileNotFound => "file_not_found",

            Self::Filesystem => "filesystem_error",
            Self::Git => "git_error",
            Self::Database => "database_error",
            Self::Internal => "internal_error",

            Self::CommitListing => "commit_listing_failed",

            Self::UnsupportedFileType => "unsupported_file_type",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let code = self.code();

        tracing::error!(
            status = %status,
            error_code = code,
            error = ?self
        );

        (
            status,
            Json(ErrorResponse {
                code,
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}
impl From<std::io::Error> for ApiError {
    fn from(_: std::io::Error) -> Self {
        Self::Filesystem
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(_: sqlx::Error) -> Self {
        Self::Database
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;

        match err.kind() {
            ErrorKind::ExpiredSignature => Self::TokenExpired,
            _ => Self::InvalidToken,
        }
    }
}
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("token expired")]
    TokenExpired,
    #[error("missing header: {0}")]
    MissingHeader(&'static str),
    #[error("duplicate email")]
    DuplicateEmail,
    #[error("duplicate username")]
    DuplicateUsername,
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("password hashing failed")]
    PasswordHashing,
    #[error("password verification failed")]
    PasswordVerification,
    #[error("token generation failed")]
    TokenGen,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("randomness generator error: {0}")]
    Random(#[from] rand::rngs::SysError),
    #[error("invalid client IP")]
    InvalidIp(#[from] ipnetwork::IpNetworkError),
}

impl From<AuthError> for ApiError {
    fn from(_: AuthError) -> Self {
        Self::Internal
    }
}
