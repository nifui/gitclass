use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize, Serialize)]
enum Provider {
    Github,
    Gitlab,
    Gitea,
    Gitberg,
    Local,
    Other,
}
pub struct ExternalIdentityRequest {
    provider: Provider,
    provider_user_id: uuid::Uuid,
    username: String,
    display_name: String,
}
pub fn add_external_identity(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExternalIdentityRequest>,
) {
}
//Defines a trait for subscription to Webhooks for providers as they are not standardized.
//Does not have to be a Webhook but some sort of evevnt system.
pub trait Subscriber {
    fn subscribe();
    fn unsubscribe();
}
pub struct InternalSubscriber {}

impl Subscriber for InternalSubscriber {
    fn subscribe() {}
    fn unsubscribe() {}
}
