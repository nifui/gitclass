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
use serde::{Deserialize, Serialize};
use time::Duration;

use base64::{Engine as _, engine::general_purpose};
use rand::{TryRng, rngs::SysRng};
use sha2::{Digest, Sha256};
use sqlx::{Executor, Postgres};
use time::OffsetDateTime;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    //The resource being targetted.
    pub sub: Uuid,
    //Expiration
    pub exp: i64,
    //Issued at
    pub iat: i64,
    //Who issued the token
    pub iss: i64,
    //Audience
    pub aud: i64,
    //JWT ID
    pub jti: Uuid,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
