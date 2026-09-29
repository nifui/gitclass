use crate::{errors::AuthError, routes::temp::models::Claims};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::phc::SaltString,
};
use base64::{Engine as _, engine::general_purpose};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::{TryRng, rngs::SysRng};
use sha2::{Digest, Sha256};
use time::Duration;
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
