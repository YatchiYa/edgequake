/** SPEC-169 — map API failures on the Members screen to stable, translatable reasons. */
import { ApiRequestError } from "@/lib/api/client";

const REASONS: [RegExp, string][] = [
  [/already_member/i, "alreadyMember"],
  [/already_invited/i, "alreadyInvited"],
  [/seat_limit/i, "seatLimit"],
  [/last owner/i, "lastOwner"],
  [/last administrator/i, "lastAdmin"],
  [/own account/i, "selfSuspend"],
  [/insufficient_role/i, "insufficientRole"],
  [/instance_admin_required/i, "instanceAdminRequired"],
  [/access_link_expired/i, "linkExpired"],
  [/access_link_invalid/i, "linkInvalid"],
  [/current_password_incorrect/i, "wrongPassword"],
  [/single sign-on/i, "ssoManaged"],
  [/suspended/i, "suspended"],
  [/create a link for yourself/i, "selfLink"],
  [/invitation_expired/i, "inviteExpired"],
  [/invitation_invalid/i, "inviteInvalid"],
  [/valid email/i, "invalidEmail"],
  [/must differ/i, "samePassword"],
  [/password error|weak/i, "weakPassword"],
  [/new password is required/i, "passwordRequired"],
  [/username already/i, "usernameTaken"],
];

/** `members.errors.<key>` for a known failure, or `null` to show the raw message. */
export function memberErrorKey(error: unknown): string | null {
  if (!(error instanceof ApiRequestError)) return null;
  const haystack = `${error.message} ${error.code ?? ""} ${JSON.stringify(error.details ?? {})}`;
  const hit = REASONS.find(([re]) => re.test(haystack));
  if (hit) return hit[1];
  if (error.status === 403) return "forbidden";
  return null;
}

export function isForbidden(error: unknown): boolean {
  return error instanceof ApiRequestError && error.status === 403;
}
