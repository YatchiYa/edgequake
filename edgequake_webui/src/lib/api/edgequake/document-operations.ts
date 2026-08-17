import { api } from "../client";
import { buildQueryString, withQuery } from "../query-params";

export interface DeleteDocumentAccepted {
  document_id: string;
  deleted: boolean;
  accepted: boolean;
  track_id?: string | null;
  chunks_deleted: number;
  entities_affected: number;
  relationships_affected: number;
  embeddings_deleted?: number;
  partial_failure?: boolean;
  partial_failure_reason?: string | null;
}

export async function deleteDocument(
  documentId: string,
): Promise<DeleteDocumentAccepted> {
  return api.delete<DeleteDocumentAccepted>(`/documents/${documentId}`);
}

export interface DeleteAllDocumentsResponse {
  accepted?: boolean;
  wipe_track_id?: string;
  deleted_count: number;
  total_chunks_deleted?: number;
  total_entities_removed?: number;
  total_relationships_removed?: number;
  total_pdfs_deleted?: number;
  skipped_count?: number;
  skipped_documents?: string[];
}

export async function deleteAllDocuments(): Promise<DeleteAllDocumentsResponse> {
  return api.delete<DeleteAllDocumentsResponse>("/documents");
}

export interface BatchDeleteDocumentsResponse {
  accepted: boolean;
  batch_track_id: string;
  planned_delete_count: number;
}

export async function batchDeleteDocuments(
  documentIds: string[],
): Promise<BatchDeleteDocumentsResponse> {
  return api.post<BatchDeleteDocumentsResponse>("/documents/batch-delete", {
    document_ids: documentIds,
  });
}

export interface DeletionImpact {
  document_id: string;
  chunks_to_delete: number;
  entities_to_remove: number;
  entities_to_update: number;
  relationships_to_remove: number;
  relationships_to_update: number;
  preview_only: boolean;
}

export async function getDeletionImpact(
  documentId: string,
): Promise<DeletionImpact> {
  return api.get<DeletionImpact>(`/documents/${documentId}/deletion-impact`);
}

export type ReprocessMode = "entities" | "full";

export interface ReprocessDocumentTaskId {
  document_id: string;
  task_id: string;
}

export interface ReprocessFailedResponse {
  track_id: string;
  failed_found: number;
  requeued: number;
  skipped?: number;
  skip_reasons?: Record<string, number>;
  document_ids: string[];
  task_id?: string | null;
  document_task_ids?: ReprocessDocumentTaskId[];
}

export async function reprocessDocument(
  documentId: string,
  force: boolean = true,
  mode: ReprocessMode = "entities",
): Promise<ReprocessFailedResponse> {
  return api.post<ReprocessFailedResponse>("/documents/reprocess", {
    document_id: documentId,
    force,
    max_documents: 1,
    mode,
  });
}

export async function scanDocuments(
  path?: string,
): Promise<{ track_id: string; message: string }> {
  return api.post<{ track_id: string; message: string }>(
    "/documents/scan",
    path ? { path } : {},
  );
}

export async function reprocessFailedDocuments(): Promise<ReprocessFailedResponse> {
  return api.post<ReprocessFailedResponse>("/documents/reprocess", {});
}

export interface RetryChunksResponse {
  document_id: string;
  chunks_queued: number;
  chunk_indices: number[];
  message: string;
  implemented: boolean;
}

export interface FailedChunkApiInfo {
  chunk_index: number;
  chunk_id: string;
  error_message: string;
  was_timeout: boolean;
  retry_attempts: number;
  status: string;
}

export interface ListFailedChunksResponse {
  document_id: string;
  failed_chunks: FailedChunkApiInfo[];
  total_chunks: number;
  successful_chunks: number;
}

export async function retryFailedChunks(
  documentId: string,
  chunkIndices: number[] = [],
  force: boolean = false,
): Promise<RetryChunksResponse> {
  return api.post<RetryChunksResponse>(
    `/documents/${documentId}/retry-chunks`,
    { chunk_indices: chunkIndices, force, max_retries: 3 },
  );
}

export async function listFailedChunks(
  documentId: string,
): Promise<ListFailedChunksResponse> {
  return api.get<ListFailedChunksResponse>(
    `/documents/${documentId}/failed-chunks`,
  );
}

export async function searchDocuments(params: {
  q?: string;
  page_size?: number;
  status?: string;
}): Promise<import("@/types").DocumentSearchResponse> {
  const query = buildQueryString({
    q: params.q,
    page_size: params.page_size ?? 20,
    status: params.status ?? "completed",
  });
  return api.get<import("@/types").DocumentSearchResponse>(
    withQuery("/documents/search", query),
  );
}
