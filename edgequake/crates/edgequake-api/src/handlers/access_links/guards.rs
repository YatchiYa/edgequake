//! Who may mint a sign-in / reset link for whom. Pure — no I/O.
//!
//! A link *is* a credential, so the bar is higher than for a role change:
//! the actor must strictly outrank the target, and a person who also belongs to other
//! organizations (or administers the platform) can only be reached by a platform admin —
//! otherwise an org admin could take over an account that reaches beyond their organization.

use edgequake_core::MembershipRole;

use crate::error::ApiError;
use crate::handlers::members::guards::Actor;

/// Facts about the person the link would sign in as.
#[derive(Debug, Clone, Copy)]
pub struct LinkTarget {
    pub is_self: bool,
    pub is_active: bool,
    pub is_instance_admin: bool,
    /// Holds an active grant in an organization other than the actor's.
    pub in_other_organizations: bool,
    /// Signs in through an identity provider: the IdP owns the credential.
    pub sso_managed: bool,
    pub role: MembershipRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkDenial {
    NotAManager,
    SelfTarget,
    Inactive,
    SsoManaged,
    Outranked,
    PlatformAccount,
}

impl From<LinkDenial> for ApiError {
    fn from(d: LinkDenial) -> Self {
        match d {
            LinkDenial::NotAManager | LinkDenial::Outranked => {
                ApiError::forbidden_reason("insufficient_role")
            }
            LinkDenial::PlatformAccount => ApiError::forbidden_reason("instance_admin_required"),
            LinkDenial::SelfTarget => ApiError::Conflict(
                "You cannot create a link for yourself. Use Change password instead.".into(),
            ),
            LinkDenial::Inactive => ApiError::Conflict(
                "This account is suspended. Restore it before creating a link.".into(),
            ),
            LinkDenial::SsoManaged => ApiError::Conflict(
                "This person signs in through single sign-on; their identity provider manages access."
                    .into(),
            ),
        }
    }
}

pub fn check_issue(actor: &Actor, target: &LinkTarget) -> Result<(), LinkDenial> {
    if !actor.can_manage() {
        return Err(LinkDenial::NotAManager);
    }
    if target.is_self {
        return Err(LinkDenial::SelfTarget);
    }
    if !actor.instance_admin {
        if target.is_instance_admin || target.in_other_organizations {
            return Err(LinkDenial::PlatformAccount);
        }
        if actor.rank() <= target.role.level() {
            return Err(LinkDenial::Outranked);
        }
    }
    if !target.is_active {
        return Err(LinkDenial::Inactive);
    }
    if target.sso_managed {
        return Err(LinkDenial::SsoManaged);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use MembershipRole::*;

    fn org(role: MembershipRole) -> Actor {
        Actor {
            instance_admin: false,
            role: Some(role),
        }
    }

    const PLATFORM: Actor = Actor {
        instance_admin: true,
        role: None,
    };

    fn target(role: MembershipRole) -> LinkTarget {
        LinkTarget {
            is_self: false,
            is_active: true,
            is_instance_admin: false,
            in_other_organizations: false,
            sso_managed: false,
            role,
        }
    }

    #[test]
    fn admin_may_reach_members_but_not_peers_or_owners() {
        assert_eq!(check_issue(&org(Admin), &target(Member)), Ok(()));
        assert_eq!(check_issue(&org(Admin), &target(Readonly)), Ok(()));
        assert_eq!(
            check_issue(&org(Admin), &target(Admin)),
            Err(LinkDenial::Outranked)
        );
        assert_eq!(
            check_issue(&org(Admin), &target(Owner)),
            Err(LinkDenial::Outranked)
        );
        assert_eq!(check_issue(&org(Owner), &target(Admin)), Ok(()));
        assert_eq!(
            check_issue(&org(Owner), &target(Owner)),
            Err(LinkDenial::Outranked)
        );
    }

    #[test]
    fn members_and_viewers_cannot_issue() {
        assert_eq!(
            check_issue(&org(Member), &target(Readonly)),
            Err(LinkDenial::NotAManager)
        );
        assert_eq!(
            check_issue(&org(Readonly), &target(Readonly)),
            Err(LinkDenial::NotAManager)
        );
    }

    #[test]
    fn never_for_yourself_suspended_or_sso_accounts() {
        let mut t = target(Member);
        t.is_self = true;
        assert_eq!(check_issue(&org(Owner), &t), Err(LinkDenial::SelfTarget));
        let mut t = target(Member);
        t.is_active = false;
        assert_eq!(check_issue(&org(Owner), &t), Err(LinkDenial::Inactive));
        let mut t = target(Member);
        t.sso_managed = true;
        assert_eq!(check_issue(&PLATFORM, &t), Err(LinkDenial::SsoManaged));
    }

    #[test]
    fn org_admin_cannot_reach_accounts_beyond_their_organization() {
        let mut t = target(Member);
        t.in_other_organizations = true;
        assert_eq!(
            check_issue(&org(Owner), &t),
            Err(LinkDenial::PlatformAccount)
        );
        assert_eq!(check_issue(&PLATFORM, &t), Ok(()));
        let mut t = target(Member);
        t.is_instance_admin = true;
        assert_eq!(
            check_issue(&org(Owner), &t),
            Err(LinkDenial::PlatformAccount)
        );
    }
}
