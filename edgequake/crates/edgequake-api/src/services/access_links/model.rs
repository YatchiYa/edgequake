//! Access-link value types and pure rules.

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

/// Minutes a magic sign-in link stays valid — short: it *is* a session.
pub const MAGIC_LOGIN_TTL_MINUTES: i64 = 15;
/// Minutes a password-reset link stays valid.
pub const PASSWORD_RESET_TTL_MINUTES: i64 = 24 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPurpose {
    /// Open the link, be signed in. No password involved.
    MagicLogin,
    /// Open the link, choose a new password, be signed in.
    PasswordReset,
}

impl AccessPurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MagicLogin => "magic_login",
            Self::PasswordReset => "password_reset",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "magic_login" => Some(Self::MagicLogin),
            "password_reset" => Some(Self::PasswordReset),
            _ => None,
        }
    }

    pub fn ttl(self) -> Duration {
        Duration::minutes(match self {
            Self::MagicLogin => MAGIC_LOGIN_TTL_MINUTES,
            Self::PasswordReset => PASSWORD_RESET_TTL_MINUTES,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AccessLink {
    pub link_id: Uuid,
    pub user_id: Uuid,
    /// Organization of the admin who issued it (audit + preview copy).
    pub tenant_id: Uuid,
    pub purpose: AccessPurpose,
    pub token_hash: String,
    pub created_by: Option<Uuid>,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl AccessLink {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        purpose: AccessPurpose,
        token_hash: String,
        issuer: Option<(Uuid, String)>,
    ) -> Self {
        let now = Utc::now();
        let (created_by, created_by_name) =
            issuer.map_or((None, None), |(i, n)| (Some(i), Some(n)));
        Self {
            link_id: Uuid::new_v4(),
            user_id,
            tenant_id,
            purpose,
            token_hash,
            created_by,
            created_by_name,
            created_at: now,
            expires_at: now + purpose.ttl(),
            used_at: None,
            revoked_at: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.used_at.is_none() && self.revoked_at.is_none()
    }

    pub fn is_usable(&self, now: DateTime<Utc>) -> bool {
        self.is_open() && self.expires_at > now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(purpose: AccessPurpose) -> AccessLink {
        AccessLink::new(Uuid::new_v4(), Uuid::new_v4(), purpose, "h".into(), None)
    }

    #[test]
    fn purposes_round_trip_and_unknown_is_rejected() {
        for p in [AccessPurpose::MagicLogin, AccessPurpose::PasswordReset] {
            assert_eq!(AccessPurpose::parse(p.as_str()), Some(p));
        }
        assert_eq!(AccessPurpose::parse("admin_takeover"), None);
    }

    #[test]
    fn a_magic_link_lives_minutes_a_reset_link_a_day() {
        assert_eq!(AccessPurpose::MagicLogin.ttl(), Duration::minutes(15));
        assert_eq!(AccessPurpose::PasswordReset.ttl(), Duration::hours(24));
    }

    #[test]
    fn usability_requires_open_and_unexpired() {
        let mut l = link(AccessPurpose::MagicLogin);
        assert!(l.is_usable(Utc::now()));
        assert!(!l.is_usable(Utc::now() + Duration::minutes(16)));
        l.used_at = Some(Utc::now());
        assert!(!l.is_usable(Utc::now()));
        l.used_at = None;
        l.revoked_at = Some(Utc::now());
        assert!(!l.is_usable(Utc::now()));
    }
}
