/**
 * SPEC-169 — UI mirror of `handlers/members/guards.rs`. The server stays authoritative; these
 * predicates only decide what to *offer* so a user is never shown an action that will 403.
 */
import type { MemberView, MemberViewer, OrgRole } from "@/lib/api/edgequake/members";
import { assignableRoles, ROLE_LEVEL } from "./role-meta";

const INSTANCE_ADMIN_RANK = 99;

export type BlockReason = "notManager" | "outranked" | "lastOwner" | "notPerson" | null;

export function viewerRank(v: MemberViewer): number {
  return v.instance_admin ? INSTANCE_ADMIN_RANK : v.role ? ROLE_LEVEL[v.role] : 0;
}

export function viewerCanManage(v: MemberViewer): boolean {
  return v.instance_admin || v.role === "owner" || v.role === "admin";
}

/** Why the viewer cannot change this person's role (null = allowed). */
export function roleChangeBlock(v: MemberViewer, m: MemberView): BlockReason {
  if (m.kind !== "member") return "notPerson";
  if (!viewerCanManage(v)) return "notManager";
  if (viewerRank(v) < ROLE_LEVEL[m.role]) return "outranked";
  if (m.is_last_owner) return "lastOwner";
  return null;
}

/** Why the viewer cannot remove this person (null = allowed; self may always leave). */
export function removeBlock(v: MemberViewer, m: MemberView): BlockReason {
  if (m.kind !== "member") return "notPerson";
  if (m.is_last_owner) return "lastOwner";
  if (m.is_you) return null;
  if (!viewerCanManage(v)) return "notManager";
  if (viewerRank(v) < ROLE_LEVEL[m.role]) return "outranked";
  return null;
}

export function roleOptions(v: MemberViewer): OrgRole[] {
  return assignableRoles(v.max_assignable_role);
}

/** Suspend / restore is an instance-level power and never applies to yourself. */
export function canSuspend(v: MemberViewer, m: MemberView): boolean {
  return v.instance_admin && m.kind === "member" && !m.is_you;
}

export type LinkBlockReason = BlockReason | "self" | "inactive" | "sso" | "platform";

/**
 * Why a sign-in / password-reset link cannot be created for this person (null = allowed).
 * Mirrors `handlers/access_links/guards.rs`: a link is a credential, so the viewer must
 * strictly outrank the target, and SSO-managed or suspended accounts are off limits.
 * (Accounts that also belong to other organizations are only knowable server-side.)
 */
export function linkBlock(v: MemberViewer, m: MemberView): LinkBlockReason | null {
  if (m.kind !== "member") return "notPerson";
  if (!viewerCanManage(v)) return "notManager";
  if (m.is_you) return "self";
  if (!v.instance_admin) {
    if (m.instance_role === "admin") return "platform";
    if (viewerRank(v) <= ROLE_LEVEL[m.role]) return "outranked";
  }
  if (m.status === "deactivated") return "inactive";
  if (m.sign_in.some((s) => s.kind === "sso")) return "sso";
  return null;
}
