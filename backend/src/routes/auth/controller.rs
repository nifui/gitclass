use std::sync::Arc;

use crate::{
    AppState, errors::AuthError, middleware::auth::AuthUser, routes::auth::{
        models::{
            AuthResponse, ClientMeta, RefreshRequest, SigninRequest, SigninResponse, SignupRequest,
        }, repository::get_active_admin_pin, services::{self, revoke_session_by_id, verify_password},
    },
};
use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 200, description = "Successfully registered and signed in", body = AuthResponse),
        (status = 400, description = "Invalid input or password too short"),
        (status = 409, description = "Email or username already exists")
    ),
    tag = "Authentication"
)]
pub async fn signup(
    State(state): State<Arc<AppState>>,
    meta: ClientMeta,
    Json(req): Json<SignupRequest>,
) -> Result<AuthResponse, AuthError> {
    services::signup(&state.pool, meta, req, state.jwt_secret).await
}
#[utoipa::path(
    post,
    path = "/auth/signin",
    request_body = SigninRequest,
    responses(
        (status = 200, description = "Successfully authenticated", body = AuthResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "Authentication"
)]
pub async fn signin(
    State(state): State<Arc<AppState>>,
    meta: ClientMeta,
    Json(req): Json<SigninRequest>,
) -> Result<SigninResponse, AuthError> {
    services::signin(state.pool.clone(), meta, req, state.jwt_secret).await
}
#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "New tokens issued", body = AuthResponse),
        (status = 401, description = "Invalid or expired refresh token"),
        (status = 403, description = "Session revoked")
    ),
    tag = "Authentication"
)]
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<AuthResponse, AuthError> {
    services::refresh(state.pool.clone(), req, state.jwt_secret).await
}
#[utoipa::path(
    post,
    path = "/auth/signout",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Successfully signed out"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Authentication"
)]
pub async fn signout(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<(), AuthError> {
    revoke_session_by_id(state.pool.clone(), claims.sub, claims.sid).await
}
#[utoipa::path(
    post,
    path = "/auth/revoke",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Session revoked successfully"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Authentication"
)]
async fn revoke_session(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(uuid): Json<Uuid>,
) -> Result<(), AuthError> {
    revoke_session_by_id(state.pool.clone(), claims.sub, uuid).await
}

#[utoipa::path(
    post, 
    path = "/auth/admin", 
    request_body = String, 
    responses(
        (status = 200, description = "Succesfully acquired admin")
    ),
    tag = "Authentication"
)]
pub async fn admin_auth(State(state): State<Arc<AppState>>, AuthUser(claims): AuthUser, Json(code): Json<String>) -> Result<(), AuthError> {
    
    let hash = get_active_admin_pin(&state.pool.clone()).await?;
    //Means that there is not a valid hash to compare against.
    if let Some(record) = hash 
        && verify_password(
            &code,
            &record
        )? {
            
    } else {
        return Err(AuthError::InvalidCredentials); 
    }
    //Verify against the already existing code. 
    //We either use a dedicatd table for this in the database or we just go with Redis/Valkey.
    //Once they have admin perms, we disable this route to prevent anybody else from acquiring it or
    //we can make it so other people can acquire admin.
    Ok(())
}

#[utoipa::path(
    post,
    path = "/auth/revoke_all",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "All sessions revoked successfully"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Authentication"
)]
pub async fn revoke_all_sessions(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<(), AuthError> {
    let id = sqlx::query!(
        r#"
        UPDATE sessions
        SET revoked_at = NOW()
        WHERE user_id = $1
            AND revoked_at IS NULL
        RETURNING id;
        "#,
        claims.sub,
    ).fetch_one(&state.pool).await?.id;
    Ok(())
}

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(signup))
        .routes(routes!(signin))
        .routes(routes!(refresh))
        .routes(routes!(signout))
        .routes(routes!(revoke_session))
        .routes(routes!(admin_auth))
        .routes(routes!(revoke_all_sessions))
}
