package io.edgequake.sdk.resources;

import io.edgequake.sdk.internal.HttpHelper;

import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Map;

/** Generated OpenAPI operations. Do not edit by hand. */
public class OperationsGenerated {
    private final HttpHelper http;
    public OperationsGenerated(HttpHelper http) { this.http = http; }

    private String call(String method, String path, Map<String, String> query, Object body, boolean events) {
        var text = http.exchange(method, appendQuery(path, query), body);
        return events ? String.join("\n", parseEvents(text)) : text;
    }

    private static String appendQuery(String path, Map<String, String> query) {
        if (query == null || query.isEmpty()) return path;
        var pairs = new ArrayList<String>();
        for (var entry : query.entrySet()) {
            if (entry.getValue() == null) continue;
            pairs.add(URLEncoder.encode(entry.getKey(), StandardCharsets.UTF_8)
                + "=" + URLEncoder.encode(entry.getValue(), StandardCharsets.UTF_8));
        }
        if (pairs.isEmpty()) return path;
        return path + (path.contains("?") ? "&" : "?") + String.join("&", pairs);
    }

    private static java.util.List<String> parseEvents(String text) {
        var events = new ArrayList<String>();
        for (var block : text.split("\n\n", -1)) {
            for (var line : block.split("\n", -1)) {
                if (!line.startsWith("data:")) continue;
                var data = line.substring(5).trim();
                if (!data.isEmpty() && !data.equals("[DONE]")) events.add(data);
            }
        }
        return events;
    }

