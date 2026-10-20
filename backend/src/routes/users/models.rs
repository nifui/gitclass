use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::classes::models::ClassStatus;

#[derive(Serialize, Deserialize)]
pub struct ClassMeta {
    pub id: Uuid,
    pub name: String,
    pub status: ClassStatus,
    pub term: Option<String>,
}
