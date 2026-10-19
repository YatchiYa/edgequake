/**
 * SPEC-169 hermetic mock of the Members API. Stateful (role changes, invites, removals stick), so
 * flows can be asserted end to end without a backend. Contract: handlers/members/*.rs.
 */
import type { Page, Route } from "@playwright/test";
import { SPEC155_TENANT } from "../../spec155/helpers/mock-api";
import { prepareSpec168Page } from "../../spec168/helpers/rich-mock";
import { installAccessMock, type AccessMock } from "./access-mock";
import { buildDirectory, ROLE_CATALOG, type MemberFx, type Role } from "./members-data";

const json = (data: unknown, status = 200) => ({
  status,
  contentType: "application/json",
  body: JSON.stringify(data),
});

const LEVEL: Record<Role, number> = { readonly: 1, member: 2, admin: 3, owner: 4 };

export interface MembersMockOptions {
  /** Viewer role; `null` = plain member who gets 403 on the directory. */
  viewerRole?: Role | null;
  /** Start with only the viewer in the directory. */
  alone?: boolean;
  /** Return 500 for the list. */
  fail?: boolean;
  /** Keycloak configured as an SSO provider. */
  sso?: boolean;
  maxSeats?: number | null;
}

export interface MembersMock {
  directory: MemberFx[];
  /** Every mutating request seen, in order. */
  calls: { method: string; path: string; body: unknown }[];
  /** Token returned by the next invitation. */
  lastToken: string | null;
  /** One-time links, password change and public invitation pages. */
  access: AccessMock;
}

function summarize(all: MemberFx[]) {
  const by_role: Record<string, number> = {};
  for (const m of all) by_role[m.role] = (by_role[m.role] ?? 0) + 1;
  return {
    total: all.length,
    active: all.filter((m) => m.kind === "member" && m.status === "active").length,
    deactivated: all.filter((m) => m.status === "deactivated").length,
    invited: all.filter((m) => m.kind === "invitation").length,
    by_role,
  };
}

function list(url: URL, state: MembersMock, opts: MembersMockOptions) {
  const q = (url.searchParams.get("q") ?? "").toLowerCase();
  const role = url.searchParams.get("role");
  const status = url.searchParams.get("status");
  const sort = url.searchParams.get("sort") ?? "name";
  const dir = url.searchParams.get("dir") === "desc" ? -1 : 1;
  const size = Number(url.searchParams.get("page_size") ?? 25);
  const page = Number(url.searchParams.get("page") ?? 1);

  let rows = state.directory.filter(
    (m) =>
      (!q || (m.username ?? "").toLowerCase().includes(q) || m.email.toLowerCase().includes(q)) &&
      (!role || m.role === role) &&
      (!status || m.status === status),
  );
  const key = (m: MemberFx): string | number =>
    sort === "role" ? LEVEL[m.role] : sort === "last_active" ? Date.parse(m.last_login_at ?? "1970-01-01") : sort === "joined" ? Date.parse(m.joined_at) : (m.username ?? m.email).toLowerCase();
  rows = [...rows].sort((a, b) => (key(a) < key(b) ? -dir : key(a) > key(b) ? dir : 0));
  const members = rows.slice((page - 1) * size, page * size);
  const viewerRole = opts.viewerRole ?? "owner";
  return {
    members,
    total: rows.length,
    page,
    page_size: size,
    total_pages: Math.max(1, Math.ceil(rows.length / size)),
    summary: summarize(state.directory),
    viewer: { user_id: "spec155-user-001", role: viewerRole, instance_admin: true, max_assignable_role: viewerRole === "owner" ? "owner" : viewerRole },
    seats: { used: state.directory.filter((m) => m.kind === "member").length, max: opts.maxSeats === undefined ? 25 : opts.maxSeats },
    tenant_name: "Acme Research",
  };
}

