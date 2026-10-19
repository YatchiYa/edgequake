//! Invitation record and lifecycle (pure).

use chrono::{DateTime, Duration, Utc};
use edgequake_core::MembershipRole;
use uuid::Uuid;

use crate::handlers::members::types::MemberStatus;

/// How long an accept link stays valid.
pub const INVITATION_TTL_DAYS: i64 = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invitation {
    pub invitation_id: Uuid,
    pub tenant_id: Uuid,
    pub workspace_id: Option<Uuid>,
    /// Always stored lower-case.
    pub email: String,
    pub role: MembershipRole,
    /// SHA-256 hex of the accept token; the token itself is never stored.
    pub token_hash: String,
    pub invited_by: Option<Uuid>,
    pub invited_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl Invitation {
    pub fn new(
        tenant_id: Uuid,
        workspace_id: Option<Uuid>,
        email: &str,
        role: MembershipRole,
        token_hash: String,
        invited_by: Option<(Uuid, String)>,
    ) -> Self {
        let now = Utc::now();
        Self {
            invitation_id: Uuid::new_v4(),
            tenant_id,
            workspace_id,
            email: normalize_email(email),
            role,
            token_hash,
            invited_by: invited_by.as_ref().map(|(id, _)| *id),
            invited_by_name: invited_by.map(|(_, name)| name),
            created_at: now,
            expires_at: now + Duration::days(INVITATION_TTL_DAYS),
            accepted_at: None,
            revoked_at: None,
        }
    }

    /// Open (neither accepted nor revoked); may still be expired.
    pub fn is_pending(&self) -> bool {
        self.accepted_at.is_none() && self.revoked_at.is_none()
    }

    pub fn is_usable(&self, now: DateTime<Utc>) -> bool {
        self.is_pending() && self.expires_at > now
    }

    pub fn status(&self, now: DateTime<Utc>) -> MemberStatus {
        if self.expires_at > now {
            MemberStatus::Invited
        } else {
            MemberStatus::Expired
        }
    }
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Pragmatic address check (one `@`, a dotted domain, no whitespace). The IdP or the
/// mail provider is the real verifier; this only rejects obvious typos early.
pub fn is_plausible_email(email: &str) -> bool {
    let e = email.trim();
    if e.len() > 254 || e.chars().any(char::is_whitespace) {
        return false;
    }
    let mut parts = e.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invite() -> Invitation {
        Invitation::new(
            Uuid::new_v4(),
            None,
            "  Dana@Acme.Example ",
            MembershipRole::Member,
            "h".into(),
            Some((Uuid::new_v4(), "alice".into())),
        )
    }

    #[test]
    fn email_is_normalised_and_invite_is_usable_for_seven_days() {
        let i = invite();
        assert_eq!(i.email, "dana@acme.example");
        assert!(i.is_usable(Utc::now()));
        assert_eq!(i.status(Utc::now()), MemberStatus::Invited);
        let later = Utc::now() + Duration::days(INVITATION_TTL_DAYS + 1);
        assert!(!i.is_usable(later));
        assert_eq!(i.status(later), MemberStatus::Expired);
    }

    #[test]
    fn revoked_and_accepted_invitations_are_not_usable() {
        let mut revoked = invite();
        revoked.revoked_at = Some(Utc::now());
        assert!(!revoked.is_usable(Utc::now()) && !revoked.is_pending());
        let mut accepted = invite();
        accepted.accepted_at = Some(Utc::now());
        assert!(!accepted.is_usable(Utc::now()));
    }

    #[test]
    fn plausible_email_rejects_obvious_typos() {
        for ok in ["a@b.co", "first.last+tag@sub.example.org"] {
            assert!(is_plausible_email(ok), "{ok}");
        }
        for bad in [
            "", "no-at", "a@b", "@b.co", "a@@b.co", "a b@c.co", "a@.co", "a@b.",
        ] {
            assert!(!is_plausible_email(bad), "{bad}");
        }
    }
}
