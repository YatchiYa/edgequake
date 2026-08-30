/**
 * SPEC-141 — Exhaust REST `{ items, total }` catalogs.
 *
 * MCP may depend on an older published SDK that still unwraps the first
 * page. Loop here so `workspace_list` and tenant bootstrap stay complete.
 */

const LIST_PAGE_LIMIT = 100;
const LIST_PAGE_MAX = 50;

type PagedBody<T> = { items?: T[]; total?: number };

function authHeaders(apiKey?: string): Record<string, string> {
  const headers: Record<string, string> = { Accept: "application/json" };
  if (apiKey) {
    headers["X-API-Key"] = apiKey;
  }
  return headers;
}

export async function exhaustPagedJson<T>(
  baseUrl: string,
  path: string,
  apiKey?: string,
): Promise<T[]> {
  const root = baseUrl.replace(/\/$/, "");
  const all: T[] = [];
  let offset = 0;
  for (let i = 0; i < LIST_PAGE_MAX; i += 1) {
    const url = `${root}${path}?limit=${LIST_PAGE_LIMIT}&offset=${offset}`;
    const res = await fetch(url, { headers: authHeaders(apiKey) });
    if (!res.ok) {
      throw new Error(`GET ${path} failed: ${res.status}`);
    }
    const raw = (await res.json()) as PagedBody<T> | T[];
    if (Array.isArray(raw)) {
      return i === 0 ? raw : [...all, ...raw];
    }
    const items = raw.items ?? [];
    all.push(...items);
    const total = typeof raw.total === "number" ? raw.total : all.length;
    if (items.length === 0 || all.length >= total || items.length < LIST_PAGE_LIMIT) {
      break;
    }
    offset += items.length;
  }
  return all;
}

export async function listAllTenants<T extends { id: string }>(
  baseUrl: string,
  apiKey?: string,
): Promise<T[]> {
  return exhaustPagedJson<T>(baseUrl, "/api/v1/tenants", apiKey);
}

export async function listAllWorkspaces<T>(
  baseUrl: string,
  tenantId: string,
  apiKey?: string,
): Promise<T[]> {
  return exhaustPagedJson<T>(
    baseUrl,
    `/api/v1/tenants/${tenantId}/workspaces`,
    apiKey,
  );
}
