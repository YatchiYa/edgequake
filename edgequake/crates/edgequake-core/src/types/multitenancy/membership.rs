//! Membership and role types for tenant/workspace access control.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// `metadata` key naming who manages a membership's role.
pub const MEMBERSHIP_SOURCE_KEY: &str = "source";
/// Role is re-synchronised from the identity provider on every sign-in (SPEC-158).
pub const MEMBERSHIP_SOURCE_SSO: &str = "sso";
/// Role was set by an administrator; IdP claims no longer overwrite it (SPEC-169).
pub const MEMBERSHIP_SOURCE_MANUAL: &str = "manual";
/// `metadata` key remembering the source before an administrator pinned the role.
pub const MEMBERSHIP_PREVIOUS_SOURCE_KEY: &str = "previous_source";

/// A user's membership in a tenant/workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membership {
    /// Unique membership identifier.
    pub membership_id: Uuid,
    /// User ID (from auth system).
    pub user_id: Uuid,
    /// Tenant ID.
    pub tenant_id: Uuid,
    /// Optional workspace ID (None = all workspaces in tenant).
    pub workspace_id: Option<Uuid>,
    /// Role within the tenant/workspace.
    pub role: MembershipRole,
    /// Whether the membership is active.
    pub is_active: bool,
    /// When the user joined.
    pub joined_at: chrono::DateTime<chrono::Utc>,
    /// Custom metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Membership {
    /// Tenant administration requires a tenant-wide grant; content may select a workspace.
    pub fn applies_to(&self, tenant: Uuid, workspace: Option<Uuid>) -> bool {
        self.is_active
            && self.tenant_id == tenant
            && match workspace {
                Some(workspace) => self.can_access_workspace(&workspace),
                None => self.workspace_id.is_none(),
            }
    }
    /// Create a new membership.
    pub fn new(user_id: Uuid, tenant_id: Uuid, role: MembershipRole) -> Self {
        Self {
            membership_id: Uuid::new_v4(),
            user_id,
            tenant_id,
            workspace_id: None,
            role,
            is_active: true,
            joined_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Scope membership to a specific workspace.
    pub fn for_workspace(mut self, workspace_id: Uuid) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    /// Check if user has at least the given role.
    pub fn has_role(&self, required: MembershipRole) -> bool {
        self.role.level() >= required.level()
    }

    /// Mark the membership as provisioned and role-managed by an identity provider.
    pub fn sso_managed(mut self, provider_slug: &str) -> Self {
        self.metadata.insert(
            MEMBERSHIP_SOURCE_KEY.into(),
            serde_json::json!(MEMBERSHIP_SOURCE_SSO),
        );
        self.metadata
            .insert("provider".into(), serde_json::json!(provider_slug));
        self
    }

    /// Slug of the identity provider that manages this membership, when SSO-managed.
    pub fn sso_provider(&self) -> Option<&str> {
        if !self.is_sso_managed() {
            return None;
        }
        self.metadata.get("provider").and_then(|v| v.as_str())
    }

    /// True when an IdP re-sync may rewrite this membership's role (`metadata.source == "sso"`).
    pub fn is_sso_managed(&self) -> bool {
        self.metadata
            .get(MEMBERSHIP_SOURCE_KEY)
            .and_then(|v| v.as_str())
            == Some(MEMBERSHIP_SOURCE_SSO)
    }

    /// An administrator chose this role: detach the membership from IdP role sync so the
    /// next sign-in cannot silently overwrite it. Keeps the origin in `previous_source`.
    pub fn pin_manual_role(&mut self) {
        if self.is_sso_managed() {
            self.metadata.insert(
                MEMBERSHIP_PREVIOUS_SOURCE_KEY.into(),
                serde_json::json!(MEMBERSHIP_SOURCE_SSO),
            );
            self.metadata.insert(
                MEMBERSHIP_SOURCE_KEY.into(),
                serde_json::json!(MEMBERSHIP_SOURCE_MANUAL),
            );
        }
    }

    /// Check if user can access a specific workspace.
    pub fn can_access_workspace(&self, workspace_id: &Uuid) -> bool {
        if !self.is_active {
            return false;
        }
        // None = access to all workspaces
        self.workspace_id.is_none() || self.workspace_id.as_ref() == Some(workspace_id)
    }
}

/// Roles for tenant/workspace membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MembershipRole {
    /// Read-only access.
    Readonly,
    /// Standard member (read/write).
    #[default]
    Member,
    /// Administrator (can manage users).
    Admin,
    /// Owner (full control, can delete tenant).
    Owner,
}

