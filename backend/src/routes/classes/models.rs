use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "class_memvber_role", rename_all = "UPPERCASE")]
pub enum ClassRole {
    Teacher,
    Assistant,
    Student,
}

///Basic metadata for a given class.
#[derive(Deserialize, Serialize)]
pub struct Class {
    pub id: Uuid,
    pub name: String,
    pub course_code: Option<String>,
    pub term: Option<String>,
    pub organization_id: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub archived_at: Option<OffsetDateTime>,
}
/// Information detailing a member of a given class.
#[derive(Deserialize, Serialize)]
pub struct ClassMember {
    pub role: ClassRole,
    pub user_id: Uuid,
    pub joined_at: OffsetDateTime,
    pub left_at: Option<OffsetDateTime>,
}
