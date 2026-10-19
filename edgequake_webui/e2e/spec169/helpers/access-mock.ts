/**
 * SPEC-169 hermetic mock of one-time access links, password change and the public invitation
 * pages. Contract: handlers/access_links/*.rs and handlers/members/accept.rs.
 */
import type { Page, Route } from "@playwright/test";
import { SPEC155_TENANT } from "../../spec155/helpers/mock-api";
import type { MemberFx } from "./members-data";

const json = (data: unknown, status = 200) => ({
  status,
  contentType: "application/json",
  body: JSON.stringify(data),
});

export const CURRENT_PASSWORD = "Current#Pass1";
export const STRONG_PASSWORD = "Str0ng!Passw0rd#2026";

type Purpose = "magic_login" | "password_reset";
interface LinkFx {
  purpose: Purpose;
  username: string;
  email: string;
}

export interface AccessMock {
  /** Open links by token. `magic-ok` and `reset-ok` are pre-seeded for Diego. */
  links: Map<string, LinkFx>;
  issued: { userId: string; purpose: string }[];
  redeemed: { token: string; password?: string }[];
  passwordChanges: { current_password: string; new_password: string }[];
}

const session = (username: string) => ({
  access_token: "e2e-access-token",
  token_type: "Bearer",
  expires_in: 900,
  user: { id: "spec155-user-001", username, email: `${username}@acme.io`, role: "admin" },
});

const weak = (pw: string) => pw.length < 8 || !/[A-Z]/.test(pw) || !/[0-9]/.test(pw) || /password/i.test(pw);
const WEAK_BODY = { message: "Password error: Password must contain at least 3 of: uppercase, lowercase, digit, special character" };

function preview(link: LinkFx) {
  return {
    purpose: link.purpose,
    username: link.username,
    email: link.email,
    tenant_name: "Acme Research",
    issued_by: "Ada Lovelace",
    expires_at: new Date(Date.now() + 15 * 60_000).toISOString(),
  };
}

async function handleLink(route: Route, state: AccessMock) {
  const req = route.request();
  const [, token, action] = new URL(req.url()).pathname.match(/\/auth\/access-links\/([^/]+)(?:\/(redeem))?$/) ?? [];
  if (token?.startsWith("expired")) return route.fulfill(json({ message: "access_link_expired" }, 410));
  const link = state.links.get(token);
  if (!link) return route.fulfill(json({ message: "access_link_invalid" }, 404));
  if (req.method() === "GET") return route.fulfill(json(preview(link)));
  if (action !== "redeem") return route.fallback();

  const body = req.postData() ? JSON.parse(req.postData() as string) : {};
  if (link.purpose === "password_reset") {
    if (!body.password) return route.fulfill(json({ message: "A new password is required" }, 400));
    if (weak(body.password)) return route.fulfill(json(WEAK_BODY, 400));
  }
  state.redeemed.push({ token, password: body.password });
  state.links.delete(token);
  return route.fulfill(json(session(link.username)));
}

async function handlePassword(route: Route, state: AccessMock) {
  const body = JSON.parse(route.request().postData() ?? "{}");
  if (body.current_password !== CURRENT_PASSWORD) return route.fulfill(json({ message: "current_password_incorrect" }, 400));
  if (body.new_password === body.current_password) return route.fulfill(json({ message: "The new password must differ from the current one" }, 400));
  if (weak(body.new_password)) return route.fulfill(json(WEAK_BODY, 400));
  state.passwordChanges.push(body);
  return route.fulfill(json(session("Ada Lovelace")));
}

function handleIssue(route: Route, state: AccessMock, directory: () => MemberFx[]) {
  const req = route.request();
  const userId = new URL(req.url()).pathname.match(/members\/([^/]+)\/access-links$/)?.[1] ?? "";
  const member = directory().find((m) => m.id === userId);
  if (!member) return route.fulfill(json({ message: "Member not found" }, 404));
  const { purpose } = JSON.parse(req.postData() ?? "{}") as { purpose: Purpose };
  const token = `acc_${purpose}_${state.issued.length}_AbCdEfGhIjKlMnOpQrStUvWxYz0123456789`;
  state.issued.push({ userId, purpose });
  state.links.set(token, { purpose, username: member.username ?? member.email, email: member.email });
  return route.fulfill(
    json(
      {
        token,
        purpose,
        username: member.username,
        email: member.email,
        expires_at: new Date().toISOString(),
        expires_in_minutes: purpose === "magic_login" ? 15 : 1440,
      },
      201,
    ),
  );
}

const invitation = (over: object = {}) => ({
  tenant_name: "Acme Research",
  email: "jules.verne@acme.io",
  role: "member",
  role_label: "Member",
  invited_by: "Ada Lovelace",
  expires_at: new Date(Date.now() + 6 * 86_400_000).toISOString(),
  account_exists: false,
  sso_providers: [{ slug: "keycloak", display_name: "Keycloak", kind: "keycloak", login_path: "/api/v1/auth/oidc/login?provider=keycloak" }],
  ...over,
});

async function handleInvitation(route: Route) {
  const req = route.request();
  const token = new URL(req.url()).pathname.match(/\/auth\/invitations\/([^/]+)/)?.[1] ?? "";
  if (token === "inv-expired") return route.fulfill(json({ message: "invitation_expired" }, 410));
  if (!token.startsWith("inv-")) return route.fulfill(json({ message: "invitation_invalid" }, 404));
  if (req.method() === "POST") return route.fulfill(json({ tenant_id: SPEC155_TENANT.id, tenant_name: "Acme Research", role: "member", email: "jules.verne@acme.io", username: "jules", created_account: true }, 201));
  return route.fulfill(json(invitation(token === "inv-existing" ? { account_exists: true, email: "diego.alvarez@acme.io" } : {})));
}

/** Install the access-link, password and public-invitation mocks (call after the members mock). */
export async function installAccessMock(page: Page, directory: () => MemberFx[]): Promise<AccessMock> {
  const state: AccessMock = {
    links: new Map<string, LinkFx>([
      ["magic-ok", { purpose: "magic_login", username: "Diego Alvarez", email: "diego.alvarez@acme.io" }],
      ["reset-ok", { purpose: "password_reset", username: "Diego Alvarez", email: "diego.alvarez@acme.io" }],
    ]),
    issued: [],
    redeemed: [],
    passwordChanges: [],
  };
  await page.route("**/api/v1/auth/access-links/**", (r) => handleLink(r, state));
  await page.route("**/api/v1/auth/password", (r) => handlePassword(r, state));
  await page.route("**/api/v1/auth/invitations/**", handleInvitation);
  await page.route(`**/api/v1/tenants/${SPEC155_TENANT.id}/members/*/access-links`, (r) => handleIssue(r, state, directory));
  return state;
}