impl MembershipRole {
    /// Get the permission level (higher = more permissions).
    pub fn level(&self) -> u8 {
        match self {
            MembershipRole::Readonly => 1,
            MembershipRole::Member => 2,
            MembershipRole::Admin => 3,
            MembershipRole::Owner => 4,
        }
    }

    /// Check if this role can write data.
    pub fn can_write(&self) -> bool {
        matches!(
            self,
            MembershipRole::Member | MembershipRole::Admin | MembershipRole::Owner
        )
    }

    /// Check if this role can manage users.
    pub fn can_manage_users(&self) -> bool {
        matches!(self, MembershipRole::Admin | MembershipRole::Owner)
    }

    /// Check if this role can create and configure workspaces.
    pub fn can_manage_workspaces(&self) -> bool {
        matches!(self, MembershipRole::Admin | MembershipRole::Owner)
    }

    /// Check if this role can delete the tenant.
    pub fn can_delete_tenant(&self) -> bool {
        matches!(self, MembershipRole::Owner)
    }
}

impl std::fmt::Display for MembershipRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MembershipRole::Readonly => write!(f, "readonly"),
            MembershipRole::Member => write!(f, "member"),
            MembershipRole::Admin => write!(f, "admin"),
            MembershipRole::Owner => write!(f, "owner"),
        }
    }
}

impl std::str::FromStr for MembershipRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "readonly" => Ok(MembershipRole::Readonly),
            "member" => Ok(MembershipRole::Member),
            "admin" => Ok(MembershipRole::Admin),
            "owner" => Ok(MembershipRole::Owner),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn admin_assigned_role_detaches_sso_managed_membership_from_idp_sync() {
        let mut m = Membership::new(Uuid::new_v4(), Uuid::new_v4(), MembershipRole::Member)
            .sso_managed("keycloak");
        assert!(m.is_sso_managed());
        assert_eq!(m.sso_provider(), Some("keycloak"));

        m.pin_manual_role();
        assert!(!m.is_sso_managed(), "IdP sync must no longer own the role");
        assert_eq!(m.sso_provider(), None);
        assert_eq!(m.metadata["source"], "manual");
        assert_eq!(m.metadata["previous_source"], "sso");
    }

    #[test]
    fn pinning_a_manual_membership_is_a_no_op() {
        let mut m = Membership::new(Uuid::new_v4(), Uuid::new_v4(), MembershipRole::Admin);
        m.pin_manual_role();
        assert!(m.metadata.is_empty());
    }

    #[test]
    fn workspace_administration_is_an_admin_capability() {
        assert!(!MembershipRole::Member.can_manage_workspaces());
        assert!(MembershipRole::Admin.can_manage_workspaces());
        assert!(MembershipRole::Owner.can_manage_workspaces());
    }

    use super::*;

    #[test]
    fn workspace_grants_never_become_tenant_administration() {
        let tenant = Uuid::new_v4();
        let workspace = Uuid::new_v4();
        let mut membership =
            Membership::new(Uuid::new_v4(), tenant, MembershipRole::Admin).for_workspace(workspace);
        assert!(membership.applies_to(tenant, Some(workspace)));
        assert!(!membership.applies_to(tenant, None));
        assert!(!membership.applies_to(tenant, Some(Uuid::new_v4())));
        assert!(!membership.applies_to(Uuid::new_v4(), Some(workspace)));
        membership.workspace_id = None;
        assert!(membership.applies_to(tenant, None));
        assert!(membership.applies_to(tenant, Some(Uuid::new_v4())));
        membership.is_active = false;
        assert!(!membership.applies_to(tenant, None));
        assert!(!membership.applies_to(tenant, Some(workspace)));
    }
}
