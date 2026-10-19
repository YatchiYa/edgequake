//! PostgreSQL [`InvitationStore`] (migration 213). Claims are `UPDATE … RETURNING` so two
//! replicas can never both accept one link.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use edgequake_core::MembershipRole;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::model::Invitation;
use super::store::InvitationStore;
use crate::error::ApiError;

pub struct PgInvitationStore {
    pool: PgPool,
}

impl PgInvitationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn db(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| ApiError::Internal(format!("invitation store ({context}): {e}"))
}

const COLS: &str = "invitation_id, tenant_id, workspace_id, email, role, token_hash, \
     invited_by, invited_by_name, created_at, expires_at, accepted_at, revoked_at";

fn from_row(row: &sqlx::postgres::PgRow) -> Result<Invitation, sqlx::Error> {
    let role: String = row.try_get("role")?;
    Ok(Invitation {
        invitation_id: row.try_get("invitation_id")?,
        tenant_id: row.try_get("tenant_id")?,
        workspace_id: row.try_get("workspace_id")?,
        email: row.try_get("email")?,
        role: role
            .parse::<MembershipRole>()
            .unwrap_or(MembershipRole::Readonly),
        token_hash: row.try_get("token_hash")?,
        invited_by: row.try_get("invited_by")?,
        invited_by_name: row.try_get("invited_by_name")?,
        created_at: row.try_get("created_at")?,
        expires_at: row.try_get("expires_at")?,
        accepted_at: row.try_get("accepted_at")?,
        revoked_at: row.try_get("revoked_at")?,
    })
}

fn one(
    row: Option<sqlx::postgres::PgRow>,
    ctx: &'static str,
) -> Result<Option<Invitation>, ApiError> {
    row.as_ref().map(from_row).transpose().map_err(db(ctx))
}

#[async_trait]
impl InvitationStore for PgInvitationStore {
    async fn create(&self, i: &Invitation) -> Result<(), ApiError> {
        let result = sqlx::query(
            "INSERT INTO tenant_invitations \
             (invitation_id, tenant_id, workspace_id, email, role, token_hash, \
              invited_by, invited_by_name, created_at, expires_at) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(i.invitation_id)
        .bind(i.tenant_id)
        .bind(i.workspace_id)
        .bind(&i.email)
        .bind(i.role.to_string())
        .bind(&i.token_hash)
        .bind(i.invited_by)
        .bind(&i.invited_by_name)
        .bind(i.created_at)
        .bind(i.expires_at)
        .execute(&self.pool)
        .await;
        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                Err(ApiError::Conflict("already_invited".into()))
            }
            Err(e) => Err(db("create")(e)),
        }
    }

    async fn list_open(&self, tenant_id: Uuid) -> Result<Vec<Invitation>, ApiError> {
        let rows = sqlx::query(&format!(
            "SELECT {COLS} FROM tenant_invitations \
             WHERE tenant_id = $1 AND accepted_at IS NULL AND revoked_at IS NULL \
             ORDER BY created_at, invitation_id"
        ))
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db("list_open"))?;
        rows.iter()
            .map(from_row)
            .collect::<Result<_, _>>()
            .map_err(db("list_open"))
    }

    async fn get(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Invitation>, ApiError> {
        let row = sqlx::query(&format!(
            "SELECT {COLS} FROM tenant_invitations WHERE tenant_id = $1 AND invitation_id = $2"
        ))
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db("get"))?;
        one(row, "get")
    }

    async fn find_open_by_token(&self, token_hash: &str) -> Result<Option<Invitation>, ApiError> {
        let row = sqlx::query(&format!(
            "SELECT {COLS} FROM tenant_invitations \
             WHERE token_hash = $1 AND accepted_at IS NULL AND revoked_at IS NULL"
        ))
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(db("find_open_by_token"))?;
        one(row, "find_open_by_token")
    }

    async fn rotate_token(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<bool, ApiError> {
        let n = sqlx::query(
            "UPDATE tenant_invitations SET token_hash = $3, expires_at = $4 \
             WHERE tenant_id = $1 AND invitation_id = $2 \
               AND accepted_at IS NULL AND revoked_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(db("rotate_token"))?
        .rows_affected();
        Ok(n == 1)
    }

    async fn revoke(&self, tenant_id: Uuid, id: Uuid) -> Result<bool, ApiError> {
        let n = sqlx::query(
            "UPDATE tenant_invitations SET revoked_at = NOW() \
             WHERE tenant_id = $1 AND invitation_id = $2 \
               AND accepted_at IS NULL AND revoked_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(db("revoke"))?
        .rows_affected();
        Ok(n == 1)
    }

    async fn claim(&self, token_hash: &str, user_id: Uuid) -> Result<Option<Invitation>, ApiError> {
        let row = sqlx::query(&format!(
            "UPDATE tenant_invitations SET accepted_at = NOW(), accepted_by = $2 \
             WHERE token_hash = $1 AND accepted_at IS NULL AND revoked_at IS NULL \
               AND expires_at > NOW() RETURNING {COLS}"
        ))
        .bind(token_hash)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db("claim"))?;
        one(row, "claim")
    }

    async fn release(&self, id: Uuid) -> Result<(), ApiError> {
        sqlx::query(
            "UPDATE tenant_invitations SET accepted_at = NULL, accepted_by = NULL \
             WHERE invitation_id = $1 AND revoked_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(db("release"))?;
        Ok(())
    }
}
