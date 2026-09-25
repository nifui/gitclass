use crate::{
    AppState,
    errors::{ApiError, AuthError},
    map_sqlx_error,
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::phc::SaltString,
};
use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{StatusCode, header, request::Parts},
    response::IntoResponse,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use time::Duration;

use base64::{Engine as _, engine::general_purpose};
use rand::{TryRng, rngs::SysRng};
use sha2::{Digest, Sha256};
use sqlx::{Executor, PgPool, Postgres, types::ipnetwork::IpNetwork};
use time::OffsetDateTime;
use utoipa::ToSchema;
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

        Ok(Self {
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
pub struct SignoutRequest {
    //Require a refresh token as we want toe examine the session_id and log out of it.
    pub refresh_token: String,
    //We look at the jti of this if we do implement jti and put it into Redis if we do use Redis.
    //The jti will live as long as the ttl of the jwt.
    pub access_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SigninRequest {
    pub identifier: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}
type SigninResponse = AuthResponse;

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

pub fn generate_access_token(
    user_id: Uuid,
    session_id: Uuid,
    secret: &'static [u8],
) -> Result<String, AuthError> {
    let now = time::OffsetDateTime::now_utc();
    let claims = Claims {
        sub: user_id,
        exp: (now + Duration::minutes(10)).unix_timestamp(),
        iat: now.unix_timestamp(),
        iss: "my-api".to_string(),
        aud: "my-app".to_string(),
        sid: session_id,
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

pub async fn store_refresh_token<'e, E>(
    executor: E,
    token: &str,
    session_id: Uuid,
) -> Result<(), AuthError>
where
    E: Executor<'e, Database = Postgres>,
{
    let token_hash = hash_refresh_token(token);
    let expires_at = OffsetDateTime::now_utc() + Duration::days(7);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens
        (token_hash, session_id, expires_at)
        VALUES ($1, $2, $3)
        "#,
        token_hash,
        session_id,
        expires_at,
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

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<AuthResponse, AuthError> {
    let token_hash = hash_refresh_token(&req.refresh_token);
    let mut tx = state.pool.begin().await.map_err(AuthError::Database)?;
    //Ensure the session is still valid and has not been revoked.
    let record = sqlx::query!(
        r#"
        SELECT 
            rt.id, 
            rt.session_id, 
            rt.expires_at, 
            s.user_id,
            s.revoked_at
        FROM refresh_tokens rt
        JOIN sessions s ON rt.session_id = s.id
        WHERE rt.token_hash = $1
        AND s.revoked_at != $2
        "#,
        token_hash,
        OffsetDateTime::now_utc()
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(AuthError::Database)?
    .ok_or(AuthError::InvalidCredentials)?;

    if record.expires_at < OffsetDateTime::now_utc() {
        return Err(AuthError::TokenExpired);
    }

    if let Some(timestamp) = record.revoked_at
        && timestamp < OffsetDateTime::now_utc()
    {
        return Err(AuthError::SessionRevoked);
    }

    sqlx::query!("DELETE FROM refresh_tokens WHERE id = $1", record.id)
        .execute(&mut *tx)
        .await
        .map_err(AuthError::Database)?;

    let new_refresh = generate_refresh_token()?;
    store_refresh_token(&mut *tx, &new_refresh, record.session_id).await?;
    tx.commit().await.map_err(AuthError::Database)?;

    let access_token = generate_access_token(record.user_id, record.session_id, state.jwt_secret)?;
    Ok(AuthResponse {
        access_token,
        refresh_token: new_refresh,
    })
}
pub async fn create_session(
    executor: &PgPool,
    user_id: Uuid,
    meta: &ClientMeta,
) -> Result<Uuid, AuthError> {
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
    )
    .fetch_one(executor)
    .await
    .map_err(map_sqlx_error)?;

    Ok(session_id)
}

//Verify the request is valid(password length/strength, eg)
//Create a user.
//Issue a session for the user.
//Once we issued the session we hand out a refresh token.
//Then we hand out a access token based off of the refresh token.
//Return an access and refresh token.
//
pub async fn signup(
    State(state): State<Arc<AppState>>,
    meta: ClientMeta,
    Json(req): Json<SignupRequest>,
) -> Result<AuthResponse, AuthError> {
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

    //Generate an access token based off of the session id
    let session_id = create_session(&state.pool, user_id, &meta).await?;
    let access_token = generate_access_token(user_id, session_id, state.jwt_secret)?;
    let refresh_token = generate_refresh_token()?;
    store_refresh_token(&state.pool, &refresh_token, session_id).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
    })
}

pub async fn signin(
    State(state): State<Arc<AppState>>,
    meta: ClientMeta,
    Json(req): Json<SigninRequest>,
) -> Result<SigninResponse, AuthError> {
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

    let is_correct = verify_password(&req.password, &password_hash)?;

    if user_id.is_none() || !is_correct {
        return Err(AuthError::InvalidCredentials);
    }
    //Safely unwrap as we prevented the None case ^.
    let user_id = user_id.unwrap();

    let record = sqlx::query!(
        r#"
        SELECT id 
        FROM sessions
        WHERE user_id = $1
            AND device_name = $2
            AND ip_address = $3 
            AND user_agent  = $4
        "#,
        user_id,
        meta.device_fingerprint,
        IpNetwork::from_str(&meta.ip_address)?,
        meta.user_agent,
    )
    .fetch_optional(&state.pool)
    .await?;

    let session_id = if let Some(record) = record {
        record.id
    } else {
        create_session(&state.pool, user_id, &meta).await?
    };

    let access_token = generate_access_token(user_id, session_id, state.jwt_secret)?;

    let refresh_token = generate_refresh_token()?;

    store_refresh_token(&state.pool, &refresh_token, session_id).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
    })
}

pub async fn signout(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<(), AuthError> {
    revoke_session_by_id(&state.pool, claims.sub, claims.sid).await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevocationRequest {
    refresh_token: String,
    access_token: String,
    session_id: Uuid,
}

async fn revoke_session_by_id(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Uuid,
) -> Result<(), AuthError> {
    let mut tx = pool.begin().await.map_err(AuthError::Database)?;

    let result = sqlx::query!(
        r#"
        UPDATE sessions
        SET revoked_at = NOW()
        WHERE id = $1
          AND user_id = $2
          AND revoked_at IS NULL
        "#,
        session_id,
        user_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(AuthError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AuthError::SessionRevoked);
    }

    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE session_id = $1
        "#,
        session_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(AuthError::Database)?;

    tx.commit().await.map_err(AuthError::Database)?;

    Ok(())
}

pub async fn revoke_all_sessions(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE sessions
        SET revoked_at = NOW()
        WHERE user_id = $1
          AND revoked_at IS NULL
        "#,
        claims.sub,
    )
    .execute(&state.pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(())
}

pub async fn auth_required(
    executor: &PgPool,
    auth_header: &str,
    secret: &'static [u8],
) -> Result<Claims, AuthError> {
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidCredentials)?;
    let claims = verify_token(token, secret)?;
    //Can be removed and replaced for Redis/Valkey if db reads are a bottleneck.

    let valid_sid = sqlx::query_scalar!(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM sessions
                WHERE id = $1
                  AND user_id = $2
                  AND revoked_at IS NULL
            )
            "#,
        claims.sid,
        claims.sub,
    )
    .fetch_one(executor)
    .await
    .map_err(AuthError::Database)?
    .unwrap_or(false);

    if !valid_sid {
        Err(AuthError::SessionRevoked)
    } else {
        Ok(claims)
    }
}

pub struct AuthUser(pub Claims);

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingHeader("Authorization"))?;

        Ok(Self(
            auth_required(&state.pool, auth_header, state.jwt_secret).await?,
        ))
    }
}

pub struct MaybeAuthUser(pub Option<Claims>);

impl FromRequestParts<Arc<AppState>> for MaybeAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let claims = match auth_header {
            Some(auth_header) => auth_required(&state.pool, auth_header, state.jwt_secret)
                .await
                .ok(),
            None => None,
        };

        Ok(Self(claims))
    }
}

pub async fn generate_admin_code() {}
