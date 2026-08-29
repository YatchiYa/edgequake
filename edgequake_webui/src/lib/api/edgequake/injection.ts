/**
 * Domain API module — split from edgequake.ts (SPEC-017 UI-DRY-001).
 */

import { api } from "../client";
import { fetchAllPages } from "../fetch-all-pages";
import { buildQueryString, withQuery } from "../query-params";

export interface PutInjectionRequest {
  name: string;
  content: string;
}

export interface PutInjectionResponse {
  injection_id: string;
  workspace_id: string;
  version: number;
  status: string;
}

export interface InjectionSummary {
  injection_id: string;
  name: string;
  status: string;
  entity_count: number;
  source_type: string;
  error?: string;
  created_at: string;
  updated_at: string;
}

export interface ListInjectionsResponse {
  items: InjectionSummary[];
  total: number;
  has_more?: boolean;
  limit?: number;
  offset?: number;
}

export interface InjectionDetailResponse {
  injection_id: string;
  name: string;
  content: string;
  version: number;
  status: string;
  entity_count: number;
  source_type: string;
  error?: string;
  created_at: string;
  updated_at: string;
}

export interface UpdateInjectionRequest {
  name?: string;
  content?: string;
}

export interface DeleteInjectionResponse {
  deleted: boolean;
  message: string;
}

export async function putInjection(
  workspaceId: string,
  request: PutInjectionRequest,
): Promise<PutInjectionResponse> {
  return api.put<PutInjectionResponse>(
    `/workspaces/${workspaceId}/injection`,
    request,
  );
}

export async function listInjections(
  workspaceId: string,
): Promise<ListInjectionsResponse> {
  const items = await fetchAllPages(async (offset, limit) => {
    const response = await api.get<ListInjectionsResponse>(
      withQuery(
        `/workspaces/${workspaceId}/injections`,
        buildQueryString({ limit, offset }),
      ),
    );
    return {
      items: response.items ?? [],
      total:
        typeof response.total === "number"
          ? response.total
          : (response.items?.length ?? 0),
    };
  });
  return {
    items,
    total: items.length,
    has_more: false,
    offset: 0,
    limit: items.length,
  };
}

export async function getInjection(
  workspaceId: string,
  injectionId: string,
): Promise<InjectionDetailResponse> {
  return api.get<InjectionDetailResponse>(
    `/workspaces/${workspaceId}/injections/${injectionId}`,
  );
}

export async function deleteInjection(
  workspaceId: string,
  injectionId: string,
): Promise<DeleteInjectionResponse> {
  return api.delete<DeleteInjectionResponse>(
    `/workspaces/${workspaceId}/injections/${injectionId}`,
  );
}

export async function updateInjection(
  workspaceId: string,
  injectionId: string,
  request: UpdateInjectionRequest,
): Promise<PutInjectionResponse> {
  return api.patch<PutInjectionResponse>(
    `/workspaces/${workspaceId}/injections/${injectionId}`,
    request,
  );
}

/**
 * Upload a file as a knowledge injection.
 * Accepts multipart form with "file" field (txt/md/csv/json, max 10 MB).
 */
export async function putInjectionFile(
  workspaceId: string,
  name: string,
  file: File,
): Promise<PutInjectionResponse> {
  const formData = new FormData();
  formData.append('file', file, file.name);
  if (name.trim()) {
    formData.append('name', name.trim());
  }
  // WHY: Use api.put to send X-Workspace-ID and auth headers automatically (same as all other API calls)
  return api.put<PutInjectionResponse>(
    `/workspaces/${workspaceId}/injection/file`,
    formData,
  );
}
