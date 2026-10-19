//! In-memory [`AccessLinkStore`] (tests / PG-less harnesses). Process-local only.

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::model::{AccessLink, AccessPurpose};
use super::store::AccessLinkStore;
use crate::error::ApiError;

#[derive(Default)]
pub struct MemoryAccessLinkStore {
    rows: Mutex<HashMap<Uuid, AccessLink>>,
}

impl MemoryAccessLinkStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl AccessLinkStore for MemoryAccessLinkStore {
    async fn issue(&self, link: &AccessLink) -> Result<(), ApiError> {
        let mut rows = self.rows.lock().await;
        let now = Utc::now();
        for old in rows
            .values_mut()
            .filter(|l| l.user_id == link.user_id && l.purpose == link.purpose && l.is_open())
        {
            old.revoked_at = Some(now);
        }
        rows.insert(link.link_id, link.clone());
        Ok(())
    }

    async fn find_open_by_token(&self, token_hash: &str) -> Result<Option<AccessLink>, ApiError> {
        let rows = self.rows.lock().await;
        Ok(rows
            .values()
            .find(|l| l.token_hash == token_hash && l.is_open())
            .cloned())
    }

    async fn claim(&self, token_hash: &str) -> Result<Option<AccessLink>, ApiError> {
        let mut rows = self.rows.lock().await;
        let now = Utc::now();
        match rows
            .values_mut()
            .find(|l| l.token_hash == token_hash && l.is_usable(now))
        {
            Some(l) => {
                l.used_at = Some(now);
                Ok(Some(l.clone()))
            }
            None => Ok(None),
        }
    }

    async fn release(&self, link_id: Uuid) -> Result<(), ApiError> {
        if let Some(l) = self.rows.lock().await.get_mut(&link_id) {
            if l.revoked_at.is_none() {
                l.used_at = None;
            }
        }
        Ok(())
    }

    async fn revoke_for_user(
        &self,
        user_id: Uuid,
        purpose: Option<AccessPurpose>,
    ) -> Result<u64, ApiError> {
        let mut rows = self.rows.lock().await;
        let now = Utc::now();
        let mut n = 0;
        for l in rows.values_mut().filter(|l| {
            l.user_id == user_id && l.is_open() && purpose.is_none_or(|p| p == l.purpose)
        }) {
            l.revoked_at = Some(now);
            n += 1;
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(user: Uuid, purpose: AccessPurpose, hash: &str) -> AccessLink {
        AccessLink::new(user, Uuid::new_v4(), purpose, hash.into(), None)
    }

    #[tokio::test]
    async fn claim_is_single_use() {
        let s = MemoryAccessLinkStore::new();
        s.issue(&link(Uuid::new_v4(), AccessPurpose::MagicLogin, "a"))
            .await
            .unwrap();
        assert!(s.claim("a").await.unwrap().is_some());
        assert!(s.claim("a").await.unwrap().is_none());
        assert!(s.find_open_by_token("a").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn release_makes_a_claimed_link_usable_again() {
        let s = MemoryAccessLinkStore::new();
        s.issue(&link(Uuid::new_v4(), AccessPurpose::PasswordReset, "a"))
            .await
            .unwrap();
        let claimed = s.claim("a").await.unwrap().unwrap();
        s.release(claimed.link_id).await.unwrap();
        assert!(s.claim("a").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn reissuing_revokes_the_previous_link_of_that_purpose_only() {
        let s = MemoryAccessLinkStore::new();
        let u = Uuid::new_v4();
        s.issue(&link(u, AccessPurpose::MagicLogin, "m1"))
            .await
            .unwrap();
        s.issue(&link(u, AccessPurpose::PasswordReset, "r1"))
            .await
            .unwrap();
        s.issue(&link(u, AccessPurpose::MagicLogin, "m2"))
            .await
            .unwrap();
        assert!(s.claim("m1").await.unwrap().is_none(), "superseded");
        assert!(s.claim("m2").await.unwrap().is_some());
        assert!(
            s.claim("r1").await.unwrap().is_some(),
            "other purpose untouched"
        );
    }

    #[tokio::test]
    async fn revoke_for_user_kills_open_links() {
        let s = MemoryAccessLinkStore::new();
        let u = Uuid::new_v4();
        s.issue(&link(u, AccessPurpose::MagicLogin, "m"))
            .await
            .unwrap();
        s.issue(&link(u, AccessPurpose::PasswordReset, "r"))
            .await
            .unwrap();
        assert_eq!(s.revoke_for_user(u, None).await.unwrap(), 2);
        assert!(s.claim("m").await.unwrap().is_none());
    }
}
