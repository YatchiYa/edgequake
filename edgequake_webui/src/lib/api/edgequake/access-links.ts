/**
 * SPEC-169 — one-time sign-in / password-reset links and self-service password change.
 * Mirrors `handlers/access_links/*`.
 */

import type { LoginResponse } from "@/types";
import { api } from "../client";

export type AccessPurpose = "magic_login" | "password_reset";

export interface AccessLinkIssued {
  /** Single-use token, shown once; build `/access/{token}` from it. */
  token: string;
  purpose: AccessPurpose;
  username: string;
  email: string;
  expires_at: string;
  expires_in_minutes: number;
}

export interface AccessLinkPreview {
  purpose: AccessPurpose;
  username: string;
  email: string;
  tenant_name?: string | null;
  issued_by?: string | null;
  expires_at: string;
}

export const issueAccessLink = (tenantId: string, userId: string, purpose: AccessPurpose) =>
  api.post<AccessLinkIssued>(
    `/tenants/${encodeURIComponent(tenantId)}/members/${encodeURIComponent(userId)}/access-links`,
    { purpose },
  );

export const previewAccessLink = (token: string) =>
  api.get<AccessLinkPreview>(`/auth/access-links/${encodeURIComponent(token)}`);

/** Signs the person in; for `password_reset` also sets `password`. */
export const redeemAccessLink = (token: string, body: { password?: string } = {}) =>
  api.post<LoginResponse>(`/auth/access-links/${encodeURIComponent(token)}/redeem`, body);

/** Returns a fresh session: the device that changed the password stays signed in. */
export const changePassword = (body: { current_password: string; new_password: string }) =>
  api.post<LoginResponse>("/auth/password", body);

/** Absolute browser URL of an access link. */
export const accessUrl = (token: string, origin = window.location.origin) =>
  `${origin}/access/${token}`;
