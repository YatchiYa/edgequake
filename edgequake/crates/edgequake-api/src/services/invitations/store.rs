//! Invitation persistence port (Dependency Inversion, mirrors the federation store).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::model::Invitation;
use crate::error::ApiError;

#[async_trait]
pub trait InvitationStore: Send + Sync {
    /// Insert; `Conflict` when the email already has an open invitation in the tenant.
    async fn create(&self, invitation: &Invitation) -> Result<(), ApiError>;

    /// Open invitations (pending or expired), oldest first.
    async fn list_open(&self, tenant_id: Uuid) -> Result<Vec<Invitation>, ApiError>;

    async fn get(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Invitation>, ApiError>;

    /// Open invitation for an accept token hash (expiry is the caller's check).
    async fn find_open_by_token(&self, token_hash: &str) -> Result<Option<Invitation>, ApiError>;

    /// Replace the token and extend the expiry of an open invitation.
    async fn rotate_token(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<bool, ApiError>;

    async fn revoke(&self, tenant_id: Uuid, id: Uuid) -> Result<bool, ApiError>;

    /// Atomic single-use claim: `Some` for exactly one caller, only while usable.
    async fn claim(&self, token_hash: &str, user_id: Uuid) -> Result<Option<Invitation>, ApiError>;

    /// Undo a claim when account creation failed afterwards, so the link works again.
    async fn release(&self, id: Uuid) -> Result<(), ApiError>;
}

pub type SharedInvitationStore = Arc<dyn InvitationStore>;
