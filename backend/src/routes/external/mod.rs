use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

pub mod identities;
pub mod organizations;
pub mod providers;
pub mod repositories;
pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::default()
}
//Any external errors. Mainly has to deal with API calls that are made.
#[derive(Serialize, Deserialize, Debug, Error)]
pub enum ExternalError {
    //This applies to API calls that are external and not part of the backends own routes.
    #[error("Failed to execute an API call")]
    FailedAPICall,
    #[error("Failed to find the user with the given parameters")]
    InvalidUser,
    #[error("Failed to find the requested repository with the given paramters.")]
    InvalidRepository,
}

//Allow users to manually link or use OAuth
//OAuth should be easier for most people.
//The issue with manually linking is there is no concrete way to verify the account being linked
//actually belongs to the user.
pub enum ProviderAuthType {
    OAuth2,
    OAuth,
}
