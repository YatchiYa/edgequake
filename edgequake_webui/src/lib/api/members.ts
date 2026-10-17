/**
 * @module members-api
 * @description Tenant / workspace membership management (`/tenants/{id}/members`).
 */

import { api } from './client';

export const MEMBER_ROLES = ['owner', 'admin', 'member', 'readonly'] as const;
export type MemberRole = (typeof MEMBER_ROLES)[number];

export interface TenantMember {
  membership_id: string;
  user_id: string;
  username: string | null;
  email: string | null;
  role: MemberRole;
  /** null = every workspace of the tenant. */
  workspace_id: string | null;
  workspace_name: string | null;
  is_active: boolean;
  joined_at: string;
}

export interface TenantMemberList {
  items: TenantMember[];
  /** Caller may grant / change / remove `owner` memberships. */
  can_manage_owners: boolean;
}

export interface AddMemberRequest {
  /** Username, email or user id. */
  user: string;
  role: MemberRole;
  workspace_id?: string | null;
}

export const listTenantMembers = (tenantId: string) =>
  api.get<TenantMemberList>(`/tenants/${tenantId}/members`);

export const addTenantMember = (tenantId: string, body: AddMemberRequest) =>
  api.post<TenantMember>(`/tenants/${tenantId}/members`, body);

export const updateTenantMember = (tenantId: string, membershipId: string, role: MemberRole) =>
  api.patch<TenantMember>(`/tenants/${tenantId}/members/${membershipId}`, { role });

export const removeTenantMember = (tenantId: string, membershipId: string) =>
  api.delete<void>(`/tenants/${tenantId}/members/${membershipId}`);

// ── Pure rules mirrored from the backend (UI hides what the server would refuse) ──

const isTenantOwner = (m: TenantMember) => m.role === 'owner' && m.workspace_id === null;

/** Only remaining tenant-wide owner (server answers 409 on demote/remove). */
export function isLastOwner(members: TenantMember[], m: TenantMember): boolean {
  return isTenantOwner(m) && !members.some((x) => isTenantOwner(x) && x.user_id !== m.user_id);
}

/** Roles the caller may assign. */
export const assignableRoles = (canManageOwners: boolean): MemberRole[] =>
  MEMBER_ROLES.filter((r) => canManageOwners || r !== 'owner');

/** Caller may change / remove this membership at all. */
export const canEditMember = (m: TenantMember, canManageOwners: boolean): boolean =>
  canManageOwners || m.role !== 'owner';

/** `scope` = 'all' (whole tenant) or a workspace id (members with access to it). */
export function membersInScope(members: TenantMember[], scope: string): TenantMember[] {
  if (scope === 'all') return members;
  return members.filter((m) => m.workspace_id === null || m.workspace_id === scope);
}
