//! PostgreSQL [`AccessLinkStore`] (migration 214). Claims are `UPDATE … RETURNING`, so two
//! replicas can never both redeem one link.

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::model::{AccessLink, AccessPurpose};
use super::store::AccessLinkStore;
use crate::error::ApiError;

pub struct PgAccessLinkStore {
    pool: PgPool,
}

impl PgAccessLinkStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn db(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| ApiError::Internal(format!("access link store ({context}): {e}"))
}

const COLS: &str = "link_id, user_id, tenant_id, purpose, token_hash, created_by, \
     created_by_name, created_at, expires_at, used_at, revoked_at";

fn from_row(row: &sqlx::postgres::PgRow) -> Result<AccessLink, sqlx::Error> {
    let purpose: String = row.try_get("purpose")?;
    Ok(AccessLink {
        link_id: row.try_get("link_id")?,
        user_id: row.try_get("user_id")?,
        tenant_id: row.try_get("tenant_id")?,
        purpose: AccessPurpose::parse(&purpose).unwrap_or(AccessPurpose::MagicLogin),
        token_hash: row.try_get("token_hash")?,
        created_by: row.try_get("created_by")?,
        created_by_name: row.try_get("created_by_name")?,
        created_at: row.try_get("created_at")?,
        expires_at: row.try_get("expires_at")?,
        used_at: row.try_get("used_at")?,
        revoked_at: row.try_get("revoked_at")?,
    })
}

fn one(
    row: Option<sqlx::postgres::PgRow>,
    ctx: &'static str,
) -> Result<Option<AccessLink>, ApiError> {
    row.as_ref().map(from_row).transpose().map_err(db(ctx))
}

#[async_trait]
impl AccessLinkStore for PgAccessLinkStore {
    async fn issue(&self, l: &AccessLink) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await.map_err(db("issue.begin"))?;
        sqlx::query(
            "UPDATE user_access_links SET revoked_at = NOW() \
             WHERE user_id = $1 AND purpose = $2 AND used_at IS NULL AND revoked_at IS NULL",
        )
        .bind(l.user_id)
        .bind(l.purpose.as_str())
        .execute(&mut *tx)
        .await
        .map_err(db("issue.revoke"))?;
        sqlx::query(
            "INSERT INTO user_access_links \
             (link_id, user_id, tenant_id, purpose, token_hash, created_by, created_by_name, \
              created_at, expires_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(l.link_id)
        .bind(l.user_id)
        .bind(l.tenant_id)
        .bind(l.purpose.as_str())
        .bind(&l.token_hash)
        .bind(l.created_by)
        .bind(&l.created_by_name)
        .bind(l.created_at)
        .bind(l.expires_at)
        .execute(&mut *tx)
        .await
        .map_err(db("issue.insert"))?;
        tx.commit().await.map_err(db("issue.commit"))
    }

    async fn find_open_by_token(&self, token_hash: &str) -> Result<Option<AccessLink>, ApiError> {
        let row = sqlx::query(&format!(
            "SELECT {COLS} FROM user_access_links \
             WHERE token_hash = $1 AND used_at IS NULL AND revoked_at IS NULL"
        ))
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(db("find_open_by_token"))?;
        one(row, "find_open_by_token")
    }

    async fn claim(&self, token_hash: &str) -> Result<Option<AccessLink>, ApiError> {
        let row = sqlx::query(&format!(
            "UPDATE user_access_links SET used_at = NOW() \
             WHERE token_hash = $1 AND used_at IS NULL AND revoked_at IS NULL \
               AND expires_at > NOW() RETURNING {COLS}"
        ))
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(db("claim"))?;
        one(row, "claim")
    }

    async fn release(&self, link_id: Uuid) -> Result<(), ApiError> {
        sqlx::query(
            "UPDATE user_access_links SET used_at = NULL WHERE link_id = $1 AND revoked_at IS NULL",
        )
        .bind(link_id)
        .execute(&self.pool)
        .await
        .map_err(db("release"))?;
        Ok(())
    }

    async fn revoke_for_user(
        &self,
        user_id: Uuid,
        purpose: Option<AccessPurpose>,
    ) -> Result<u64, ApiError> {
        let n = sqlx::query(
            "UPDATE user_access_links SET revoked_at = NOW() \
             WHERE user_id = $1 AND ($2::text IS NULL OR purpose = $2) \
               AND used_at IS NULL AND revoked_at IS NULL",
        )
        .bind(user_id)
        .bind(purpose.map(|p| p.as_str()))
        .execute(&self.pool)
        .await
        .map_err(db("revoke_for_user"))?
        .rows_affected();
        Ok(n)
    }
}
