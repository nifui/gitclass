use crate::{
    errors::AuthError,
    routes::temp::{
        models::*,
        repository::{
            create_session, create_user, delete_refresh_token, find_refresh_token, find_session,
            find_user_by_identifier, revoke_session, session_exists, store_refresh_token,
        },
    },
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::phc::SaltString,
};
use base64::{Engine as _, engine::general_purpose};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::{TryRng, rngs::SysRng};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

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

pub fn generate_refresh_token() -> Result<String, AuthError> {
    let mut bytes = [0u8; 32];
    SysRng
        .try_fill_bytes(&mut bytes)
        .map_err(AuthError::Random)?;
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

pub async fn refresh(
    pool: PgPool,
    req: RefreshRequest,
    jwt_secret: &'static [u8],
) -> Result<AuthResponse, AuthError> {
    let token_hash = hash_refresh_token(&req.refresh_token);
    let mut tx = pool.begin().await.map_err(AuthError::Database)?;
    //Ensure the session is still valid and has not been revoked.
    let record = find_refresh_token(&pool, token_hash)
        .await?
        .ok_or(AuthError::TokenExpired)?;
    if record.expires_at < OffsetDateTime::now_utc() {
        return Err(AuthError::TokenExpired);
    }
    if let Some(timestamp) = record.revoked_at
        && timestamp < OffsetDateTime::now_utc()
    {
        return Err(AuthError::SessionRevoked);
    }

    delete_refresh_token(&mut *tx, record.id).await?;

    let new_refresh = generate_refresh_token()?;
    store_refresh_token(&mut *tx, &new_refresh, record.session_id).await?;
    tx.commit().await.map_err(AuthError::Database)?;

    let access_token = generate_access_token(record.user_id, record.session_id, jwt_secret)?;
    Ok(AuthResponse {
        access_token,
        refresh_token: new_refresh,
    })
}
pub async fn signup(
    pool: PgPool,
    meta: ClientMeta,
    req: SignupRequest,
    jwt_secret: &'static [u8],
) -> Result<AuthResponse, AuthError> {
    if req.password.len() < 8 {
        return Err(AuthError::InvalidInput(
            "password must be at least 8 characters",
        ));
    }
    let email = req.email.to_lowercase();
    let password_hash = hash_password(&req.password)?;
    let user_id = create_user(&pool, email, req.username, password_hash).await?;
    let session_id = create_session(&pool, user_id, &meta).await?;
    let access_token = generate_access_token(user_id, session_id, jwt_secret)?;
    let refresh_token = generate_refresh_token()?;
    store_refresh_token(&pool, &refresh_token, session_id).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
    })
}
pub async fn signin(
    pool: PgPool,
    meta: ClientMeta,
    req: SigninRequest,
    jwt_secret: &'static [u8],
) -> Result<SigninResponse, AuthError> {
    const DUMMY_HASH: &str =
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$7vQm0zFjI4G0g8m1QqY6K6v6T6XQ3V8lYQv8h0w5W0A";
    let user = find_user_by_identifier(&pool, &req.identifier).await?;

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

    let session_id = if let Some(id) = find_session(&pool, user_id, &meta).await? {
        id
    } else {
        create_session(&pool, user_id, &meta).await?
    };

    let access_token = generate_access_token(user_id, session_id, jwt_secret)?;

    let refresh_token = generate_refresh_token()?;

    store_refresh_token(&pool, &refresh_token, session_id).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
    })
}

pub async fn signout(pool: PgPool, session_id: Uuid, user_id: Uuid) -> Result<(), AuthError> {
    revoke_session_by_id(pool, user_id, session_id).await
}

async fn revoke_session_by_id(
    pool: PgPool,
    user_id: Uuid,
    session_id: Uuid,
) -> Result<(), AuthError> {
    let mut tx = pool.begin().await.map_err(AuthError::Database)?;
    revoke_session(&mut *tx, user_id, session_id).await?;

    tx.commit().await.map_err(AuthError::Database)?;
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
    let valid_sid = session_exists(executor, claims.sub, claims.sid).await?;
    if !valid_sid {
        Err(AuthError::SessionRevoked)
    } else {
        Ok(claims)
    }
}
