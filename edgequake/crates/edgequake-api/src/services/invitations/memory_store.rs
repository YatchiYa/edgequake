//! In-memory [`InvitationStore`] (tests / PG-less harnesses). Process-local only.

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use uuid::Uuid;

use super::model::Invitation;
use super::store::InvitationStore;
use crate::error::ApiError;

#[derive(Default)]
pub struct MemoryInvitationStore {
    rows: Mutex<HashMap<Uuid, Invitation>>,
}

impl MemoryInvitationStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl InvitationStore for MemoryInvitationStore {
    async fn create(&self, invitation: &Invitation) -> Result<(), ApiError> {
        let mut rows = self.rows.lock().await;
        let duplicate = rows.values().any(|i| {
            i.tenant_id == invitation.tenant_id && i.email == invitation.email && i.is_pending()
        });
        if duplicate {
            return Err(ApiError::Conflict("already_invited".into()));
        }
        rows.insert(invitation.invitation_id, invitation.clone());
        Ok(())
    }

    async fn list_open(&self, tenant_id: Uuid) -> Result<Vec<Invitation>, ApiError> {
        let rows = self.rows.lock().await;
        let mut open: Vec<_> = rows
            .values()
            .filter(|i| i.tenant_id == tenant_id && i.is_pending())
            .cloned()
            .collect();
        open.sort_by_key(|i| (i.created_at, i.invitation_id));
        Ok(open)
    }

    async fn get(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Invitation>, ApiError> {
        let rows = self.rows.lock().await;
        Ok(rows.get(&id).filter(|i| i.tenant_id == tenant_id).cloned())
    }

    async fn find_open_by_token(&self, token_hash: &str) -> Result<Option<Invitation>, ApiError> {
        let rows = self.rows.lock().await;
        Ok(rows
            .values()
            .find(|i| i.token_hash == token_hash && i.is_pending())
            .cloned())
    }

    async fn rotate_token(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<bool, ApiError> {
        let mut rows = self.rows.lock().await;
        match rows
            .get_mut(&id)
            .filter(|i| i.tenant_id == tenant_id && i.is_pending())
        {
            Some(i) => {
                i.token_hash = token_hash.to_string();
                i.expires_at = expires_at;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn revoke(&self, tenant_id: Uuid, id: Uuid) -> Result<bool, ApiError> {
        let mut rows = self.rows.lock().await;
        match rows
            .get_mut(&id)
            .filter(|i| i.tenant_id == tenant_id && i.is_pending())
        {
            Some(i) => {
                i.revoked_at = Some(Utc::now());
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn claim(
        &self,
        token_hash: &str,
        _user_id: Uuid,
    ) -> Result<Option<Invitation>, ApiError> {
        let mut rows = self.rows.lock().await;
        let now = Utc::now();
        match rows
            .values_mut()
            .find(|i| i.token_hash == token_hash && i.is_usable(now))
        {
            Some(i) => {
                i.accepted_at = Some(now);
                Ok(Some(i.clone()))
            }
            None => Ok(None),
        }
    }

    async fn release(&self, id: Uuid) -> Result<(), ApiError> {
        if let Some(i) = self.rows.lock().await.get_mut(&id) {
            i.accepted_at = None;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use edgequake_core::MembershipRole;

    fn invite(tenant: Uuid, email: &str, hash: &str) -> Invitation {
        Invitation::new(
            tenant,
            None,
            email,
            MembershipRole::Member,
            hash.into(),
            None,
        )
    }

    #[tokio::test]
    async fn one_open_invitation_per_email_per_tenant() {
        let store = MemoryInvitationStore::new();
        let t = Uuid::new_v4();
        store.create(&invite(t, "a@x.co", "h1")).await.unwrap();
        assert!(matches!(
            store.create(&invite(t, "A@X.co", "h2")).await,
            Err(ApiError::Conflict(_))
        ));
        store
            .create(&invite(Uuid::new_v4(), "a@x.co", "h3"))
            .await
            .unwrap();
        assert_eq!(store.list_open(t).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn revoked_invitation_frees_the_email_and_cannot_be_claimed() {
        let store = MemoryInvitationStore::new();
        let t = Uuid::new_v4();
        let i = invite(t, "a@x.co", "h1");
        store.create(&i).await.unwrap();
        assert!(store.revoke(t, i.invitation_id).await.unwrap());
        assert!(!store.revoke(t, i.invitation_id).await.unwrap());
        assert!(store.claim("h1", Uuid::new_v4()).await.unwrap().is_none());
        store.create(&invite(t, "a@x.co", "h2")).await.unwrap();
    }

    #[tokio::test]
    async fn claim_is_single_use_and_release_reopens_it() {
        let store = MemoryInvitationStore::new();
        let t = Uuid::new_v4();
        let i = invite(t, "a@x.co", "h1");
        store.create(&i).await.unwrap();
        let user = Uuid::new_v4();
        assert!(store.claim("h1", user).await.unwrap().is_some());
        assert!(store.claim("h1", user).await.unwrap().is_none(), "replay");
        assert!(store.find_open_by_token("h1").await.unwrap().is_none());
        store.release(i.invitation_id).await.unwrap();
        assert!(store.claim("h1", user).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn expired_invitations_cannot_be_claimed_but_can_be_rotated() {
        let store = MemoryInvitationStore::new();
        let t = Uuid::new_v4();
        let mut i = invite(t, "a@x.co", "h1");
        i.expires_at = Utc::now() - Duration::minutes(1);
        store.create(&i).await.unwrap();
        assert!(store.claim("h1", Uuid::new_v4()).await.unwrap().is_none());
        let fresh = Utc::now() + Duration::days(7);
        assert!(store
            .rotate_token(t, i.invitation_id, "h2", fresh)
            .await
            .unwrap());
        assert!(
            store.claim("h1", Uuid::new_v4()).await.unwrap().is_none(),
            "old token dead"
        );
        assert!(store.claim("h2", Uuid::new_v4()).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn tenant_scoping_hides_other_organizations() {
        let store = MemoryInvitationStore::new();
        let (t, other) = (Uuid::new_v4(), Uuid::new_v4());
        let i = invite(t, "a@x.co", "h1");
        store.create(&i).await.unwrap();
        assert!(store.get(other, i.invitation_id).await.unwrap().is_none());
        assert!(!store.revoke(other, i.invitation_id).await.unwrap());
    }
}
