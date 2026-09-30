use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use crate::{
    AppState,
    errors::{ApiError, AuthError},
    routes::auth::{models::Claims, repository::SystemRole, services::auth_required},
};

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
    type Rejection = Infallible;

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

pub struct AdminUser(pub Claims);
impl FromRequestParts<Arc<AppState>> for AdminUser {
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

        let claims = auth_required(&state.pool, auth_header, state.jwt_secret).await?;
        //Check permissions now.
        let role = sqlx::query!(
            r#"
            SELECT system_role as "system_role: SystemRole"
            FROM users
            WHERE id = $1
            "#,
            &claims.sub
        )
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AuthError::InvalidCredentials)?
        .system_role;
        if role == SystemRole::User {
            Err(AuthError::InvalidCredentials)
        } else {
            Ok(Self(claims))
        }
    }
}
