/**
 * Tenants resource — multi-tenant management.
 *
 * @module resources/tenants
 * @see edgequake/crates/edgequake-api/src/handlers/tenants.rs
 */

import type {
  CreateTenantRequest,
  CreateWorkspaceRequest,
  TenantDetail,
  TenantInfo,
  UpdateTenantRequest,
  WorkspaceInfo,
} from "../types/workspaces.js";
import { Resource } from "./base.js";

const LIST_PAGE_LIMIT = 100;
const LIST_PAGE_MAX = 50;

type PagedBody<T> = { items?: T[]; total?: number };

/**
 * SPEC-141 — Exhaust offset/limit catalog pages. A raw array is treated as
 * a complete legacy payload (no further requests).
 */
async function exhaustPagedList<T>(
  fetchPage: (offset: number, limit: number) => Promise<PagedBody<T> | T[]>,
): Promise<T[]> {
  const all: T[] = [];
  let offset = 0;
  for (let i = 0; i < LIST_PAGE_MAX; i += 1) {
    const raw = await fetchPage(offset, LIST_PAGE_LIMIT);
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

export class TenantsResource extends Resource {
  /** Create a new tenant. */
  async create(request: CreateTenantRequest): Promise<TenantInfo> {
    return this._post("/api/v1/tenants", request);
  }

  /** List all tenants. */
  async list(): Promise<TenantInfo[]> {
    return exhaustPagedList((offset, limit) =>
      this._get<PagedBody<TenantInfo> | TenantInfo[]>("/api/v1/tenants", {
        limit,
        offset,
      }),
    );
  }

  /** Get a tenant by ID. */
  async get(tenantId: string): Promise<TenantDetail> {
    return this._get(`/api/v1/tenants/${tenantId}`);
  }

  /** Update a tenant. */
  async update(
    tenantId: string,
    request: UpdateTenantRequest,
  ): Promise<TenantInfo> {
    return this._put(`/api/v1/tenants/${tenantId}`, request);
  }

  /** Delete a tenant. */
  async delete(tenantId: string): Promise<void> {
    await this._del(`/api/v1/tenants/${tenantId}`);
  }

  /** Create a workspace within a tenant. */
  async createWorkspace(
    tenantId: string,
    request: CreateWorkspaceRequest,
  ): Promise<WorkspaceInfo> {
    return this._post(`/api/v1/tenants/${tenantId}/workspaces`, request);
  }

  /** List workspaces within a tenant. */
  async listWorkspaces(tenantId: string): Promise<WorkspaceInfo[]> {
    return exhaustPagedList((offset, limit) =>
      this._get<PagedBody<WorkspaceInfo> | WorkspaceInfo[]>(
        `/api/v1/tenants/${tenantId}/workspaces`,
        { limit, offset },
      ),
    );
  }

  /** Get workspace by slug within a tenant. */
  async getWorkspaceBySlug(
    tenantId: string,
    slug: string,
  ): Promise<WorkspaceInfo> {
    return this._get(`/api/v1/tenants/${tenantId}/workspaces/by-slug/${slug}`);
  }
}