async function handleMembers(route: Route, state: MembersMock, opts: MembersMockOptions) {
  const req = route.request();
  const url = new URL(req.url());
  const method = req.method();
  const body = req.postData() ? JSON.parse(req.postData() as string) : undefined;
  const tail = url.pathname.split(`/tenants/${SPEC155_TENANT.id}/`)[1] ?? "";

  if (!/^(members|invitations)/.test(tail)) return route.fallback();
  if (method !== "GET") state.calls.push({ method, path: tail, body });

  if (opts.viewerRole === null) return route.fulfill(json({ error: "forbidden", message: "insufficient_role" }, 403));
  if (method === "GET" && tail === "members") {
    return opts.fail ? route.fulfill(json({ message: "boom" }, 500)) : route.fulfill(json(list(url, state, opts)));
  }
  const memberId = tail.match(/^members\/([^/]+)$/)?.[1];
  if (memberId && method === "PATCH") {
    const m = state.directory.find((x) => x.id === memberId);
    if (!m) return route.fulfill(json({ message: "Member not found" }, 404));
    if (body.role) {
      if (m.is_last_owner && body.role !== "owner") {
        return route.fulfill(json({ message: "The last owner cannot be demoted or removed. Promote another owner first." }, 409));
      }
      m.role = body.role;
      m.role_managed_by = null; // a manual edit pins the role (assign_membership_role)
    }
    if (body.is_active !== undefined) m.status = body.is_active ? "active" : "deactivated";
    return route.fulfill(json(m));
  }
  if (memberId && method === "DELETE") {
    state.directory = state.directory.filter((x) => x.id !== memberId);
    return route.fulfill({ status: 204 });
  }
  if (tail === "invitations" && method === "POST") {
    if (state.directory.some((m) => m.email === body.email)) return route.fulfill(json({ message: "already_member" }, 409));
    const inv: MemberFx = {
      id: `inv-${String(state.directory.length + 100)}`,
      kind: "invitation",
      user_id: null,
      username: null,
      email: body.email,
      role: body.role,
      workspace_id: null,
      status: "invited",
      instance_role: null,
      sign_in: [],
      role_managed_by: null,
      joined_at: new Date().toISOString(),
      last_login_at: null,
      invited_by: "Ada Lovelace",
      expires_at: new Date(Date.now() + 7 * 86_400_000).toISOString(),
      is_you: false,
      is_last_owner: false,
    };
    state.directory.push(inv);
    state.lastToken = `tok_${inv.id}_AbCdEfGhIjKlMnOpQrStUvWxYz0123456789`;
    return route.fulfill(json({ invitation: inv, token: state.lastToken, expires_in_days: 7 }, 201));
  }
  const invId = tail.match(/^invitations\/([^/]+)(\/resend)?$/);
  if (invId && method === "DELETE") {
    state.directory = state.directory.filter((x) => x.id !== invId[1]);
    return route.fulfill({ status: 204 });
  }
  if (invId && invId[2] && method === "POST") {
    const inv = state.directory.find((x) => x.id === invId[1]);
    if (!inv) return route.fulfill(json({ message: "Invitation not found" }, 404));
    inv.status = "invited";
    inv.expires_at = new Date(Date.now() + 7 * 86_400_000).toISOString();
    state.lastToken = `tok_${inv.id}_resent_AbCdEfGhIjKlMnOpQrStUvWxYz01234`;
    return route.fulfill(json({ invitation: inv, token: state.lastToken, expires_in_days: 7 }));
  }
  return route.fallback();
}

/** Install the Members mock on top of the SPEC-168 rich mock; returns the mutable state. */
export async function prepareMembersPage(page: Page, opts: MembersMockOptions = {}): Promise<MembersMock> {
  await prepareSpec168Page(page);
  const state: MembersMock = { directory: opts.alone ? buildDirectory().slice(0, 1) : buildDirectory(), calls: [], lastToken: null, access: undefined as unknown as AccessMock };
  await page.route(`**/api/v1/tenants/${SPEC155_TENANT.id}/**`, (r) => handleMembers(r, state, opts));
  state.access = await installAccessMock(page, () => state.directory);
  await page.route("**/api/v1/roles**", (r) => r.fulfill(json(ROLE_CATALOG)));
  const providers = opts.sso === false ? [] : [{ slug: "keycloak", display_name: "Keycloak", kind: "keycloak", login_path: "/api/v1/auth/oidc/login?provider=keycloak" }];
  await page.route("**/api/v1/auth/sso/providers**", (r) => r.fulfill(json({ providers })));
  await page.route("**/api/v1/admin/identity-providers**", (r) =>
    r.fulfill(
      json(
        providers.map((p) => ({
          slug: p.slug, kind: p.kind, display_name: p.display_name,
          issuer: "http://localhost:8081/realms/edgequake", client_id: "edgequake-api", client_secret_ref: "EDGEQUAKE_OIDC_CLIENT_SECRET",
          redirect_uri: "http://localhost:8080/api/v1/auth/oidc/callback", trust_email: true, link_policy: "verified_email",
          jit_enabled: true, role_claim: "roles", max_role: "admin", enabled: true,
        })),
      ),
    ),
  );
  return state;
}
