//! Pure authorization rules for member changes (SPEC-169). No I/O — unit-testable.
//!
//! * You cannot modify, or grant a role above, your own rank (instance admins outrank Owners).
//! * The last Owner can neither be demoted nor removed — an organization is never orphaned.

use edgequake_core::MembershipRole;

use crate::error::ApiError;

/// Rank of an instance admin: above every organization role.
const INSTANCE_ADMIN_RANK: u8 = 5;

/// The person performing the change.
#[derive(Debug, Clone, Copy)]
pub struct Actor {
    pub instance_admin: bool,
    pub role: Option<MembershipRole>,
}

impl Actor {
    pub fn rank(&self) -> u8 {
        if self.instance_admin {
            INSTANCE_ADMIN_RANK
        } else {
            self.role.map_or(0, |r| r.level())
        }
    }

    /// Defense in depth: the route already requires `TenantAccess::Manage`.
    pub fn can_manage(&self) -> bool {
        self.instance_admin || self.role.is_some_and(|r| r.can_manage_users())
    }

    /// Highest organization role this actor may assign.
    pub fn max_assignable(&self) -> MembershipRole {
        if self.instance_admin {
            return MembershipRole::Owner;
        }
        self.role.unwrap_or(MembershipRole::Readonly)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Denial {
    /// Target or requested role outranks the actor.
    InsufficientRank,
    /// Would leave the organization without an Owner.
    LastOwner,
}

impl From<Denial> for ApiError {
    fn from(d: Denial) -> Self {
        match d {
            Denial::InsufficientRank => ApiError::forbidden_reason("insufficient_role"),
            Denial::LastOwner => ApiError::Conflict(
                "The last owner cannot be demoted or removed. Promote another owner first.".into(),
            ),
        }
    }
}

pub fn check_assign(
    actor: &Actor,
    current: MembershipRole,
    new_role: MembershipRole,
    target_is_last_owner: bool,
) -> Result<(), Denial> {
    if !actor.can_manage() || actor.rank() < current.level() || actor.rank() < new_role.level() {
        return Err(Denial::InsufficientRank);
    }
    if target_is_last_owner && new_role != MembershipRole::Owner {
        return Err(Denial::LastOwner);
    }
    Ok(())
}

pub fn check_remove(
    actor: &Actor,
    current: MembershipRole,
    target_is_last_owner: bool,
    is_self: bool,
) -> Result<(), Denial> {
    if !is_self && (!actor.can_manage() || actor.rank() < current.level()) {
        return Err(Denial::InsufficientRank);
    }
    if target_is_last_owner {
        return Err(Denial::LastOwner);
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

    #[test]
    fn admin_cannot_grant_owner_or_touch_an_owner() {
        assert_eq!(
            check_assign(&org(Admin), Member, Owner, false),
            Err(Denial::InsufficientRank)
        );
        assert_eq!(
            check_assign(&org(Admin), Owner, Member, false),
            Err(Denial::InsufficientRank)
        );
        assert_eq!(check_assign(&org(Admin), Member, Admin, false), Ok(()));
    }

    #[test]
    fn member_and_viewer_cannot_assign_anything() {
        assert!(check_assign(&org(Member), Readonly, Member, false).is_err());
        assert!(check_assign(&org(Readonly), Readonly, Readonly, false).is_err());
        assert_eq!(org(Member).max_assignable(), Member);
    }

    #[test]
    fn owner_can_grant_owner_and_instance_admin_outranks_owner() {
        assert_eq!(check_assign(&org(Owner), Admin, Owner, false), Ok(()));
        assert_eq!(check_assign(&PLATFORM, Owner, Member, false), Ok(()));
        assert_eq!(PLATFORM.max_assignable(), Owner);
    }

    #[test]
    fn last_owner_is_protected_from_demotion_and_removal() {
        assert_eq!(
            check_assign(&org(Owner), Owner, Admin, true),
            Err(Denial::LastOwner)
        );
        assert_eq!(
            check_assign(&PLATFORM, Owner, Member, true),
            Err(Denial::LastOwner)
        );
        assert_eq!(check_assign(&org(Owner), Owner, Owner, true), Ok(()));
        assert_eq!(
            check_remove(&org(Owner), Owner, true, true),
            Err(Denial::LastOwner)
        );
    }

    #[test]
    fn removal_respects_rank_but_anyone_may_leave() {
        assert_eq!(
            check_remove(&org(Admin), Owner, false, false),
            Err(Denial::InsufficientRank)
        );
        assert_eq!(check_remove(&org(Admin), Member, false, false), Ok(()));
        assert_eq!(check_remove(&org(Member), Member, false, true), Ok(()));
    }
}
