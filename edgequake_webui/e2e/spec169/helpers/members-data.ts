/**
 * SPEC-169 fixtures: a realistic organization directory + the role catalog, shaped exactly like
 * `GET /tenants/{id}/members` and `GET /roles` (see handlers/members/types.rs).
 */
export type Role = "owner" | "admin" | "member" | "readonly";

export interface MemberFx {
  id: string;
  kind: "member" | "invitation";
  user_id: string | null;
  username: string | null;
  email: string;
  role: Role;
  workspace_id: string | null;
  status: "active" | "deactivated" | "invited" | "expired";
  instance_role: "admin" | "user" | "readonly" | null;
  sign_in: { kind: "password" | "sso"; provider_slug?: string; provider_name?: string; provider_kind?: string }[];
  role_managed_by: string | null;
  joined_at: string;
  last_login_at: string | null;
  invited_by: string | null;
  expires_at: string | null;
  is_you: boolean;
  is_last_owner: boolean;
}

export const YOU_ID = "spec155-user-001";
const ago = (h: number) => new Date(Date.now() - h * 3_600_000).toISOString();
const inDays = (d: number) => new Date(Date.now() + d * 86_400_000).toISOString();

const KEYCLOAK = { kind: "sso" as const, provider_slug: "keycloak", provider_name: "Keycloak", provider_kind: "keycloak" };
const PASSWORD = { kind: "password" as const };

function person(
  n: number,
  username: string,
  role: Role,
  extra: Partial<MemberFx> = {},
): MemberFx {
  const id = n === 1 ? YOU_ID : `00000000-0000-4000-8000-0000000000${String(n).padStart(2, "0")}`;
  return {
    id,
    kind: "member",
    user_id: id,
    username,
    email: `${username.toLowerCase().replace(/\s+/g, ".")}@acme.io`,
    role,
    workspace_id: null,
    status: "active",
    instance_role: "user",
    sign_in: [PASSWORD],
    role_managed_by: null,
    joined_at: ago(24 * (60 - n * 4)),
    last_login_at: ago(n * 7),
    invited_by: null,
    expires_at: null,
    is_you: false,
    is_last_owner: false,
    ...extra,
  };
}

function invitation(id: string, email: string, role: Role, extra: Partial<MemberFx> = {}): MemberFx {
  return {
    id,
    kind: "invitation",
    user_id: null,
    username: null,
    email,
    role,
    workspace_id: null,
    status: "invited",
    instance_role: null,
    sign_in: [],
    role_managed_by: null,
    joined_at: ago(30),
    last_login_at: null,
    invited_by: "Ada Lovelace",
    expires_at: inDays(6),
    is_you: false,
    is_last_owner: false,
    ...extra,
  };
}

export function buildDirectory(): MemberFx[] {
  return [
    person(1, "Ada Lovelace", "owner", { is_you: true, instance_role: "admin", last_login_at: ago(0.1), is_last_owner: true }),
    person(2, "Bob Martin", "admin", { sign_in: [PASSWORD, KEYCLOAK], role_managed_by: "keycloak", last_login_at: ago(3) }),
    person(3, "Carol Nguyen", "member", { sign_in: [KEYCLOAK], role_managed_by: "keycloak", last_login_at: ago(20) }),
    person(4, "Diego Alvarez", "member", { last_login_at: ago(52) }),
    person(5, "Elena Petrova", "readonly", { sign_in: [KEYCLOAK], role_managed_by: "keycloak", last_login_at: ago(240) }),
    person(6, "Farid Haddad", "member", { last_login_at: ago(24 * 9) }),
    person(7, "Grace Okafor", "admin", { last_login_at: ago(30) }),
    person(8, "Hiro Tanaka", "readonly", { status: "deactivated", last_login_at: ago(24 * 40) }),
    person(9, "Ines Moreau", "member", { last_login_at: null }),
    invitation("inv-0001", "jules.verne@acme.io", "member"),
    invitation("inv-0002", "kara.thrace@acme.io", "readonly", { status: "expired", expires_at: ago(30), joined_at: ago(24 * 10) }),
  ];
}

export const ROLE_CATALOG = {
  organization_roles: [
    { id: "owner", label: "Owner", description: "Full control of the organization, including ownership transfer and deletion.", level: 4 },
    { id: "admin", label: "Admin", description: "Manages members, roles and workspaces. Cannot grant Owner.", level: 3 },
    { id: "member", label: "Member", description: "Works with knowledge: uploads, edits, queries and chat.", level: 2 },
    { id: "readonly", label: "Viewer", description: "Read-only access: browse documents, graph and run queries.", level: 1 },
  ],
  instance_roles: [
    { id: "admin", label: "Instance admin", description: "Platform administrator: SSO, models, quotas and every organization. Use sparingly.", permission_count: 38 },
    { id: "user", label: "Standard", description: "Regular account. Access is decided by organization roles.", permission_count: 21 },
    { id: "readonly", label: "Read-only", description: "Account-wide read-only guard, regardless of organization role.", permission_count: 9 },
  ],
  capabilities: [
    { id: "knowledge.read", group: "Knowledge", label: "View documents, the knowledge graph and answers", roles: ["owner", "admin", "member", "readonly"] },
    { id: "knowledge.query", group: "Knowledge", label: "Run queries and chat with the knowledge base", roles: ["owner", "admin", "member", "readonly"] },
    { id: "knowledge.write", group: "Knowledge", label: "Upload, edit and delete documents, entities and relationships", roles: ["owner", "admin", "member"] },
    { id: "workspace.manage", group: "Organization", label: "Create, configure and delete workspaces", roles: ["owner", "admin"] },
    { id: "members.manage", group: "Organization", label: "Invite members, change roles and remove people", roles: ["owner", "admin"] },
    { id: "members.grant_owner", group: "Organization", label: "Grant or revoke the Owner role", roles: ["owner"] },
    { id: "tenant.delete", group: "Organization", label: "Delete the organization", roles: ["owner"] },
  ],
};
