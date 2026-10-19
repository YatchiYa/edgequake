/**
 * SPEC-169 — presentation metadata for organization roles (labels live in i18n; the
 * *permissions* behind each role come from `GET /roles`, never from this file).
 */
import type { MemberStatus, OrgRole } from "@/lib/api/edgequake/members";
import type { StatusTone } from "@/components/shared/status-badge";
import { Crown, Eye, Shield, User, type LucideIcon } from "lucide-react";

/** Highest privilege first — the order shown in menus and the matrix. */
export const ORG_ROLES: readonly OrgRole[] = ["owner", "admin", "member", "readonly"];

export const ROLE_ICON: Record<OrgRole, LucideIcon> = {
  owner: Crown,
  admin: Shield,
  member: User,
  readonly: Eye,
};

/** English fallbacks; `readonly` is the product's "Viewer". */
export const ROLE_LABEL_FALLBACK: Record<OrgRole, string> = {
  owner: "Owner",
  admin: "Admin",
  member: "Member",
  readonly: "Viewer",
};

export const ROLE_LEVEL: Record<OrgRole, number> = { readonly: 1, member: 2, admin: 3, owner: 4 };

/** Roles the viewer may hand out (at or below their own rank). */
export function assignableRoles(max: OrgRole): OrgRole[] {
  return ORG_ROLES.filter((r) => ROLE_LEVEL[r] <= ROLE_LEVEL[max]);
}

export const STATUS_TONE: Record<MemberStatus, StatusTone> = {
  active: "success",
  invited: "info",
  expired: "warning",
  deactivated: "neutral",
};
