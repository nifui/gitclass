use crate::{
    AppState,
    errors::{ApiError, AuthError},
    map_sqlx_error,
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
use utoipa_axum::{router::OpenApiRouter, routes};

use sqlx::{Executor, PgPool, Postgres, types::ipnetwork::IpNetwork};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

pub async fn create_class() {}

pub async fn destroy_class() {}

pub async fn add_student() {}

pub async fn remove_student() {}

pub async fn assign_grade() {}
