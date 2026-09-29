// auth_repository.rs

use crate::routes::temp::models::ClientMeta;
use crate::{errors::AuthError, map_sqlx_error};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use sqlx::types::ipnetwork::IpNetwork;
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct UserInfo {
    pub id: Uuid,
    pub password_hash: String,
}
#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "system_role", rename_all = "UPPERCASE")]
pub enum SystemRole {
    User,
    Admin,
}

pub struct RefreshTokenRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub expires_at: Option<OffsetDateTime>,
    pub user_id: Uuid,
    pub revoked_at: Option<OffsetDateTime>,
}

#[derive(Clone)]
pub struct AuthRepository {
    pool: PgPool,
}
impl AuthRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // -------------------------
    // Users
    // -------------------------

    pub async fn create_user(
        &self,
        email: String,
        username: String,
        password_hash: String,
    ) -> Result<Uuid, AuthError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO users
                (email, username, password_hash)
            VALUES
                ($1, $2, $3)
            RETURNING id
            "#,
            email,
            username,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(id)
    }
    pub async fn find_user_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<UserInfo>, AuthError> {
        let user = sqlx::query_as!(
            UserInfo,
            r#"
            SELECT
                id, 
                password_hash
            FROM users
            WHERE email = $1
               OR username = $1
            "#,
            identifier
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AuthError::Database)?;

        Ok(user)
    }

    pub async fn get_user_role(&self, user_id: Uuid) -> Result<SystemRole, AuthError> {
        let role = sqlx::query_scalar!(
            r#"
            SELECT system_role as "system_role: SystemRole"
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AuthError::Database)?
        .ok_or(AuthError::InvalidCredentials)?;

        Ok(role)
    }

    // -------------------------
    // Sessions
    // -------------------------

    pub async fn create_session(
        &self,
        user_id: Uuid,
        meta: &ClientMeta,
    ) -> Result<Uuid, AuthError> {
        let session_id = sqlx::query_scalar!(
            r#"
            INSERT INTO sessions
                (
                    user_id,
                    device_name,
                    ip_address,
                    user_agent
                )
            VALUES
                ($1, $2, $3, $4)
            RETURNING id
            "#,
            user_id,
            meta.device_fingerprint,
            IpNetwork::from_str(&meta.ip_address)?,
            meta.user_agent
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(session_id)
    }

    pub async fn find_session(
        &self,
        user_id: Uuid,
        device_name: Option<String>,
        ip: &str,
        user_agent: String,
    ) -> Result<Option<Uuid>, AuthError> {
        let session = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM sessions
            WHERE user_id = $1
              AND device_name = $2
              AND ip_address = $3
              AND user_agent = $4
            "#,
            user_id,
            device_name,
            IpNetwork::from_str(ip)?,
            user_agent
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AuthError::Database)?;

        Ok(session)
    }

    pub async fn revoke_session(&self, user_id: Uuid, session_id: Uuid) -> Result<(), AuthError> {
        let mut tx = self.pool.begin().await.map_err(AuthError::Database)?;

        let result = sqlx::query!(
            r#"
            UPDATE sessions
            SET revoked_at = NOW()
            WHERE id = $1
              AND user_id = $2
              AND revoked_at IS NULL
            "#,
            session_id,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(AuthError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AuthError::SessionRevoked);
        }

        sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE session_id = $1
            "#,
            session_id
        )
        .execute(&mut *tx)
        .await
        .map_err(AuthError::Database)?;

        tx.commit().await.map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query!(
            r#"
            UPDATE sessions
            SET revoked_at = NOW()
            WHERE user_id = $1
              AND revoked_at IS NULL
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn session_exists(&self, user_id: Uuid, session_id: Uuid) -> Result<bool, AuthError> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM sessions
                WHERE id = $1
                  AND user_id = $2
                  AND revoked_at IS NULL
            )
            "#,
            session_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AuthError::Database)?
        .unwrap_or(false);

        Ok(exists)
    }

    // -------------------------
    // Refresh Tokens
    // -------------------------

    pub async fn store_refresh_token(
        &self,
        token_hash: String,
        session_id: Uuid,
    ) -> Result<(), AuthError> {
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);

        sqlx::query!(
            r#"
            INSERT INTO refresh_tokens
                (
                    token_hash,
                    session_id,
                    expires_at
                )
            VALUES
                ($1, $2, $3)
            "#,
            token_hash,
            session_id,
            expires_at
        )
        .execute(&self.pool)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn find_refresh_token(
        &self,
        token_hash: String,
    ) -> Result<Option<RefreshTokenRecord>, AuthError> {
        let record = sqlx::query_as!(
            RefreshTokenRecord,
            r#"
            SELECT
                rt.id,
                rt.session_id,
                rt.expires_at,
                s.user_id,
                s.revoked_at
            FROM refresh_tokens rt
            JOIN sessions s
                ON rt.session_id = s.id
            WHERE rt.token_hash = $1
            "#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AuthError::Database)?;

        Ok(record)
    }

    pub async fn delete_refresh_token(&self, id: Uuid) -> Result<(), AuthError> {
        sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    // -------------------------
    // Admin PIN
    // -------------------------

    pub async fn get_active_admin_pin(&self) -> Result<Option<String>, AuthError> {
        let hash = sqlx::query_scalar!(
            r#"
            SELECT code_hash
            FROM pincode
            WHERE expires_at > $1
            "#,
            OffsetDateTime::now_utc()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AuthError::Database)?;

        Ok(hash)
    }

    pub async fn replace_admin_pin(
        &self,
        hash: String,
        expires_at: OffsetDateTime,
    ) -> Result<(), AuthError> {
        let mut tx = self.pool.begin().await.map_err(AuthError::Database)?;

        sqlx::query!("TRUNCATE pincode")
            .execute(&mut *tx)
            .await
            .map_err(AuthError::Database)?;

        sqlx::query!(
            r#"
            INSERT INTO pincode
                (
                    code_hash,
                    expires_at
                )
            VALUES
                ($1, $2)
            "#,
            hash,
            expires_at
        )
        .execute(&mut *tx)
        .await
        .map_err(AuthError::Database)?;

        tx.commit().await.map_err(AuthError::Database)?;

        Ok(())
    }
}
