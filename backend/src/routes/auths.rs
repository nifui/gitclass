use crate::{
    AppState,
    errors::{ApiError, AuthError},
    map_sqlx_error,
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::phc::SaltString,
};
use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use time::Duration;

use base64::{Engine as _, engine::general_purpose};
use rand::{TryRng, rngs::SysRng};
use sha2::{Digest, Sha256};
use sqlx::{Executor, Postgres, types::ipnetwork::IpNetwork};
use time::OffsetDateTime;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use axum_client_ip::ClientIp;

#[derive(Debug, Clone)]
pub struct ClientMeta {
    pub ip_address: String,
    pub user_agent: String,
    pub device_fingerprint: Option<String>,
}

impl<S> FromRequestParts<S> for ClientMeta
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract IP Address
        // InsecureClientIp checks X-Forwarded-For, X-Real-IP, and ConnectInfo sequentially
        let ip_address = ClientIp::from_request_parts(parts, state)
            .await
            .map(|ClientIp(ip)| ip.to_string())
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        // 2. Extract User-Agent (Browser / OS metadata)
        let user_agent = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();

        // 3. Optional: Custom Header (e.g., X-Device-Id or X-Fingerprint from a mobile client)
        let device_fingerprint = parts
            .headers
            .get("X-Device-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Ok(ClientMeta {
            ip_address,
            user_agent,
            device_fingerprint,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    //The resource being targetted.
    pub sub: Uuid,
    //Expiration
    pub exp: i64,
    //Issued at
    pub iat: i64,
    //Who issued the token
    pub iss: String,
    //Audience
    pub aud: String,
    //Session id.
    pub sid: Uuid,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SignupRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}
impl IntoResponse for AuthResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
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
    ]);
    //Check the session id
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
        sid: Uuid::new_v4(),
    };

    Ok(encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )?)
}
pub fn hash_refresh_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub async fn store_refresh_token<'e, E>(executor: E, token: &str) -> Result<(), AuthError>
where
    E: Executor<'e, Database = Postgres>,
{
    let token_hash = hash_refresh_token(token);
    let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (token_hash, expires_at)
        VALUES ($1, $2)
        "#,
        token_hash,
        expires_at
    )
    .execute(executor)
    .await
    .map_err(AuthError::Database)?;

    Ok(())
}
pub fn generate_refresh_token() -> Result<String, AuthError> {
    let mut bytes = [0u8; 32];
    SysRng
        .try_fill_bytes(&mut bytes)
        .map_err(AuthError::Random)?;
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

//Create a session first.

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<AuthResponse, AuthError> {
    let token_hash = hash_refresh_token(&req.refresh_token);
    let mut tx = state.pool.begin().await.map_err(AuthError::Database)?;
    let record = sqlx::query!(
        r#"
        SELECT 
        "#
    );
    let record = sqlx::query!(
        r#"
        SELECT id, session_id, expires_at
        FROM refresh_tokens
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

    sqlx::query!("DELETE FROM refresh_tokens WHERE id = $1", record.id)
        .execute(&mut *tx)
        .await
        .map_err(AuthError::Database)?;

    let new_refresh = generate_refresh_token()?;
    store_refresh_token(&mut *tx, &new_refresh).await?;
    tx.commit().await.map_err(AuthError::Database)?;

    let access_token = generate_access_token(record.session_id, state.jwt_secret)?;
    Ok(AuthResponse {
        access_token,
        refresh_token: new_refresh,
    })
}
pub async fn signup(
    State(state): State<Arc<AppState>>,
    meta: ClientMeta,
    Json(req): Json<SignupRequest>,
) -> Result<AuthResponse, ApiError> {
    if req.password.len() < 8 {
        return Err(AuthError::InvalidInput(
            "password must be at least 8 characters",
        ));
    }
    let email = req.email.to_lowercase();
    let password_hash = hash_password(&req.password)?;

    let user_id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (email, username, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        email,
        req.username,
        password_hash
    )
    .fetch_one(&state.pool)
    .await
    .map_err(map_sqlx_error)?;
    let session_id = sqlx::query_scalar!(
        r#"
    INSERT INTO sessions 
        (user_id, 
         device_name, 
         ip_address, 
         user_agent) 
    VALUES ($1, $2, $3, $4)
    RETURNING id
    "#,
        user_id,
        meta.device_fingerprint,
        IpNetwork::from_str(&meta.ip_address)?,
        meta.user_agent
    );

    //Create a user. Then we issue a session for the user.
    //Once we issued the session we hand out a refresh token.
    //Then we hand out a access token based off of the refresh token.

    let access_token = generate_access_token(user_id, state.jwt_secret)?;
    let refresh_token = generate_refresh_token()?;
    store_refresh_token(&state.pool, &refresh_token).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
    })
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<AuthResponse, AuthError> {
    const DUMMY_HASH: &str =
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$7vQm0zFjI4G0g8m1QqY6K6v6T6XQ3V8lYQv8h0w5W0A";

    let user = sqlx::query!(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE email = $1 OR username = $1
        "#,
        req.identifier
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(AuthError::Database)?;

    let (user_id, password_hash) = match user {
        Some(u) => (Some(u.id), u.password_hash),
        None => (None, DUMMY_HASH.to_string()),
    };

    let is_valid = verify_password(&req.password, &password_hash)?;
    if user_id.is_none() || !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    let user_id = user_id.unwrap();
    let access_token = generate_access_token(user_id, state.jwt_secret)?;
    let refresh_token = generate_refresh_token()?;
    store_refresh_token(&state.pool, &refresh_token).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
    })
}