    /** GET /.well-known/mcp/server.json */
    public String mcpRegistryServerJson(Map<String, String> query, Object body) {
        return call("GET", "/.well-known/mcp/server.json", query, body, false);
    }
    /** GET /.well-known/oauth-authorization-server */
    public String oauthAuthorizationServerMetadata(Map<String, String> query, Object body) {
        return call("GET", "/.well-known/oauth-authorization-server", query, body, false);
    }
    /** GET /.well-known/oauth-protected-resource */
    public String mcpOauthProtectedResource(Map<String, String> query, Object body) {
        return call("GET", "/.well-known/oauth-protected-resource", query, body, false);
    }
    /** GET /.well-known/oauth-protected-resource/mcp */
    public String mcpOauthProtectedResourcePath(Map<String, String> query, Object body) {
        return call("GET", "/.well-known/oauth-protected-resource/mcp", query, body, false);
    }
    /** GET /.well-known/openid-configuration */
    public String openidConfiguration(Map<String, String> query, Object body) {
        return call("GET", "/.well-known/openid-configuration", query, body, false);
    }
    /** POST /api/chat */
    public String ollamaChat(Map<String, String> query, Object body) {
        return call("POST", "/api/chat", query, body, false);
    }
    /** POST /api/generate */
    public String ollamaGenerate(Map<String, String> query, Object body) {
        return call("POST", "/api/generate", query, body, false);
    }
    /** GET /api/ps */
    public String ollamaPs(Map<String, String> query, Object body) {
        return call("GET", "/api/ps", query, body, false);
    }
    /** GET /api/tags */
    public String ollamaTags(Map<String, String> query, Object body) {
        return call("GET", "/api/tags", query, body, false);
    }
    /** POST /api/v1/admin/ann/warmup */
    public String annWarmup(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/admin/ann/warmup", query, body, false);
    }
    /** GET /api/v1/admin/config/defaults */
    public String getServerDefaults(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/admin/config/defaults", query, body, false);
    }
    /** PATCH /api/v1/admin/config/defaults */
    public String updateServerDefaults(Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/admin/config/defaults", query, body, false);
    }
    /** GET /api/v1/admin/entities/reconcile */
    public String entityReconcilePlan(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/admin/entities/reconcile", query, body, false);
    }
    /** POST /api/v1/admin/entities/reconcile */
    public String entityReconcileExecute(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/admin/entities/reconcile", query, body, false);
    }
    /** GET /api/v1/admin/identity-providers */
    public String adminListIdentityProviders(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/admin/identity-providers", query, body, false);
    }
    /** PUT /api/v1/admin/identity-providers/{slug} */
    public String adminUpsertIdentityProvider(String slug, Map<String, String> query, Object body) {
        return call("PUT", "/api/v1/admin/identity-providers/" + encodePathParam(slug), query, body, false);
    }
    /** DELETE /api/v1/admin/identity-providers/{slug} */
    public String adminDeleteIdentityProvider(String slug, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/admin/identity-providers/" + encodePathParam(slug), query, body, false);
    }
    /** GET /api/v1/admin/migration-jobs */
    public String listMigrationJobs(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/admin/migration-jobs", query, body, false);
    }
    /** GET /api/v1/admin/migration-jobs/{job_id} */
    public String getMigrationJob(String jobId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/admin/migration-jobs/" + encodePathParam(jobId), query, body, false);
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/cancel */
    public String cancelMigrationJob(String jobId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/cancel", query, body, false);
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/pause */
    public String pauseMigrationJob(String jobId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/pause", query, body, false);
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/resume */
    public String resumeMigrationJob(String jobId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/resume", query, body, false);
    }
    /** GET /api/v1/admin/storage/inspect */
    public String storageInspect(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/admin/storage/inspect", query, body, false);
    }
    /** POST /api/v1/admin/storage/repair */
    public String storageRepair(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/admin/storage/repair", query, body, false);
    }
    /** PATCH /api/v1/admin/tenants/{tenant_id}/quota */
    public String updateTenantQuota(String tenantId, Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/admin/tenants/" + encodePathParam(tenantId) + "/quota", query, body, false);
    }
    /** DELETE /api/v1/api-keys/{key_id} */
    public String revokeApiKey(String keyId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/api-keys/" + encodePathParam(keyId), query, body, false);
    }
    /** POST /api/v1/auth/handoff */
    public String redeemSsoHandoff(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/auth/handoff", query, body, false);
    }
    /** POST /api/v1/auth/logout */
    public String logout(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/auth/logout", query, body, false);
    }
    /** POST /api/v1/auth/oidc/backchannel-logout */
    public String oidcBackchannelLogout(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/auth/oidc/backchannel-logout", query, body, false);
    }
    /** GET /api/v1/auth/oidc/callback */
    public String oidcCallback(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/auth/oidc/callback", query, body, false);
    }
    /** GET /api/v1/auth/oidc/login */
    public String oidcLogin(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/auth/oidc/login", query, body, false);
    }
    /** GET /api/v1/auth/sso/providers */
    public String listSsoProviders(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/auth/sso/providers", query, body, false);
    }
    /** GET /api/v1/chunks/{chunk_id} */
    public String getChunkDetail(String chunkId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/chunks/" + encodePathParam(chunkId), query, body, false);
    }
    /** GET /api/v1/chunks/{chunk_id}/lineage */
    public String getChunkLineage(String chunkId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/chunks/" + encodePathParam(chunkId) + "/lineage", query, body, false);
    }
    /** GET /api/v1/config/effective */
    public String getEffectiveConfig(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/config/effective", query, body, false);
    }
    /** PUT /api/v1/connections/{id} */
    public String updateConnection(String id, Map<String, String> query, Object body) {
        return call("PUT", "/api/v1/connections/" + encodePathParam(id), query, body, false);
    }
    /** DELETE /api/v1/connections/{id} */
    public String deleteConnection(String id, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/connections/" + encodePathParam(id), query, body, false);
    }
    /** POST /api/v1/connections/{id}/test */
    public String testStoredConnection(String id, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/connections/" + encodePathParam(id) + "/test", query, body, false);
    }
    /** PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback */
    public String setMessageFeedback(String conversationId, String messageId, Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/conversations/" + encodePathParam(conversationId) + "/messages/" + encodePathParam(messageId) + "/feedback", query, body, false);
    }
    /** GET /api/v1/conversations/{id}/messages */
    public String listMessages(String id, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/conversations/" + encodePathParam(id) + "/messages", query, body, false);
    }
    /** POST /api/v1/conversations/{id}/messages */
    public String createMessage(String id, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/conversations/" + encodePathParam(id) + "/messages", query, body, false);
    }
    /** POST /api/v1/conversations/{id}/share */
    public String shareConversation(String id, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/conversations/" + encodePathParam(id) + "/share", query, body, false);
    }
    /** DELETE /api/v1/conversations/{id}/share */
    public String unshareConversation(String id, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/conversations/" + encodePathParam(id) + "/share", query, body, false);
    }
    /** GET /api/v1/decision/models */
    public String getDecisionModels(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/decision/models", query, body, false);
    }
    /** GET /api/v1/decision/status */
    public String getDecisionStatus(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/decision/status", query, body, false);
    }
    /** POST /api/v1/documents/batch-delete */
    public String batchDeleteDocuments(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/documents/batch-delete", query, body, false);
    }
    /** GET /api/v1/documents/pdf/progress/stream/{track_id} */
    public String getPdfProgressStream(String trackId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/pdf/progress/stream/" + encodePathParam(trackId), query, body, true);
    }
    /** GET /api/v1/documents/pdf/progress/{track_id} */
    public String getPdfProgress(String trackId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/pdf/progress/" + encodePathParam(trackId), query, body, false);
    }
    /** GET /api/v1/documents/pdf/{pdf_id} */
    public String getPdfStatus(String pdfId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/pdf/" + encodePathParam(pdfId), query, body, false);
    }
    /** DELETE /api/v1/documents/pdf/{pdf_id} */
    public String deletePdf(String pdfId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/documents/pdf/" + encodePathParam(pdfId), query, body, false);
    }
    /** DELETE /api/v1/documents/pdf/{pdf_id}/cancel */
    public String cancelPdfProcessing(String pdfId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/cancel", query, body, false);
    }
    /** GET /api/v1/documents/pdf/{pdf_id}/content */
    public String getPdfContent(String pdfId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/content", query, body, false);
    }
    /** GET /api/v1/documents/pdf/{pdf_id}/download */
    public String downloadPdf(String pdfId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/download", query, body, false);
    }
    /** POST /api/v1/documents/pdf/{pdf_id}/retry */
    public String retryPdfProcessing(String pdfId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/retry", query, body, false);
    }
    /** GET /api/v1/documents/search */
    public String searchDocuments(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/search", query, body, false);
    }
    /** GET /api/v1/documents/track/{track_id} */
    public String getTrackStatus(String trackId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/track/" + encodePathParam(trackId), query, body, false);
    }
    /** GET /api/v1/documents/{document_id} */
    public String getDocument(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId), query, body, false);
    }
    /** DELETE /api/v1/documents/{document_id} */
    public String deleteDocument(String documentId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/documents/" + encodePathParam(documentId), query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/assets */
    public String listDocumentAssets(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/assets", query, body, false);
    }
    /** POST /api/v1/documents/{document_id}/assets/include-from-pdf */
    public String includeDocumentAssetsFromPdf(String documentId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/assets/include-from-pdf", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/assets/{asset_id} */
    public String downloadDocumentAssetById(String documentId, String assetId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/assets/" + encodePathParam(assetId), query, body, false);
    }
    /** POST /api/v1/documents/{document_id}/cancel */
    public String cancelDocument(String documentId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/cancel", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/deletion-impact */
    public String analyzeDeletionImpact(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/deletion-impact", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/download/markdown */
    public String downloadDocumentMarkdown(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/download/markdown", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/download/original */
    public String downloadDocumentOriginal(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/download/original", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/failed-chunks */
    public String listFailedChunks(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/failed-chunks", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/lineage */
    public String getDocumentFullLineage(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/lineage", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/lineage/export */
    public String exportDocumentLineage(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/lineage/export", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/metadata */
    public String getDocumentMetadata(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/metadata", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/mm-assets/{asset_path} */
    public String downloadDocumentMmAsset(String documentId, String assetPath, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/mm-assets/" + encodePathParam(assetPath), query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/pages */
    public String listDocumentPages(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/pages", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/pages/health */
    public String getPagesHealth(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/pages/health", query, body, false);
    }
    /** POST /api/v1/documents/{document_id}/pages/reprocess */
    public String reprocessPages(String documentId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/pages/reprocess", query, body, false);
    }
    /** GET /api/v1/documents/{document_id}/pages/{page_number}/layout */
    public String getDocumentPageLayout(String documentId, String pageNumber, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/documents/" + encodePathParam(documentId) + "/pages/" + encodePathParam(pageNumber) + "/layout", query, body, false);
    }
    /** POST /api/v1/documents/{document_id}/reanalyze */
    public String reanalyzeMultimodal(String documentId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/reanalyze", query, body, false);
    }
    /** POST /api/v1/documents/{document_id}/retry-chunks */
    public String retryFailedChunks(String documentId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/documents/" + encodePathParam(documentId) + "/retry-chunks", query, body, false);
    }
    /** GET /api/v1/entities/{entity_id}/provenance */
    public String getEntityProvenance(String entityId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/entities/" + encodePathParam(entityId) + "/provenance", query, body, false);
    }
    /** DELETE /api/v1/folders/{folder_id} */
    public String deleteFolder(String folderId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/folders/" + encodePathParam(folderId), query, body, false);
    }
    /** PATCH /api/v1/folders/{folder_id} */
    public String updateFolder(String folderId, Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/folders/" + encodePathParam(folderId), query, body, false);
    }
    /** GET /api/v1/graph/communities */
    public String getGraphCommunities(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/graph/communities", query, body, false);
    }
    /** GET /api/v1/graph/entities/{entity_name} */
    public String getEntity(String entityName, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/graph/entities/" + encodePathParam(entityName), query, body, false);
    }
    /** PUT /api/v1/graph/entities/{entity_name} */
    public String updateEntity(String entityName, Map<String, String> query, Object body) {
        return call("PUT", "/api/v1/graph/entities/" + encodePathParam(entityName), query, body, false);
    }
    /** DELETE /api/v1/graph/entities/{entity_name} */
    public String deleteEntity(String entityName, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/graph/entities/" + encodePathParam(entityName), query, body, false);
    }
    /** GET /api/v1/graph/entities/{entity_name}/neighborhood */
    public String getEntityNeighborhood(String entityName, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/graph/entities/" + encodePathParam(entityName) + "/neighborhood", query, body, false);
    }
    /** GET /api/v1/graph/facets */
    public String getGraphFacets(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/graph/facets", query, body, false);
    }
    /** GET /api/v1/graph/nodes/{node_id} */
    public String getNode(String nodeId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/graph/nodes/" + encodePathParam(nodeId), query, body, false);
    }
    /** GET /api/v1/graph/relationships/{relationship_id} */
    public String getRelationship(String relationshipId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query, body, false);
    }
    /** PUT /api/v1/graph/relationships/{relationship_id} */
    public String updateRelationship(String relationshipId, Map<String, String> query, Object body) {
        return call("PUT", "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query, body, false);
    }
    /** DELETE /api/v1/graph/relationships/{relationship_id} */
    public String deleteRelationship(String relationshipId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query, body, false);
    }
    /** GET /api/v1/graph/stream */
    public String streamGraph(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/graph/stream", query, body, true);
    }
    /** POST /api/v1/ingestion/progress */
    public String postIngestionProgressBatch(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/ingestion/progress", query, body, false);
    }
    /** GET /api/v1/ingestion/{track_id}/progress */
    public String getIngestionProgress(String trackId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/ingestion/" + encodePathParam(trackId) + "/progress", query, body, false);
    }
    /** GET /api/v1/lineage/documents/{document_id} */
    public String getDocumentLineage(String documentId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/lineage/documents/" + encodePathParam(documentId), query, body, false);
    }
    /** GET /api/v1/lineage/entities/{entity_name} */
    public String getEntityLineage(String entityName, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/lineage/entities/" + encodePathParam(entityName), query, body, false);
    }
    /** POST /api/v1/mcp */
    public String mcpOpenapiV1(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/mcp", query, body, false);
    }
    /** DELETE /api/v1/messages/{message_id} */
    public String deleteMessage(String messageId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/messages/" + encodePathParam(messageId), query, body, false);
    }
    /** PATCH /api/v1/messages/{message_id} */
    public String updateMessage(String messageId, Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/messages/" + encodePathParam(messageId), query, body, false);
    }
    /** POST /api/v1/models/discover/refresh */
    public String refreshModelDiscovery(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/models/discover/refresh", query, body, false);
    }
    /** GET /api/v1/models/embedding */
    public String listEmbeddingModels(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/models/embedding", query, body, false);
    }
    /** GET /api/v1/models/llm */
    public String listLlmModels(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/models/llm", query, body, false);
    }
    /** GET /api/v1/models/search */
    public String searchModels(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/models/search", query, body, false);
    }
    /** GET /api/v1/models/{provider} */
    public String getProvider(String provider, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/models/" + encodePathParam(provider), query, body, false);
    }
    /** GET /api/v1/models/{provider}/{model} */
    public String getModel(String provider, String model, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/models/" + encodePathParam(provider) + "/" + encodePathParam(model), query, body, false);
    }
    /** POST /api/v1/parse */
    public String parseDocument(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/parse", query, body, false);
    }
    /** GET /api/v1/parse/backends */
    public String listParseBackends(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/parse/backends", query, body, false);
    }
    /** GET /api/v1/parse/jobs/{id} */
    public String getParseJob(String id, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/parse/jobs/" + encodePathParam(id), query, body, false);
    }
    /** GET /api/v1/pipeline/activity */
    public String getPipelineActivity(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/pipeline/activity", query, body, false);
    }
    /** POST /api/v1/pipeline/cancel */
    public String cancelPipeline(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/pipeline/cancel", query, body, false);
    }
    /** POST /api/v1/pipeline/costs/estimate */
    public String estimateCost(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/pipeline/costs/estimate", query, body, false);
    }
    /** GET /api/v1/pipeline/costs/pricing */
    public String getModelPricing(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/pipeline/costs/pricing", query, body, false);
    }
    /** POST /api/v1/query/context */
    public String retrieveQueryContext(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/query/context", query, body, false);
    }
    /** GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id} */
    public String getContextArtifact(String artifactType, String artifactId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/query/context/artifacts/" + encodePathParam(artifactType) + "/" + encodePathParam(artifactId), query, body, false);
    }
    /** POST /api/v1/query/context/search */
    public String searchQueryContext(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/query/context/search", query, body, false);
    }
    /** GET /api/v1/query/context/{retrieval_id} */
    public String fetchQueryContext(String retrievalId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/query/context/" + encodePathParam(retrievalId), query, body, false);
    }
    /** GET /api/v1/settings/app-attribution */
    public String getAppAttributionSettings(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/settings/app-attribution", query, body, false);
    }
    /** PATCH /api/v1/settings/app-attribution */
    public String updateAppAttribution(Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/settings/app-attribution", query, body, false);
    }
    /** GET /api/v1/settings/attribution */
    public String getAttributionSettings(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/settings/attribution", query, body, false);
    }
    /** GET /api/v1/settings/langfuse */
    public String getLangfuseSettings(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/settings/langfuse", query, body, false);
    }
    /** GET /api/v1/settings/llm-defaults */
    public String getLlmDefaults(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/settings/llm-defaults", query, body, false);
    }
    /** PATCH /api/v1/settings/llm-defaults */
    public String updateLlmDefaults(Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/settings/llm-defaults", query, body, false);
    }
    /** GET /api/v1/settings/providers */
    public String listAvailableProviders(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/settings/providers", query, body, false);
    }
    /** POST /api/v1/setup/initialize */
    public String setupInitialize(Map<String, String> query, Object body) {
        return call("POST", "/api/v1/setup/initialize", query, body, false);
    }
    /** GET /api/v1/setup/status */
    public String setupStatus(Map<String, String> query, Object body) {
        return call("GET", "/api/v1/setup/status", query, body, false);
    }
    /** GET /api/v1/shared/{share_id} */
    public String getSharedConversation(String shareId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/shared/" + encodePathParam(shareId), query, body, false);
    }
    /** GET /api/v1/tasks/{track_id} */
    public String getTask(String trackId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/tasks/" + encodePathParam(trackId), query, body, false);
    }
    /** POST /api/v1/tasks/{track_id}/cancel */
    public String cancelTask(String trackId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/tasks/" + encodePathParam(trackId) + "/cancel", query, body, false);
    }
    /** POST /api/v1/tasks/{track_id}/retry */
    public String retryTask(String trackId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/tasks/" + encodePathParam(trackId) + "/retry", query, body, false);
    }
    /** GET /api/v1/tenants/{tenant_id}/workspaces/by-slug/{slug} */
    public String getWorkspaceBySlug(String tenantId, String slug, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/tenants/" + encodePathParam(tenantId) + "/workspaces/by-slug/" + encodePathParam(slug), query, body, false);
    }
    /** GET /api/v1/users/{user_id} */
    public String getUser(String userId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/users/" + encodePathParam(userId), query, body, false);
    }
    /** DELETE /api/v1/users/{user_id} */
    public String deleteUser(String userId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/users/" + encodePathParam(userId), query, body, false);
    }
    /** PATCH /api/v1/users/{user_id} */
    public String updateUser(String userId, Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/users/" + encodePathParam(userId), query, body, false);
    }
    /** PUT /api/v1/workspaces/{workspace_id}/injection */
    public String putInjection(String workspaceId, Map<String, String> query, Object body) {
        return call("PUT", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injection", query, body, false);
    }
    /** PUT /api/v1/workspaces/{workspace_id}/injection/file */
    public String putInjectionFile(String workspaceId, Map<String, String> query, Object body) {
        return call("PUT", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injection/file", query, body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id}/injections */
    public String listInjections(String workspaceId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections", query, body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    public String getInjection(String workspaceId, String injectionId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query, body, false);
    }
    /** DELETE /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    public String deleteInjection(String workspaceId, String injectionId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query, body, false);
    }
    /** PATCH /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    public String updateInjection(String workspaceId, String injectionId, Map<String, String> query, Object body) {
        return call("PATCH", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query, body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id}/metrics-history */
    public String getMetricsHistory(String workspaceId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/metrics-history", query, body, false);
    }
    /** POST /api/v1/workspaces/{workspace_id}/metrics-snapshot */
    public String triggerMetricsSnapshot(String workspaceId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/metrics-snapshot", query, body, false);
    }
    /** POST /api/v1/workspaces/{workspace_id}/rebuild-embeddings */
    public String rebuildEmbeddings(String workspaceId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/rebuild-embeddings", query, body, false);
    }
    /** POST /api/v1/workspaces/{workspace_id}/rebuild-knowledge-graph */
    public String rebuildKnowledgeGraph(String workspaceId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/rebuild-knowledge-graph", query, body, false);
    }
    /** POST /api/v1/workspaces/{workspace_id}/reprocess-documents */
    public String reprocessAllDocuments(String workspaceId, Map<String, String> query, Object body) {
        return call("POST", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/reprocess-documents", query, body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id}/stats */
    public String getWorkspaceStats(String workspaceId, Map<String, String> query, Object body) {
        return call("GET", "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/stats", query, body, false);
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs */
    public String listWorkspaceJobs(String workspaceId, Map<String, String> query, Object body) {
        return call("GET", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs", query, body, false);
    }
    /** POST /api/v2/workspaces/{workspace_id}/jobs */
    public String createWorkspaceJob(String workspaceId, Map<String, String> query, Object body) {
        return call("POST", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs", query, body, false);
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs/catalog */
    public String listWorkspaceJobCatalog(String workspaceId, Map<String, String> query, Object body) {
        return call("GET", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/catalog", query, body, false);
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs/{job_id} */
    public String getWorkspaceJob(String workspaceId, String jobId, Map<String, String> query, Object body) {
        return call("GET", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/" + encodePathParam(jobId), query, body, false);
    }
    /** DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id} */
    public String cancelWorkspaceJob(String workspaceId, String jobId, Map<String, String> query, Object body) {
        return call("DELETE", "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/" + encodePathParam(jobId), query, body, false);
    }
    /** GET /api/version */
    public String ollamaVersion(Map<String, String> query, Object body) {
        return call("GET", "/api/version", query, body, false);
    }
    /** POST /mcp */
    public String mcpOpenapi(Map<String, String> query, Object body) {
        return call("POST", "/mcp", query, body, false);
    }
    /** GET /oauth/authorize */
    public String oauthAuthorizeGet(Map<String, String> query, Object body) {
        return call("GET", "/oauth/authorize", query, body, false);
    }
    /** POST /oauth/authorize */
    public String oauthAuthorizePost(Map<String, String> query, Object body) {
        return call("POST", "/oauth/authorize", query, body, false);
    }
    /** POST /oauth/register */
    public String oauthRegisterPost(Map<String, String> query, Object body) {
        return call("POST", "/oauth/register", query, body, false);
    }
    /** POST /oauth/revoke */
    public String oauthRevokePost(Map<String, String> query, Object body) {
        return call("POST", "/oauth/revoke", query, body, false);
    }
    /** POST /oauth/token */
    public String oauthTokenPost(Map<String, String> query, Object body) {
        return call("POST", "/oauth/token", query, body, false);
    }
    /** GET /ws/pipeline/progress */
    public String wsPipelineProgress() {
        return http.websocketUrl("/ws/pipeline/progress");
    }
    /** GET /ws/progress/{track_id} */
    public String wsProgressByTrackId(String trackId) {
        return http.websocketUrl("/ws/progress/" + encodePathParam(trackId));
    }
    private static String encodePathParam(String value) {
        var parts = value.split("/", -1);
        var out = new StringBuilder();
        for (int i = 0; i < parts.length; i++) {
            if (i > 0) out.append('/');
            out.append(URLEncoder.encode(parts[i], StandardCharsets.UTF_8).replace("+", "%20"));
        }
        return out.toString();
    }
}
