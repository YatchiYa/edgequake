//! Role & permission catalog (SPEC-169) — the single source of truth the WebUI renders.
//!
//! Every capability row is a predicate over [`MembershipRole`], the same predicates the
//! request-authorization layer enforces (`can_write`, `can_manage_users`,
//! `can_manage_workspaces`, `can_delete_tenant`). The matrix therefore cannot drift from
//! enforcement, and the UI hard-codes no permission table (DRY).

use edgequake_auth::{RbacService, Role};
use edgequake_core::MembershipRole;
use serde::Serialize;
use utoipa::ToSchema;

/// Organization roles, lowest privilege first (matrix column order).
pub const ORGANIZATION_ROLES: [MembershipRole; 4] = [
    MembershipRole::Readonly,
    MembershipRole::Member,
    MembershipRole::Admin,
    MembershipRole::Owner,
];

/// Instance (platform-wide) roles carried in the session JWT.
const INSTANCE_ROLES: [Role; 3] = [Role::Readonly, Role::User, Role::Admin];

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OrganizationRoleInfo {
    /// Wire id (`owner`, `admin`, `member`, `readonly`).
    pub id: String,
    /// Product label (`Viewer` for `readonly`).
    pub label: String,
    pub description: String,
    /// Privilege rank (1 = lowest); a manager can only assign roles at or below their own.
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InstanceRoleInfo {
    pub id: String,
    pub label: String,
    pub description: String,
    /// Number of fine-grained RBAC permissions the role carries.
    pub permission_count: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CapabilityInfo {
    pub id: String,
    pub group: String,
    pub label: String,
    /// Organization role ids that hold this capability.
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RoleCatalog {
    pub organization_roles: Vec<OrganizationRoleInfo>,
    pub instance_roles: Vec<InstanceRoleInfo>,
    pub capabilities: Vec<CapabilityInfo>,
}

struct CapabilityDef {
    id: &'static str,
    group: &'static str,
    label: &'static str,
    granted: fn(MembershipRole) -> bool,
}

const CAPABILITIES: [CapabilityDef; 7] = [
    CapabilityDef {
        id: "knowledge.read",
        group: "Knowledge",
        label: "View documents, the knowledge graph and answers",
        granted: |_| true,
    },
    CapabilityDef {
        id: "knowledge.query",
        group: "Knowledge",
        label: "Run queries and chat with the knowledge base",
        granted: |_| true,
    },
    CapabilityDef {
        id: "knowledge.write",
        group: "Knowledge",
        label: "Upload, edit and delete documents, entities and relationships",
        granted: |r| r.can_write(),
    },
    CapabilityDef {
        id: "workspace.manage",
        group: "Organization",
        label: "Create, configure and delete workspaces",
        granted: |r| r.can_manage_workspaces(),
    },
    CapabilityDef {
        id: "members.manage",
        group: "Organization",
        label: "Invite members, change roles and remove people",
        granted: |r| r.can_manage_users(),
    },
    CapabilityDef {
        id: "members.grant_owner",
        group: "Organization",
        label: "Grant or revoke the Owner role",
        granted: |r| r == MembershipRole::Owner,
    },
    CapabilityDef {
        id: "tenant.delete",
        group: "Organization",
        label: "Delete the organization",
        granted: |r| r.can_delete_tenant(),
    },
];

/// Product label for an organization role id.
pub fn organization_role_label(role: MembershipRole) -> &'static str {
    match role {
        MembershipRole::Owner => "Owner",
        MembershipRole::Admin => "Admin",
        MembershipRole::Member => "Member",
        MembershipRole::Readonly => "Viewer",
    }
}

fn organization_role_description(role: MembershipRole) -> &'static str {
    match role {
        MembershipRole::Owner => {
            "Full control of the organization, including ownership transfer and deletion."
        }
        MembershipRole::Admin => "Manages members, roles and workspaces. Cannot grant Owner.",
        MembershipRole::Member => "Works with knowledge: uploads, edits, queries and chat.",
        MembershipRole::Readonly => "Read-only access: browse documents, graph and run queries.",
    }
}

fn instance_role_label(role: &Role) -> &'static str {
    match role {
        Role::Admin => "Instance admin",
        Role::User => "Standard",
        Role::Readonly => "Read-only",
    }
}

