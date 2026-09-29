use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use axum_client_ip::ClientIp;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
