//! Access-link persistence port.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use super::model::{AccessLink, AccessPurpose};
use crate::error::ApiError;

#[async_trait]
pub trait AccessLinkStore: Send + Sync {
    /// Store a new link and revoke any other open link of the same user and purpose, so
    /// at most one is ever live (re-issuing invalidates what was shared before).
    async fn issue(&self, link: &AccessLink) -> Result<(), ApiError>;

    /// Open link for a token hash (expiry is the caller's check).
    async fn find_open_by_token(&self, token_hash: &str) -> Result<Option<AccessLink>, ApiError>;

    /// Atomic single-use claim: `Some` for exactly one caller, only while usable.
    async fn claim(&self, token_hash: &str) -> Result<Option<AccessLink>, ApiError>;

    /// Undo a claim when the follow-up step failed, so the link works again.
    async fn release(&self, link_id: Uuid) -> Result<(), ApiError>;

    /// Revoke every open link of a user (optionally one purpose) — after a password change.
    async fn revoke_for_user(
        &self,
        user_id: Uuid,
        purpose: Option<AccessPurpose>,
    ) -> Result<u64, ApiError>;
}

pub type SharedAccessLinkStore = Arc<dyn AccessLinkStore>;