fn instance_role_description(role: &Role) -> &'static str {
    match role {
        Role::Admin => {
            "Platform administrator: SSO, models, quotas and every organization. Use sparingly."
        }
        Role::User => "Regular account. Access is decided by organization roles.",
        Role::Readonly => "Account-wide read-only guard, regardless of organization role.",
    }
}

/// Build the catalog (pure; cheap enough to compute per request).
pub fn build_role_catalog(rbac: &RbacService) -> RoleCatalog {
    RoleCatalog {
        organization_roles: ORGANIZATION_ROLES
            .iter()
            .rev()
            .map(|r| OrganizationRoleInfo {
                id: r.to_string(),
                label: organization_role_label(*r).to_string(),
                description: organization_role_description(*r).to_string(),
                level: r.level(),
            })
            .collect(),
        instance_roles: INSTANCE_ROLES
            .iter()
            .rev()
            .map(|r| InstanceRoleInfo {
                id: r.as_str().to_string(),
                label: instance_role_label(r).to_string(),
                description: instance_role_description(r).to_string(),
                permission_count: rbac.role_permissions(r).len(),
            })
            .collect(),
        capabilities: CAPABILITIES
            .iter()
            .map(|c| CapabilityInfo {
                id: c.id.to_string(),
                group: c.group.to_string(),
                label: c.label.to_string(),
                roles: ORGANIZATION_ROLES
                    .iter()
                    .rev()
                    .filter(|r| (c.granted)(**r))
                    .map(|r| r.to_string())
                    .collect(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> RoleCatalog {
        build_role_catalog(&RbacService::new())
    }

    fn holders(c: &RoleCatalog, id: &str) -> Vec<String> {
        c.capabilities
            .iter()
            .find(|x| x.id == id)
            .unwrap_or_else(|| panic!("capability {id}"))
            .roles
            .clone()
    }

    #[test]
    fn roles_are_listed_highest_privilege_first_with_viewer_label() {
        let c = catalog();
        let ids: Vec<_> = c.organization_roles.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["owner", "admin", "member", "readonly"]);
        assert_eq!(c.organization_roles[3].label, "Viewer");
        let levels: Vec<_> = c.organization_roles.iter().map(|r| r.level).collect();
        assert!(levels.windows(2).all(|w| w[0] > w[1]));
    }

    #[test]
    fn capability_matrix_mirrors_enforcement_predicates() {
        let c = catalog();
        assert_eq!(
            holders(&c, "knowledge.write"),
            ["owner", "admin", "member"],
            "viewers never write"
        );
        assert_eq!(holders(&c, "members.manage"), ["owner", "admin"]);
        assert_eq!(holders(&c, "workspace.manage"), ["owner", "admin"]);
        assert_eq!(holders(&c, "members.grant_owner"), ["owner"]);
        assert_eq!(holders(&c, "tenant.delete"), ["owner"]);
        assert_eq!(holders(&c, "knowledge.read").len(), 4);
    }

    #[test]
    fn every_role_in_the_matrix_exists_in_the_role_list() {
        let c = catalog();
        let known: Vec<_> = c.organization_roles.iter().map(|r| r.id.clone()).collect();
        for cap in &c.capabilities {
            assert!(cap.roles.iter().all(|r| known.contains(r)), "{}", cap.id);
        }
    }

    #[test]
    fn instance_admin_carries_the_most_permissions() {
        let c = catalog();
        assert_eq!(c.instance_roles[0].id, "admin");
        let counts: Vec<_> = c
            .instance_roles
            .iter()
            .map(|r| r.permission_count)
            .collect();
        assert!(counts[0] > counts[1] && counts[1] >= counts[2]);
    }
}
