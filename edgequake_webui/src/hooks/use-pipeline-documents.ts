"use client";

import { getDocuments } from "@/lib/api/edgequake";
import {
  scopedQueryKey,
  usePipelineWorkspace,
} from "@/lib/pipeline/pipeline-workspace-context";
import { useQuery } from "@tanstack/react-query";

export const PIPELINE_DOCUMENT_PREVIEW_SIZE = 50;

export function pipelineDocumentsQueryKey(
  tenantId: string | null,
  workspaceId: string | null,
  page: number,
  pageSize: number,
  status?: string,
) {
  return [
    ...scopedQueryKey("pipeline-documents", tenantId, workspaceId),
    page,
    pageSize,
    status ?? null,
  ] as const;
}

/**
 * Shared pipeline document query.
 *
 * Aggregate counts describe the full scoped workspace. `items` are deliberately
 * a bounded preview, and every query-function variable is represented in the
 * cache key.
 */
export function usePipelineDocuments(
  options: {
    page?: number;
    pageSize?: number;
    status?: string;
    refetchInterval?: number;
  } = {},
) {
  const { selectedTenantId, selectedWorkspaceId } = usePipelineWorkspace();
  const page = options.page ?? 1;
  const pageSize = options.pageSize ?? PIPELINE_DOCUMENT_PREVIEW_SIZE;
  const status = options.status;

  return useQuery({
    queryKey: pipelineDocumentsQueryKey(
      selectedTenantId,
      selectedWorkspaceId,
      page,
      pageSize,
      status,
    ),
    queryFn: () =>
      getDocuments({
        page,
        page_size: pageSize,
        status,
      }),
    refetchInterval: options.refetchInterval ?? 2000,
  });
}
