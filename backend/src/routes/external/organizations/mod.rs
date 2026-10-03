use sqlx::PgPool;
use uuid::Uuid;

//Handles mapping external organizations to a class.
//Unsure if this even has a use considering external identities already exists.
//
pub async fn create_organization() {}
pub async fn organization_exists(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<bool, sqlx::Error> {
    todo!()
}
