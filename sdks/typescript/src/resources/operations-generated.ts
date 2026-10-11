/** Generated OpenAPI operations. Do not edit by hand.
 *
 * Regenerate: python3 sdks/tools/sdk_coverage.py --generate
 */
import { Resource } from "./base.js";

function encodePathParam(value: string): string {
  return String(value).split("/").map((part) => encodeURIComponent(part)).join("/");
}

function appendQuery(path: string, query?: Record<string, string>): string {
  if (!query) return path;
  const qs = new URLSearchParams(
    Object.entries(query).filter(([, value]) => value != null) as [string, string][],
  ).toString();
  return qs ? path + (path.includes("?") ? "&" : "?") + qs : path;
}

export class OperationsResource extends Resource {

  private async call(
    method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE",
    path: string,
    query?: Record<string, string>,
    body?: unknown,
    events = false,
  ): Promise<unknown> {
    if (events) {
      const chunks: unknown[] = [];
      const target = appendQuery(path, query);
      const stream = method === "GET" ? this._streamSSE<unknown>(target) : this._streamSSE<unknown>(target, body);
      for await (const event of stream) chunks.push(event);
      return chunks;
    }
    return this.transport.request({ method, path, query, body });
  }
  /** GET /.well-known/mcp/server.json */
  mcpRegistryServerJson(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/.well-known/mcp/server.json", query, body, false);
  }
  /** GET /.well-known/oauth-authorization-server */
  oauthAuthorizationServerMetadata(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/.well-known/oauth-authorization-server", query, body, false);
  }
  /** GET /.well-known/oauth-protected-resource */
  mcpOauthProtectedResource(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/.well-known/oauth-protected-resource", query, body, false);
  }
  /** GET /.well-known/oauth-protected-resource/mcp */
  mcpOauthProtectedResourcePath(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/.well-known/oauth-protected-resource/mcp", query, body, false);
  }
  /** GET /.well-known/openid-configuration */
  openidConfiguration(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/.well-known/openid-configuration", query, body, false);
  }
  /** POST /api/v1/admin/ann/warmup */
  annWarmup(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/admin/ann/warmup", query, body, false);
  }
  /** GET /api/v1/admin/entities/reconcile */
  entityReconcilePlan(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/admin/entities/reconcile", query, body, false);
  }
  /** POST /api/v1/admin/entities/reconcile */
  entityReconcileExecute(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/admin/entities/reconcile", query, body, false);
  }
  /** GET /api/v1/admin/identity-providers */
  adminListIdentityProviders(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/admin/identity-providers", query, body, false);
  }
  /** PUT /api/v1/admin/identity-providers/{slug} */
  adminUpsertIdentityProvider(slug: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PUT", "/api/v1/admin/identity-providers/" + encodePathParam(slug), query, body, false);
  }
  /** DELETE /api/v1/admin/identity-providers/{slug} */
  adminDeleteIdentityProvider(slug: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/admin/identity-providers/" + encodePathParam(slug), query, body, false);
  }
  /** GET /api/v1/admin/migration-jobs */
  listMigrationJobs(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/admin/migration-jobs", query, body, false);
  }
  /** GET /api/v1/admin/migration-jobs/{job_id} */
  getMigrationJob(jobId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/admin/migration-jobs/" + encodePathParam(jobId), query, body, false);
  }
  /** POST /api/v1/admin/migration-jobs/{job_id}/cancel */
  cancelMigrationJob(jobId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/cancel", query, body, false);
  }
  /** POST /api/v1/admin/migration-jobs/{job_id}/pause */
  pauseMigrationJob(jobId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/pause", query, body, false);
  }
  /** POST /api/v1/admin/migration-jobs/{job_id}/resume */
  resumeMigrationJob(jobId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/resume", query, body, false);
  }
  /** GET /api/v1/admin/storage/inspect */
  storageInspect(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/admin/storage/inspect", query, body, false);
  }
  /** POST /api/v1/admin/storage/repair */
  storageRepair(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/admin/storage/repair", query, body, false);
  }
  /** PATCH /api/v1/admin/tenants/{tenant_id}/quota */
  updateTenantQuota(tenantId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/admin/tenants/" + encodePathParam(tenantId) + "/quota", query, body, false);
  }
  /** DELETE /api/v1/api-keys/{key_id} */
  revokeApiKey(keyId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/api-keys/" + encodePathParam(keyId), query, body, false);
  }
  /** POST /api/v1/auth/handoff */
  redeemSsoHandoff(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/auth/handoff", query, body, false);
  }
  /** POST /api/v1/auth/oidc/backchannel-logout */
  oidcBackchannelLogout(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/auth/oidc/backchannel-logout", query, body, false);
  }
  /** GET /api/v1/auth/oidc/callback */
  oidcCallback(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/auth/oidc/callback", query, body, false);
  }
  /** GET /api/v1/auth/oidc/login */
  oidcLogin(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/auth/oidc/login", query, body, false);
  }
  /** GET /api/v1/auth/sso/providers */
  listSsoProviders(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/auth/sso/providers", query, body, false);
  }
  /** GET /api/v1/chunks/{chunk_id} */
  getChunkDetail(chunkId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/chunks/" + encodePathParam(chunkId), query, body, false);
  }
  /** GET /api/v1/chunks/{chunk_id}/lineage */
  getChunkLineage(chunkId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/chunks/" + encodePathParam(chunkId) + "/lineage", query, body, false);
  }
  /** PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback */
  setMessageFeedback(conversationId: string, messageId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/conversations/" + encodePathParam(conversationId) + "/messages/" + encodePathParam(messageId) + "/feedback", query, body, false);
  }
  /** GET /api/v1/conversations/{id} */
  getConversation(id: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/conversations/" + encodePathParam(id), query, body, false);
  }
  /** DELETE /api/v1/conversations/{id} */
  deleteConversation(id: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/conversations/" + encodePathParam(id), query, body, false);
  }
  /** PATCH /api/v1/conversations/{id} */
  updateConversation(id: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/conversations/" + encodePathParam(id), query, body, false);
  }
  /** GET /api/v1/conversations/{id}/messages */
  listMessages(id: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/conversations/" + encodePathParam(id) + "/messages", query, body, false);
  }
  /** POST /api/v1/conversations/{id}/messages */
  createMessage(id: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/conversations/" + encodePathParam(id) + "/messages", query, body, false);
  }
  /** POST /api/v1/conversations/{id}/share */
  shareConversation(id: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/conversations/" + encodePathParam(id) + "/share", query, body, false);
  }
  /** DELETE /api/v1/conversations/{id}/share */
  unshareConversation(id: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/conversations/" + encodePathParam(id) + "/share", query, body, false);
  }
  /** GET /api/v1/decision/models */
  getDecisionModels(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/decision/models", query, body, false);
  }
  /** GET /api/v1/decision/status */
  getDecisionStatus(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/decision/status", query, body, false);
  }
  /** POST /api/v1/documents/batch-delete */
  batchDeleteDocuments(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/documents/batch-delete", query, body, false);
  }
  /** GET /api/v1/documents/pdf/progress/stream/{track_id} */
  getPdfProgressStream(trackId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/pdf/progress/stream/" + encodePathParam(trackId), query, body, true);
  }
  /** GET /api/v1/documents/pdf/progress/{track_id} */
  getPdfProgress(trackId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/pdf/progress/" + encodePathParam(trackId), query, body, false);
  }
  /** GET /api/v1/documents/pdf/{pdf_id} */
  getPdfStatus(pdfId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/pdf/" + encodePathParam(pdfId), query, body, false);
  }
  /** DELETE /api/v1/documents/pdf/{pdf_id} */
  deletePdf(pdfId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/documents/pdf/" + encodePathParam(pdfId), query, body, false);
  }
  /** DELETE /api/v1/documents/pdf/{pdf_id}/cancel */
  cancelPdfProcessing(pdfId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/cancel", query, body, false);
  }
  /** GET /api/v1/documents/pdf/{pdf_id}/content */
  getPdfContent(pdfId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/content", query, body, false);
  }
  /** GET /api/v1/documents/pdf/{pdf_id}/download */
  downloadPdf(pdfId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/download", query, body, false);
  }
  /** POST /api/v1/documents/pdf/{pdf_id}/retry */
  retryPdfProcessing(pdfId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/retry", query, body, false);
  }
  /** GET /api/v1/documents/search */
  searchDocuments(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/search", query, body, false);
  }
  /** GET /api/v1/documents/track/{track_id} */
  getTrackStatus(trackId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/track/" + encodePathParam(trackId), query, body, false);
  }
  /** GET /api/v1/documents/{document_id} */
  getDocument(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId), query, body, false);
  }
  /** DELETE /api/v1/documents/{document_id} */
  deleteDocument(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/documents/" + encodePathParam(documentId), query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/assets */
  listDocumentAssets(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/assets", query, body, false);
  }
  /** POST /api/v1/documents/{document_id}/assets/include-from-pdf */
  includeDocumentAssetsFromPdf(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/assets/include-from-pdf", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/assets/{asset_id} */
  downloadDocumentAssetById(documentId: string, assetId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/assets/" + encodePathParam(assetId), query, body, false);
  }
  /** POST /api/v1/documents/{document_id}/cancel */
  cancelDocument(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/cancel", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/deletion-impact */
  analyzeDeletionImpact(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/deletion-impact", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/download/markdown */
  downloadDocumentMarkdown(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/download/markdown", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/download/original */
  downloadDocumentOriginal(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/download/original", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/failed-chunks */
  listFailedChunks(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/failed-chunks", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/lineage */
  getDocumentFullLineage(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/lineage", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/lineage/export */
  exportDocumentLineage(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/lineage/export", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/metadata */
  getDocumentMetadata(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/metadata", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/mm-assets/{asset_path} */
  downloadDocumentMmAsset(documentId: string, assetPath: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/mm-assets/" + encodePathParam(assetPath), query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/pages */
  listDocumentPages(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/pages", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/pages/health */
  getPagesHealth(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/pages/health", query, body, false);
  }
  /** POST /api/v1/documents/{document_id}/pages/reprocess */
  reprocessPages(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/pages/reprocess", query, body, false);
  }
  /** GET /api/v1/documents/{document_id}/pages/{page_number}/layout */
  getDocumentPageLayout(documentId: string, pageNumber: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/pages/" + encodePathParam(pageNumber) + "/layout", query, body, false);
  }
  /** POST /api/v1/documents/{document_id}/reanalyze */
  reanalyzeMultimodal(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/reanalyze", query, body, false);
  }
  /** POST /api/v1/documents/{document_id}/retry-chunks */
  retryFailedChunks(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/retry-chunks", query, body, false);
  }
  /** GET /api/v1/entities/{entity_id}/provenance */
  getEntityProvenance(entityId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/entities/" + encodePathParam(entityId) + "/provenance", query, body, false);
  }
  /** DELETE /api/v1/folders/{folder_id} */
  deleteFolder(folderId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/folders/" + encodePathParam(folderId), query, body, false);
  }
  /** PATCH /api/v1/folders/{folder_id} */
  updateFolder(folderId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/folders/" + encodePathParam(folderId), query, body, false);
  }
  /** GET /api/v1/graph/communities */
  getGraphCommunities(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/graph/communities", query, body, false);
  }
  /** GET /api/v1/graph/entities/{entity_name} */
  getEntity(entityName: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/graph/entities/" + encodePathParam(entityName), query, body, false);
  }
  /** PUT /api/v1/graph/entities/{entity_name} */
  updateEntity(entityName: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PUT", "/api/v1/graph/entities/" + encodePathParam(entityName), query, body, false);
  }
  /** DELETE /api/v1/graph/entities/{entity_name} */
  deleteEntity(entityName: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/graph/entities/" + encodePathParam(entityName), query, body, false);
  }
  /** GET /api/v1/graph/entities/{entity_name}/neighborhood */
  getEntityNeighborhood(entityName: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/graph/entities/" + encodePathParam(entityName) + "/neighborhood", query, body, false);
  }
  /** GET /api/v1/graph/facets */
  getGraphFacets(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/graph/facets", query, body, false);
  }
  /** GET /api/v1/graph/nodes/{node_id} */
  getNode(nodeId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/graph/nodes/" + encodePathParam(nodeId), query, body, false);
  }
  /** GET /api/v1/graph/relationships/{relationship_id} */
  getRelationship(relationshipId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query, body, false);
  }
  /** PUT /api/v1/graph/relationships/{relationship_id} */
  updateRelationship(relationshipId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PUT", "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query, body, false);
  }
  /** DELETE /api/v1/graph/relationships/{relationship_id} */
  deleteRelationship(relationshipId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query, body, false);
  }
  /** POST /api/v1/ingestion/progress */
  postIngestionProgressBatch(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/ingestion/progress", query, body, false);
  }
  /** GET /api/v1/ingestion/{track_id}/progress */
  getIngestionProgress(trackId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/ingestion/" + encodePathParam(trackId) + "/progress", query, body, false);
  }
  /** GET /api/v1/lineage/documents/{document_id} */
  getDocumentLineage(documentId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/lineage/documents/" + encodePathParam(documentId), query, body, false);
  }
  /** GET /api/v1/lineage/entities/{entity_name} */
  getEntityLineage(entityName: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/lineage/entities/" + encodePathParam(entityName), query, body, false);
  }
  /** POST /api/v1/mcp */
  mcpOpenapiV1(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/mcp", query, body, false);
  }
  /** DELETE /api/v1/messages/{message_id} */
  deleteMessage(messageId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/messages/" + encodePathParam(messageId), query, body, false);
  }
  /** PATCH /api/v1/messages/{message_id} */
  updateMessage(messageId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/messages/" + encodePathParam(messageId), query, body, false);
  }
  /** POST /api/v1/models/discover/refresh */
  refreshModelDiscovery(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/models/discover/refresh", query, body, false);
  }
  /** GET /api/v1/models/search */
  searchModels(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/models/search", query, body, false);
  }
  /** GET /api/v1/models/{provider} */
  getProvider(provider: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/models/" + encodePathParam(provider), query, body, false);
  }
  /** GET /api/v1/models/{provider}/{model} */
  getModel(provider: string, model: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/models/" + encodePathParam(provider) + "/" + encodePathParam(model), query, body, false);
  }
  /** GET /api/v1/parse/jobs/{id} */
  getParseJob(id: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/parse/jobs/" + encodePathParam(id), query, body, false);
  }
  /** GET /api/v1/pipeline/activity */
  getPipelineActivity(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/pipeline/activity", query, body, false);
  }
  /** POST /api/v1/query/context */
  retrieveQueryContext(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/query/context", query, body, false);
  }
  /** GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id} */
  getContextArtifact(artifactType: string, artifactId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/query/context/artifacts/" + encodePathParam(artifactType) + "/" + encodePathParam(artifactId), query, body, false);
  }
  /** POST /api/v1/query/context/search */
  searchQueryContext(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/query/context/search", query, body, false);
  }
  /** GET /api/v1/query/context/{retrieval_id} */
  fetchQueryContext(retrievalId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/query/context/" + encodePathParam(retrievalId), query, body, false);
  }
  /** GET /api/v1/settings/app-attribution */
  getAppAttributionSettings(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/settings/app-attribution", query, body, false);
  }
  /** PATCH /api/v1/settings/app-attribution */
  updateAppAttribution(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/settings/app-attribution", query, body, false);
  }
  /** GET /api/v1/settings/attribution */
  getAttributionSettings(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/settings/attribution", query, body, false);
  }
  /** GET /api/v1/settings/langfuse */
  getLangfuseSettings(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/settings/langfuse", query, body, false);
  }
  /** GET /api/v1/settings/llm-defaults */
  getLlmDefaults(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/settings/llm-defaults", query, body, false);
  }
  /** PATCH /api/v1/settings/llm-defaults */
  updateLlmDefaults(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/settings/llm-defaults", query, body, false);
  }
  /** POST /api/v1/setup/initialize */
  setupInitialize(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/setup/initialize", query, body, false);
  }
  /** GET /api/v1/setup/status */
  setupStatus(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/setup/status", query, body, false);
  }
  /** GET /api/v1/shared/{share_id} */
  getSharedConversation(shareId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/shared/" + encodePathParam(shareId), query, body, false);
  }
  /** GET /api/v1/tasks/{track_id} */
  getTask(trackId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/tasks/" + encodePathParam(trackId), query, body, false);
  }
  /** POST /api/v1/tasks/{track_id}/cancel */
  cancelTask(trackId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/tasks/" + encodePathParam(trackId) + "/cancel", query, body, false);
  }
  /** POST /api/v1/tasks/{track_id}/retry */
  retryTask(trackId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/tasks/" + encodePathParam(trackId) + "/retry", query, body, false);
  }
  /** GET /api/v1/tenants/{tenant_id} */
  getTenant(tenantId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/tenants/" + encodePathParam(tenantId), query, body, false);
  }
  /** PUT /api/v1/tenants/{tenant_id} */
  updateTenant(tenantId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PUT", "/api/v1/tenants/" + encodePathParam(tenantId), query, body, false);
  }
  /** DELETE /api/v1/tenants/{tenant_id} */
  deleteTenant(tenantId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/tenants/" + encodePathParam(tenantId), query, body, false);
  }
  /** GET /api/v1/tenants/{tenant_id}/workspaces */
  listWorkspaces(tenantId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/tenants/" + encodePathParam(tenantId) + "/workspaces", query, body, false);
  }
  /** POST /api/v1/tenants/{tenant_id}/workspaces */
  createWorkspace(tenantId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/tenants/" + encodePathParam(tenantId) + "/workspaces", query, body, false);
  }
  /** GET /api/v1/tenants/{tenant_id}/workspaces/by-slug/{slug} */
  getWorkspaceBySlug(tenantId: string, slug: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/tenants/" + encodePathParam(tenantId) + "/workspaces/by-slug/" + encodePathParam(slug), query, body, false);
  }
  /** GET /api/v1/users/{user_id} */
  getUser(userId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/users/" + encodePathParam(userId), query, body, false);
  }
  /** DELETE /api/v1/users/{user_id} */
  deleteUser(userId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/users/" + encodePathParam(userId), query, body, false);
  }
  /** PATCH /api/v1/users/{user_id} */
  updateUser(userId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/users/" + encodePathParam(userId), query, body, false);
  }
  /** GET /api/v1/workspaces/{workspace_id} */
  getWorkspace(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId), query, body, false);
  }
  /** PUT /api/v1/workspaces/{workspace_id} */
  updateWorkspace(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PUT", "/api/v1/workspaces/" + encodePathParam(workspaceId), query, body, false);
  }
  /** DELETE /api/v1/workspaces/{workspace_id} */
  deleteWorkspace(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/workspaces/" + encodePathParam(workspaceId), query, body, false);
  }
  /** PUT /api/v1/workspaces/{workspace_id}/injection */
  putInjection(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PUT", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injection", query, body, false);
  }
  /** PUT /api/v1/workspaces/{workspace_id}/injection/file */
  putInjectionFile(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PUT", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injection/file", query, body, false);
  }
  /** GET /api/v1/workspaces/{workspace_id}/injections */
  listInjections(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections", query, body, false);
  }
  /** GET /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
  getInjection(workspaceId: string, injectionId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query, body, false);
  }
  /** DELETE /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
  deleteInjection(workspaceId: string, injectionId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query, body, false);
  }
  /** PATCH /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
  updateInjection(workspaceId: string, injectionId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("PATCH", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query, body, false);
  }
  /** GET /api/v1/workspaces/{workspace_id}/metrics-history */
  getMetricsHistory(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/metrics-history", query, body, false);
  }
  /** POST /api/v1/workspaces/{workspace_id}/metrics-snapshot */
  triggerMetricsSnapshot(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/metrics-snapshot", query, body, false);
  }
  /** POST /api/v1/workspaces/{workspace_id}/rebuild-embeddings */
  rebuildEmbeddings(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/rebuild-embeddings", query, body, false);
  }
  /** POST /api/v1/workspaces/{workspace_id}/rebuild-knowledge-graph */
  rebuildKnowledgeGraph(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/rebuild-knowledge-graph", query, body, false);
  }
  /** POST /api/v1/workspaces/{workspace_id}/reprocess-documents */
  reprocessAllDocuments(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/reprocess-documents", query, body, false);
  }
  /** GET /api/v1/workspaces/{workspace_id}/stats */
  getWorkspaceStats(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/stats", query, body, false);
  }
  /** GET /api/v2/workspaces/{workspace_id}/jobs */
  listWorkspaceJobs(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs", query, body, false);
  }
  /** POST /api/v2/workspaces/{workspace_id}/jobs */
  createWorkspaceJob(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs", query, body, false);
  }
  /** GET /api/v2/workspaces/{workspace_id}/jobs/catalog */
  listWorkspaceJobCatalog(workspaceId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/catalog", query, body, false);
  }
  /** GET /api/v2/workspaces/{workspace_id}/jobs/{job_id} */
  getWorkspaceJob(workspaceId: string, jobId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/" + encodePathParam(jobId), query, body, false);
  }
  /** DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id} */
  cancelWorkspaceJob(workspaceId: string, jobId: string, query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("DELETE", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/" + encodePathParam(jobId), query, body, false);
  }
  /** POST /mcp */
  mcpOpenapi(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/mcp", query, body, false);
  }
  /** GET /oauth/authorize */
  oauthAuthorizeGet(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("GET", "/oauth/authorize", query, body, false);
  }
  /** POST /oauth/authorize */
  oauthAuthorizePost(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/oauth/authorize", query, body, false);
  }
  /** POST /oauth/register */
  oauthRegisterPost(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/oauth/register", query, body, false);
  }
  /** POST /oauth/revoke */
  oauthRevokePost(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/oauth/revoke", query, body, false);
  }
  /** POST /oauth/token */
  oauthTokenPost(query?: Record<string, string>, body?: unknown): Promise<unknown> {
    return this.call("POST", "/oauth/token", query, body, false);
  }
  /** GET /ws/pipeline/progress */
  wsPipelineProgress(): string {
    return this.transport.websocketUrl("/ws/pipeline/progress");
  }
  /** GET /ws/progress/{track_id} */
  wsProgressByTrackId(trackId: string): string {
    return this.transport.websocketUrl("/ws/progress/" + encodePathParam(trackId));
  }
}
