/**
 * Keep the access token bound to the selected tenant/workspace.
 *
 * WHY: with auth on, the backend binds each JWT to one tenant/workspace and rejects a
 * request whose X-Tenant-ID / X-Workspace-ID differ from the token claims (403). Changing
 * the selector therefore re-issues the session server-side (`POST /auth/switch-scope`,
 * membership-checked) before the first request in the new scope goes out.
 */
import { getRuntimeApiBaseUrl, getRuntimeConfig } from "@/lib/runtime-config";
import { applyRestoredAccessSession } from "@/stores/use-auth-store";
import { getTenantContext, getTokens, serializeAuth } from "./client-context";

const DEFAULT_TENANT = "00000000-0000-0000-0000-000000000002";
const DEFAULT_WORKSPACE = "00000000-0000-0000-0000-000000000003";

export interface TokenScope {
  tenantId?: string;
  workspaceId?: string;
}

/** Read tenant/workspace claims from a JWT (no signature check — server is authoritative). */
export function tokenScope(token: string | null | undefined): TokenScope {
  const payload = token?.split(".")[1];
  if (!payload) return {};
  try {
    const json = atob(payload.replace(/-/g, "+").replace(/_/g, "/"));
    const claims = JSON.parse(json) as { tenant_id?: string; workspace_id?: string };
    return { tenantId: claims.tenant_id, workspaceId: claims.workspace_id };
  } catch {
    return {};
  }
}

function norm(id: string | null | undefined, fallback: string): string {
  const v = (id ?? "").trim().toLowerCase();
  return !v || v === "default" ? fallback : v;
}

/** True when the selected context needs a different token scope than `claims`. */
export function needsScopeSwitch(
  claims: TokenScope,
  tenantId: string | null | undefined,
  workspaceId: string | null | undefined,
): boolean {
  if (!claims.tenantId || !tenantId) return false;
  if (norm(claims.tenantId, DEFAULT_TENANT) !== norm(tenantId, DEFAULT_TENANT)) return true;
  if (!workspaceId) return false;
  return norm(claims.workspaceId, DEFAULT_WORKSPACE) !== norm(workspaceId, DEFAULT_WORKSPACE);
}

let inFlight: Promise<void> | null = null;
let inFlightKey = "";
/** `${token}|${scope}` that the server refused — don't retry it on every request. */
let refusedKey = "";

async function switchScope(tenantId: string, workspaceId: string | null, key: string): Promise<void> {
  const { accessToken } = getTokens();
  try {
    const response = await fetch(`${getRuntimeApiBaseUrl()}/auth/switch-scope`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${accessToken}`,
        "X-Edgequake-Client": "webui",
      },
      body: JSON.stringify({
        tenant_id: tenantId,
        ...(workspaceId ? { workspace_id: workspaceId } : {}),
      }),
    });
    if (!response.ok) {
      // Requests proceed and surface the backend's 403 (never silently read another scope).
      console.warn("[edgequake] switch-scope failed", response.status);
      if (response.status === 403) refusedKey = `${accessToken}|${key}`;
      return;
    }
    const data = (await response.json()) as { access_token: string; expires_in?: number };
    applyRestoredAccessSession({
      access_token: data.access_token,
      expires_in: data.expires_in && data.expires_in > 0 ? data.expires_in : 900,
    });
  } catch (error) {
    console.warn("[edgequake] switch-scope error", error);
  }
}

/** Await before building headers for any scoped request. No-op when auth is off. */
export async function ensureSessionScope(endpoint = ""): Promise<void> {
  if (endpoint.startsWith("/auth/")) return;
  const { authEnabled, disableDemoLogin } = getRuntimeConfig();
  if (!authEnabled && !disableDemoLogin) return;
  const { accessToken } = getTokens();
  if (!accessToken) return;
  const { tenantId, workspaceId } = getTenantContext();
  if (!tenantId || !needsScopeSwitch(tokenScope(accessToken), tenantId, workspaceId)) return;
  const key = `${tenantId}/${workspaceId ?? ""}`;
  if (refusedKey === `${accessToken}|${key}`) return;
  if (!inFlight || inFlightKey !== key) {
    inFlightKey = key;
    const run = serializeAuth(() => switchScope(tenantId, workspaceId ?? null, key)).finally(() => {
      if (inFlight === run) inFlight = null;
    });
    inFlight = run;
  }
  await inFlight;
}
