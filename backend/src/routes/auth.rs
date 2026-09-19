use std::sync::Arc;

use crate::{AppState, errors::AuthError};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::phc::SaltString,
};
use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::Duration;

use base64::{Engine as _, engine::general_purpose};
use rand::{TryRng, rngs::SysRng};
use sha2::{Digest, Sha256};
use sqlx::{Executor, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
    pub jti: uuid::Uuid,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SignupRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}
impl IntoResponse for AuthResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
pub fn verify_token(token: &str, secret: &'static [u8]) -> Result<Claims, AuthError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_audience(&["my-app"]);
    validation.set_issuer(&["my-api"]);
    validation.required_spec_claims = std::collections::HashSet::from([
        "exp".to_string(),
        "iat".to_string(),
        "sub".to_string(),
        "iss".to_string(),
        "aud".to_string(),
        "jti".to_string(),
    ]);
    validation.leeway = 30;

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;
    Ok(token_data.claims)
}

pub fn generate_access_token(user_id: Uuid, secret: &'static [u8]) -> Result<String, AuthError> {
    let now = time::OffsetDateTime::now_utc();
    let claims = Claims {
        sub: user_id,
        exp: (now + Duration::minutes(10)).unix_timestamp(),
        iat: now.unix_timestamp(),
        iss: "my-api".to_string(),
        aud: "my-app".to_string(),
        jti: Uuid::new_v4(),
    };
    Ok(encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )?)
}

#[axum::debug_handler]
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<AuthResponse, AuthError> {
    let token_hash = hash_refresh_token(&req.refresh_token);
    let mut tx = state.pool.begin().await.map_err(AuthError::Database)?;

    let record = sqlx::query!(
        r#"
        SELECT id, user_id, expires_at
        FROM refresh_token
        WHERE token_hash = $1
        "#,
        token_hash
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(AuthError::Database)?
    .ok_or(AuthError::InvalidCredentials)?;

    if record.expires_at < OffsetDateTime::now_utc() {
        return Err(AuthError::TokenExpired);
    }

    sqlx::query!("DELETE FROM refresh_token WHERE id = $1", record.id)
        .execute(&mut *tx)
        .await
        .map_err(AuthError::Database)?;

    let new_refresh = generate_refresh_token()?;
    store_refresh_token(&mut *tx, record.user_id, &new_refresh).await?;
    tx.commit().await.map_err(AuthError::Database)?;

    let access_token = generate_access_token(record.user_id, state.jwt_secret)?;
    Ok(AuthResponse {
        access_token,
        refresh_token: new_refresh,
    })
}
pub fn generate_refresh_token() -> Result<String, AuthError> {
    let mut bytes = [0u8; 32];
    SysRng
        .try_fill_bytes(&mut bytes)
        .map_err(AuthError::Random)?;
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}
pub fn hash_refresh_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
pub async fn store_refresh_token<'e, E>(
    executor: E,
    user_id: uuid::Uuid,
    token: &str,
) -> Result<(), AuthError>
where
    E: Executor<'e, Database = Postgres>,
{
    let token_hash = hash_refresh_token(token);
    let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);
    sqlx::query!(
        r#"
        INSERT INTO refresh_token (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        token_hash,
        expires_at
    )
    .execute(executor)
    .await
    .map_err(AuthError::Database)?;

    Ok(())
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate();
    let hash = Argon2::default()
        .hash_password_with_salt(password.as_bytes(), salt.as_bytes())
        .map_err(|_| AuthError::PasswordHashing)?
        .to_string();

    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError::InvalidCredentials)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn auth_required(auth_header: &str, secret: &'static [u8]) -> Result<Claims, AuthError> {
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidCredentials)?;

    verify_token(token, secret)
}

pub struct AuthUser(pub Claims);

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingHeader("Authorization"))?;

        Ok(Self(auth_required(auth_header, state.jwt_secret)?))
    }
}

pub struct MaybeAuthUser(pub Option<Claims>);

impl FromRequestParts<Arc<AppState>> for MaybeAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let claims = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|auth_header| auth_required(auth_header, state.jwt_secret).ok());

        Ok(Self(claims))
    }
}
