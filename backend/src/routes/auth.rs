use crate::errors::AuthError;
use jsonwebtoken::{EncodingKey, Header, encode};
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
    Ok(())
}
