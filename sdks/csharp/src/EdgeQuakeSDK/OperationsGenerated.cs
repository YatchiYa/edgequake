using System.Collections.Generic;
using System.Linq;

namespace EdgeQuakeSDK;

/// <summary>Generated OpenAPI operations. Do not edit by hand.</summary>
public class OperationsGenerated(HttpHelper http)
{
    private async Task<string> Call(string method, string path, Dictionary<string, string>? query, object? body, bool events)
    {
        var text = await http.ExchangeAsync(method, AppendQuery(path, query), body);
        return events ? string.Join('\n', ParseEvents(text)) : text;
    }

    private static string AppendQuery(string path, Dictionary<string, string>? query)
    {
        if (query is not { Count: > 0 }) return path;
        var qs = string.Join('&', query.Where(kv => kv.Value != null).Select(kv => Uri.EscapeDataString(kv.Key) + "=" + Uri.EscapeDataString(kv.Value)));
        if (qs.Length == 0) return path;
        return path + (path.Contains('?') ? "&" : "?") + qs;
    }

    private static IEnumerable<string> ParseEvents(string text)
    {
        foreach (var block in text.Split("\n\n"))
        {
        foreach (var line in block.Split('\n'))
        {
            if (!line.StartsWith("data:")) continue;
            var data = line[5..].Trim();
            if (data.Length > 0 && data != "[DONE]") yield return data;
        }
        }
    }

    /// <summary>GET /.well-known/mcp/server.json</summary>
    public Task<string> McpRegistryServerJson(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/.well-known/mcp/server.json", query, body, false);
    }
    /// <summary>GET /.well-known/oauth-authorization-server</summary>
    public Task<string> OauthAuthorizationServerMetadata(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/.well-known/oauth-authorization-server", query, body, false);
    }
    /// <summary>GET /.well-known/oauth-protected-resource</summary>
    public Task<string> McpOauthProtectedResource(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/.well-known/oauth-protected-resource", query, body, false);
    }
    /// <summary>GET /.well-known/oauth-protected-resource/mcp</summary>
    public Task<string> McpOauthProtectedResourcePath(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/.well-known/oauth-protected-resource/mcp", query, body, false);
    }
    /// <summary>GET /.well-known/openid-configuration</summary>
    public Task<string> OpenidConfiguration(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/.well-known/openid-configuration", query, body, false);
    }
    /// <summary>POST /api/chat</summary>
    public Task<string> OllamaChat(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/chat", query, body, false);
    }
    /// <summary>POST /api/generate</summary>
    public Task<string> OllamaGenerate(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/generate", query, body, false);
    }
    /// <summary>GET /api/ps</summary>
    public Task<string> OllamaPs(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/ps", query, body, false);
    }
    /// <summary>GET /api/tags</summary>
    public Task<string> OllamaTags(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/tags", query, body, false);
    }
    /// <summary>POST /api/v1/admin/ann/warmup</summary>
    public Task<string> AnnWarmup(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/admin/ann/warmup", query, body, false);
    }
    /// <summary>GET /api/v1/admin/entities/reconcile</summary>
    public Task<string> EntityReconcilePlan(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/admin/entities/reconcile", query, body, false);
    }
    /// <summary>POST /api/v1/admin/entities/reconcile</summary>
    public Task<string> EntityReconcileExecute(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/admin/entities/reconcile", query, body, false);
    }
    /// <summary>GET /api/v1/admin/identity-providers</summary>
    public Task<string> AdminListIdentityProviders(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/admin/identity-providers", query, body, false);
    }
    /// <summary>PUT /api/v1/admin/identity-providers/{slug}</summary>
    public Task<string> AdminUpsertIdentityProvider(string slug, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PUT", "/api/v1/admin/identity-providers/" + EncodePathParam(slug), query, body, false);
    }
    /// <summary>DELETE /api/v1/admin/identity-providers/{slug}</summary>
    public Task<string> AdminDeleteIdentityProvider(string slug, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/admin/identity-providers/" + EncodePathParam(slug), query, body, false);
    }
    /// <summary>GET /api/v1/admin/migration-jobs</summary>
    public Task<string> ListMigrationJobs(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/admin/migration-jobs", query, body, false);
    }
    /// <summary>GET /api/v1/admin/migration-jobs/{job_id}</summary>
    public Task<string> GetMigrationJob(string jobId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/admin/migration-jobs/" + EncodePathParam(jobId), query, body, false);
    }
    /// <summary>POST /api/v1/admin/migration-jobs/{job_id}/cancel</summary>
    public Task<string> CancelMigrationJob(string jobId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/admin/migration-jobs/" + EncodePathParam(jobId) + "/cancel", query, body, false);
    }
    /// <summary>POST /api/v1/admin/migration-jobs/{job_id}/pause</summary>
    public Task<string> PauseMigrationJob(string jobId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/admin/migration-jobs/" + EncodePathParam(jobId) + "/pause", query, body, false);
    }
    /// <summary>POST /api/v1/admin/migration-jobs/{job_id}/resume</summary>
    public Task<string> ResumeMigrationJob(string jobId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/admin/migration-jobs/" + EncodePathParam(jobId) + "/resume", query, body, false);
    }
    /// <summary>GET /api/v1/admin/storage/inspect</summary>
    public Task<string> StorageInspect(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/admin/storage/inspect", query, body, false);
    }
    /// <summary>POST /api/v1/admin/storage/repair</summary>
    public Task<string> StorageRepair(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/admin/storage/repair", query, body, false);
    }
    /// <summary>PATCH /api/v1/admin/tenants/{tenant_id}/quota</summary>
    public Task<string> UpdateTenantQuota(string tenantId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PATCH", "/api/v1/admin/tenants/" + EncodePathParam(tenantId) + "/quota", query, body, false);
    }
    /// <summary>DELETE /api/v1/api-keys/{key_id}</summary>
    public Task<string> RevokeApiKey(string keyId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/api-keys/" + EncodePathParam(keyId), query, body, false);
    }
    /// <summary>POST /api/v1/auth/handoff</summary>
    public Task<string> RedeemSsoHandoff(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/auth/handoff", query, body, false);
    }
    /// <summary>POST /api/v1/auth/oidc/backchannel-logout</summary>
    public Task<string> OidcBackchannelLogout(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/auth/oidc/backchannel-logout", query, body, false);
    }
    /// <summary>GET /api/v1/auth/oidc/callback</summary>
    public Task<string> OidcCallback(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/auth/oidc/callback", query, body, false);
    }
    /// <summary>GET /api/v1/auth/oidc/login</summary>
    public Task<string> OidcLogin(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/auth/oidc/login", query, body, false);
    }
    /// <summary>GET /api/v1/auth/sso/providers</summary>
    public Task<string> ListSsoProviders(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/auth/sso/providers", query, body, false);
    }
    /// <summary>GET /api/v1/chunks/{chunk_id}</summary>
    public Task<string> GetChunkDetail(string chunkId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/chunks/" + EncodePathParam(chunkId), query, body, false);
    }
    /// <summary>GET /api/v1/chunks/{chunk_id}/lineage</summary>
    public Task<string> GetChunkLineage(string chunkId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/chunks/" + EncodePathParam(chunkId) + "/lineage", query, body, false);
    }
    /// <summary>POST /api/v1/conversations/bulk/archive</summary>
    public Task<string> BulkArchiveConversations(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/conversations/bulk/archive", query, body, false);
    }
    /// <summary>POST /api/v1/conversations/bulk/move</summary>
    public Task<string> BulkMoveConversations(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/conversations/bulk/move", query, body, false);
    }
    /// <summary>PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback</summary>
    public Task<string> SetMessageFeedback(string conversationId, string messageId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PATCH", "/api/v1/conversations/" + EncodePathParam(conversationId) + "/messages/" + EncodePathParam(messageId) + "/feedback", query, body, false);
    }
    /// <summary>GET /api/v1/conversations/{id}/messages</summary>
    public Task<string> ListMessages(string id, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/conversations/" + EncodePathParam(id) + "/messages", query, body, false);
    }
    /// <summary>POST /api/v1/conversations/{id}/messages</summary>
    public Task<string> CreateMessage(string id, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/conversations/" + EncodePathParam(id) + "/messages", query, body, false);
    }
    /// <summary>GET /api/v1/decision/models</summary>
    public Task<string> GetDecisionModels(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/decision/models", query, body, false);
    }
    /// <summary>GET /api/v1/decision/status</summary>
    public Task<string> GetDecisionStatus(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/decision/status", query, body, false);
    }
    /// <summary>POST /api/v1/documents/batch-delete</summary>
    public Task<string> BatchDeleteDocuments(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/documents/batch-delete", query, body, false);
    }
    /// <summary>GET /api/v1/documents/pdf/progress/stream/{track_id}</summary>
    public Task<string> GetPdfProgressStream(string trackId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/pdf/progress/stream/" + EncodePathParam(trackId), query, body, true);
    }
    /// <summary>GET /api/v1/documents/pdf/progress/{track_id}</summary>
    public Task<string> GetPdfProgress(string trackId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/pdf/progress/" + EncodePathParam(trackId), query, body, false);
    }
    /// <summary>GET /api/v1/documents/pdf/{pdf_id}</summary>
    public Task<string> GetPdfStatus(string pdfId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/pdf/" + EncodePathParam(pdfId), query, body, false);
    }
    /// <summary>DELETE /api/v1/documents/pdf/{pdf_id}</summary>
    public Task<string> DeletePdf(string pdfId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/documents/pdf/" + EncodePathParam(pdfId), query, body, false);
    }
    /// <summary>DELETE /api/v1/documents/pdf/{pdf_id}/cancel</summary>
    public Task<string> CancelPdfProcessing(string pdfId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/documents/pdf/" + EncodePathParam(pdfId) + "/cancel", query, body, false);
    }
    /// <summary>GET /api/v1/documents/pdf/{pdf_id}/content</summary>
    public Task<string> GetPdfContent(string pdfId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/pdf/" + EncodePathParam(pdfId) + "/content", query, body, false);
    }
    /// <summary>GET /api/v1/documents/pdf/{pdf_id}/download</summary>
    public Task<string> DownloadPdf(string pdfId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/pdf/" + EncodePathParam(pdfId) + "/download", query, body, false);
    }
    /// <summary>POST /api/v1/documents/pdf/{pdf_id}/retry</summary>
    public Task<string> RetryPdfProcessing(string pdfId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/documents/pdf/" + EncodePathParam(pdfId) + "/retry", query, body, false);
    }
    /// <summary>POST /api/v1/documents/scan</summary>
    public Task<string> ScanDirectory(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/documents/scan", query, body, false);
    }
    /// <summary>GET /api/v1/documents/search</summary>
    public Task<string> SearchDocuments(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/search", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}</summary>
    public Task<string> GetDocument(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId), query, body, false);
    }
    /// <summary>DELETE /api/v1/documents/{document_id}</summary>
    public Task<string> DeleteDocument(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/documents/" + EncodePathParam(documentId), query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/assets</summary>
    public Task<string> ListDocumentAssets(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/assets", query, body, false);
    }
    /// <summary>POST /api/v1/documents/{document_id}/assets/include-from-pdf</summary>
    public Task<string> IncludeDocumentAssetsFromPdf(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/documents/" + EncodePathParam(documentId) + "/assets/include-from-pdf", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/assets/{asset_id}</summary>
    public Task<string> DownloadDocumentAssetById(string documentId, string assetId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/assets/" + EncodePathParam(assetId), query, body, false);
    }
    /// <summary>POST /api/v1/documents/{document_id}/cancel</summary>
    public Task<string> CancelDocument(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/documents/" + EncodePathParam(documentId) + "/cancel", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/deletion-impact</summary>
    public Task<string> AnalyzeDeletionImpact(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/deletion-impact", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/download/markdown</summary>
    public Task<string> DownloadDocumentMarkdown(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/download/markdown", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/download/original</summary>
    public Task<string> DownloadDocumentOriginal(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/download/original", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/failed-chunks</summary>
    public Task<string> ListFailedChunks(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/failed-chunks", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/lineage</summary>
    public Task<string> GetDocumentFullLineage(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/lineage", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/lineage/export</summary>
    public Task<string> ExportDocumentLineage(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/lineage/export", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/metadata</summary>
    public Task<string> GetDocumentMetadata(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/metadata", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/mm-assets/{asset_path}</summary>
    public Task<string> DownloadDocumentMmAsset(string documentId, string assetPath, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/mm-assets/" + EncodePathParam(assetPath), query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/pages</summary>
    public Task<string> ListDocumentPages(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/pages", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/pages/health</summary>
    public Task<string> GetPagesHealth(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/pages/health", query, body, false);
    }
    /// <summary>POST /api/v1/documents/{document_id}/pages/reprocess</summary>
    public Task<string> ReprocessPages(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/documents/" + EncodePathParam(documentId) + "/pages/reprocess", query, body, false);
    }
    /// <summary>GET /api/v1/documents/{document_id}/pages/{page_number}/layout</summary>
    public Task<string> GetDocumentPageLayout(string documentId, string pageNumber, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/documents/" + EncodePathParam(documentId) + "/pages/" + EncodePathParam(pageNumber) + "/layout", query, body, false);
    }
    /// <summary>POST /api/v1/documents/{document_id}/reanalyze</summary>
    public Task<string> ReanalyzeMultimodal(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/documents/" + EncodePathParam(documentId) + "/reanalyze", query, body, false);
    }
    /// <summary>POST /api/v1/documents/{document_id}/retry-chunks</summary>
    public Task<string> RetryFailedChunks(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/documents/" + EncodePathParam(documentId) + "/retry-chunks", query, body, false);
    }
    /// <summary>GET /api/v1/entities/{entity_id}/provenance</summary>
    public Task<string> GetEntityProvenance(string entityId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/entities/" + EncodePathParam(entityId) + "/provenance", query, body, false);
    }
    /// <summary>DELETE /api/v1/folders/{folder_id}</summary>
    public Task<string> DeleteFolder(string folderId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/folders/" + EncodePathParam(folderId), query, body, false);
    }
    /// <summary>PATCH /api/v1/folders/{folder_id}</summary>
    public Task<string> UpdateFolder(string folderId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PATCH", "/api/v1/folders/" + EncodePathParam(folderId), query, body, false);
    }
    /// <summary>GET /api/v1/graph/communities</summary>
    public Task<string> GetGraphCommunities(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/graph/communities", query, body, false);
    }
    /// <summary>GET /api/v1/graph/entities/{entity_name}</summary>
    public Task<string> GetEntity(string entityName, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/graph/entities/" + EncodePathParam(entityName), query, body, false);
    }
    /// <summary>PUT /api/v1/graph/entities/{entity_name}</summary>
    public Task<string> UpdateEntity(string entityName, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PUT", "/api/v1/graph/entities/" + EncodePathParam(entityName), query, body, false);
    }
    /// <summary>DELETE /api/v1/graph/entities/{entity_name}</summary>
    public Task<string> DeleteEntity(string entityName, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/graph/entities/" + EncodePathParam(entityName), query, body, false);
    }
    /// <summary>GET /api/v1/graph/entities/{entity_name}/neighborhood</summary>
    public Task<string> GetEntityNeighborhood(string entityName, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/graph/entities/" + EncodePathParam(entityName) + "/neighborhood", query, body, false);
    }
    /// <summary>GET /api/v1/graph/facets</summary>
    public Task<string> GetGraphFacets(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/graph/facets", query, body, false);
    }
    /// <summary>GET /api/v1/graph/nodes/{node_id}</summary>
    public Task<string> GetNode(string nodeId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/graph/nodes/" + EncodePathParam(nodeId), query, body, false);
    }
    /// <summary>GET /api/v1/graph/relationships/{relationship_id}</summary>
    public Task<string> GetRelationship(string relationshipId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/graph/relationships/" + EncodePathParam(relationshipId), query, body, false);
    }
    /// <summary>PUT /api/v1/graph/relationships/{relationship_id}</summary>
    public Task<string> UpdateRelationship(string relationshipId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PUT", "/api/v1/graph/relationships/" + EncodePathParam(relationshipId), query, body, false);
    }
    /// <summary>DELETE /api/v1/graph/relationships/{relationship_id}</summary>
    public Task<string> DeleteRelationship(string relationshipId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/graph/relationships/" + EncodePathParam(relationshipId), query, body, false);
    }
    /// <summary>GET /api/v1/graph/stream</summary>
    public Task<string> StreamGraph(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/graph/stream", query, body, true);
    }
    /// <summary>POST /api/v1/ingestion/progress</summary>
    public Task<string> PostIngestionProgressBatch(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/ingestion/progress", query, body, false);
    }
    /// <summary>GET /api/v1/ingestion/{track_id}/progress</summary>
    public Task<string> GetIngestionProgress(string trackId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/ingestion/" + EncodePathParam(trackId) + "/progress", query, body, false);
    }
    /// <summary>GET /api/v1/lineage/documents/{document_id}</summary>
    public Task<string> GetDocumentLineage(string documentId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/lineage/documents/" + EncodePathParam(documentId), query, body, false);
    }
    /// <summary>GET /api/v1/lineage/entities/{entity_name}</summary>
    public Task<string> GetEntityLineage(string entityName, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/lineage/entities/" + EncodePathParam(entityName), query, body, false);
    }
    /// <summary>POST /api/v1/mcp</summary>
    public Task<string> McpOpenapiV1(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/mcp", query, body, false);
    }
    /// <summary>DELETE /api/v1/messages/{message_id}</summary>
    public Task<string> DeleteMessage(string messageId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/messages/" + EncodePathParam(messageId), query, body, false);
    }
    /// <summary>PATCH /api/v1/messages/{message_id}</summary>
    public Task<string> UpdateMessage(string messageId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PATCH", "/api/v1/messages/" + EncodePathParam(messageId), query, body, false);
    }
    /// <summary>POST /api/v1/models/discover/refresh</summary>
    public Task<string> RefreshModelDiscovery(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/models/discover/refresh", query, body, false);
    }
    /// <summary>GET /api/v1/models/embedding</summary>
    public Task<string> ListEmbeddingModels(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/models/embedding", query, body, false);
    }
    /// <summary>GET /api/v1/models/llm</summary>
    public Task<string> ListLlmModels(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/models/llm", query, body, false);
    }
    /// <summary>GET /api/v1/models/search</summary>
    public Task<string> SearchModels(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/models/search", query, body, false);
    }
    /// <summary>GET /api/v1/models/{provider}</summary>
    public Task<string> GetProvider(string provider, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/models/" + EncodePathParam(provider), query, body, false);
    }
    /// <summary>GET /api/v1/models/{provider}/{model}</summary>
    public Task<string> GetModel(string provider, string model, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/models/" + EncodePathParam(provider) + "/" + EncodePathParam(model), query, body, false);
    }
    /// <summary>POST /api/v1/parse</summary>
    public Task<string> ParseDocument(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/parse", query, body, false);
    }
    /// <summary>GET /api/v1/parse/backends</summary>
    public Task<string> ListParseBackends(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/parse/backends", query, body, false);
    }
    /// <summary>GET /api/v1/parse/jobs/{id}</summary>
    public Task<string> GetParseJob(string id, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/parse/jobs/" + EncodePathParam(id), query, body, false);
    }
    /// <summary>GET /api/v1/pipeline/activity</summary>
    public Task<string> GetPipelineActivity(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/pipeline/activity", query, body, false);
    }
    /// <summary>GET /api/v1/pipeline/costs/pricing</summary>
    public Task<string> GetModelPricing(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/pipeline/costs/pricing", query, body, false);
    }
    /// <summary>POST /api/v1/query/context</summary>
    public Task<string> RetrieveQueryContext(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/query/context", query, body, false);
    }
    /// <summary>GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id}</summary>
    public Task<string> GetContextArtifact(string artifactType, string artifactId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/query/context/artifacts/" + EncodePathParam(artifactType) + "/" + EncodePathParam(artifactId), query, body, false);
    }
    /// <summary>POST /api/v1/query/context/search</summary>
    public Task<string> SearchQueryContext(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/query/context/search", query, body, false);
    }
    /// <summary>GET /api/v1/query/context/{retrieval_id}</summary>
    public Task<string> FetchQueryContext(string retrievalId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/query/context/" + EncodePathParam(retrievalId), query, body, false);
    }
    /// <summary>GET /api/v1/settings/app-attribution</summary>
    public Task<string> GetAppAttributionSettings(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/settings/app-attribution", query, body, false);
    }
    /// <summary>PATCH /api/v1/settings/app-attribution</summary>
    public Task<string> UpdateAppAttribution(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PATCH", "/api/v1/settings/app-attribution", query, body, false);
    }
    /// <summary>GET /api/v1/settings/attribution</summary>
    public Task<string> GetAttributionSettings(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/settings/attribution", query, body, false);
    }
    /// <summary>GET /api/v1/settings/langfuse</summary>
    public Task<string> GetLangfuseSettings(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/settings/langfuse", query, body, false);
    }
    /// <summary>GET /api/v1/settings/llm-defaults</summary>
    public Task<string> GetLlmDefaults(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/settings/llm-defaults", query, body, false);
    }
    /// <summary>PATCH /api/v1/settings/llm-defaults</summary>
    public Task<string> UpdateLlmDefaults(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PATCH", "/api/v1/settings/llm-defaults", query, body, false);
    }
    /// <summary>POST /api/v1/setup/initialize</summary>
    public Task<string> SetupInitialize(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/setup/initialize", query, body, false);
    }
    /// <summary>GET /api/v1/setup/status</summary>
    public Task<string> SetupStatus(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/setup/status", query, body, false);
    }
    /// <summary>GET /api/v1/shared/{share_id}</summary>
    public Task<string> GetSharedConversation(string shareId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/shared/" + EncodePathParam(shareId), query, body, false);
    }
    /// <summary>GET /api/v1/tasks/{track_id}</summary>
    public Task<string> GetTask(string trackId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/tasks/" + EncodePathParam(trackId), query, body, false);
    }
    /// <summary>POST /api/v1/tasks/{track_id}/cancel</summary>
    public Task<string> CancelTask(string trackId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/tasks/" + EncodePathParam(trackId) + "/cancel", query, body, false);
    }
    /// <summary>POST /api/v1/tasks/{track_id}/retry</summary>
    public Task<string> RetryTask(string trackId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/tasks/" + EncodePathParam(trackId) + "/retry", query, body, false);
    }
    /// <summary>GET /api/v1/tenants/{tenant_id}</summary>
    public Task<string> GetTenant(string tenantId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/tenants/" + EncodePathParam(tenantId), query, body, false);
    }
    /// <summary>PUT /api/v1/tenants/{tenant_id}</summary>
    public Task<string> UpdateTenant(string tenantId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PUT", "/api/v1/tenants/" + EncodePathParam(tenantId), query, body, false);
    }
    /// <summary>DELETE /api/v1/tenants/{tenant_id}</summary>
    public Task<string> DeleteTenant(string tenantId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/tenants/" + EncodePathParam(tenantId), query, body, false);
    }
    /// <summary>GET /api/v1/tenants/{tenant_id}/workspaces</summary>
    public Task<string> ListWorkspaces(string tenantId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/tenants/" + EncodePathParam(tenantId) + "/workspaces", query, body, false);
    }
    /// <summary>POST /api/v1/tenants/{tenant_id}/workspaces</summary>
    public Task<string> CreateWorkspace(string tenantId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/tenants/" + EncodePathParam(tenantId) + "/workspaces", query, body, false);
    }
    /// <summary>GET /api/v1/tenants/{tenant_id}/workspaces/by-slug/{slug}</summary>
    public Task<string> GetWorkspaceBySlug(string tenantId, string slug, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/tenants/" + EncodePathParam(tenantId) + "/workspaces/by-slug/" + EncodePathParam(slug), query, body, false);
    }
    /// <summary>GET /api/v1/users/{user_id}</summary>
    public Task<string> GetUser(string userId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/users/" + EncodePathParam(userId), query, body, false);
    }
    /// <summary>DELETE /api/v1/users/{user_id}</summary>
    public Task<string> DeleteUser(string userId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/users/" + EncodePathParam(userId), query, body, false);
    }
    /// <summary>PATCH /api/v1/users/{user_id}</summary>
    public Task<string> UpdateUser(string userId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PATCH", "/api/v1/users/" + EncodePathParam(userId), query, body, false);
    }
    /// <summary>GET /api/v1/workspaces/{workspace_id}</summary>
    public Task<string> GetWorkspace(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/workspaces/" + EncodePathParam(workspaceId), query, body, false);
    }
    /// <summary>PUT /api/v1/workspaces/{workspace_id}</summary>
    public Task<string> UpdateWorkspace(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PUT", "/api/v1/workspaces/" + EncodePathParam(workspaceId), query, body, false);
    }
    /// <summary>DELETE /api/v1/workspaces/{workspace_id}</summary>
    public Task<string> DeleteWorkspace(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/workspaces/" + EncodePathParam(workspaceId), query, body, false);
    }
    /// <summary>PUT /api/v1/workspaces/{workspace_id}/injection</summary>
    public Task<string> PutInjection(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PUT", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/injection", query, body, false);
    }
    /// <summary>PUT /api/v1/workspaces/{workspace_id}/injection/file</summary>
    public Task<string> PutInjectionFile(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PUT", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/injection/file", query, body, false);
    }
    /// <summary>GET /api/v1/workspaces/{workspace_id}/injections</summary>
    public Task<string> ListInjections(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/injections", query, body, false);
    }
    /// <summary>GET /api/v1/workspaces/{workspace_id}/injections/{injection_id}</summary>
    public Task<string> GetInjection(string workspaceId, string injectionId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/injections/" + EncodePathParam(injectionId), query, body, false);
    }
    /// <summary>DELETE /api/v1/workspaces/{workspace_id}/injections/{injection_id}</summary>
    public Task<string> DeleteInjection(string workspaceId, string injectionId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/injections/" + EncodePathParam(injectionId), query, body, false);
    }
    /// <summary>PATCH /api/v1/workspaces/{workspace_id}/injections/{injection_id}</summary>
    public Task<string> UpdateInjection(string workspaceId, string injectionId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("PATCH", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/injections/" + EncodePathParam(injectionId), query, body, false);
    }
    /// <summary>GET /api/v1/workspaces/{workspace_id}/metrics-history</summary>
    public Task<string> GetMetricsHistory(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/metrics-history", query, body, false);
    }
    /// <summary>POST /api/v1/workspaces/{workspace_id}/metrics-snapshot</summary>
    public Task<string> TriggerMetricsSnapshot(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/metrics-snapshot", query, body, false);
    }
    /// <summary>POST /api/v1/workspaces/{workspace_id}/rebuild-embeddings</summary>
    public Task<string> RebuildEmbeddings(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/rebuild-embeddings", query, body, false);
    }
    /// <summary>POST /api/v1/workspaces/{workspace_id}/rebuild-knowledge-graph</summary>
    public Task<string> RebuildKnowledgeGraph(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/rebuild-knowledge-graph", query, body, false);
    }
    /// <summary>POST /api/v1/workspaces/{workspace_id}/reprocess-documents</summary>
    public Task<string> ReprocessAllDocuments(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/reprocess-documents", query, body, false);
    }
    /// <summary>GET /api/v1/workspaces/{workspace_id}/stats</summary>
    public Task<string> GetWorkspaceStats(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v1/workspaces/" + EncodePathParam(workspaceId) + "/stats", query, body, false);
    }
    /// <summary>GET /api/v2/workspaces/{workspace_id}/jobs</summary>
    public Task<string> ListWorkspaceJobs(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v2/workspaces/" + EncodePathParam(workspaceId) + "/jobs", query, body, false);
    }
    /// <summary>POST /api/v2/workspaces/{workspace_id}/jobs</summary>
    public Task<string> CreateWorkspaceJob(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/api/v2/workspaces/" + EncodePathParam(workspaceId) + "/jobs", query, body, false);
    }
    /// <summary>GET /api/v2/workspaces/{workspace_id}/jobs/catalog</summary>
    public Task<string> ListWorkspaceJobCatalog(string workspaceId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v2/workspaces/" + EncodePathParam(workspaceId) + "/jobs/catalog", query, body, false);
    }
    /// <summary>GET /api/v2/workspaces/{workspace_id}/jobs/{job_id}</summary>
    public Task<string> GetWorkspaceJob(string workspaceId, string jobId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/v2/workspaces/" + EncodePathParam(workspaceId) + "/jobs/" + EncodePathParam(jobId), query, body, false);
    }
    /// <summary>DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id}</summary>
    public Task<string> CancelWorkspaceJob(string workspaceId, string jobId, Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("DELETE", "/api/v2/workspaces/" + EncodePathParam(workspaceId) + "/jobs/" + EncodePathParam(jobId), query, body, false);
    }
    /// <summary>GET /api/version</summary>
    public Task<string> OllamaVersion(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/api/version", query, body, false);
    }
    /// <summary>POST /mcp</summary>
    public Task<string> McpOpenapi(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/mcp", query, body, false);
    }
    /// <summary>GET /oauth/authorize</summary>
    public Task<string> OauthAuthorizeGet(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("GET", "/oauth/authorize", query, body, false);
    }
    /// <summary>POST /oauth/authorize</summary>
    public Task<string> OauthAuthorizePost(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/oauth/authorize", query, body, false);
    }
    /// <summary>POST /oauth/register</summary>
    public Task<string> OauthRegisterPost(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/oauth/register", query, body, false);
    }
    /// <summary>POST /oauth/revoke</summary>
    public Task<string> OauthRevokePost(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/oauth/revoke", query, body, false);
    }
    /// <summary>POST /oauth/token</summary>
    public Task<string> OauthTokenPost(Dictionary<string, string>? query = null, object? body = null)
    {
        return Call("POST", "/oauth/token", query, body, false);
    }
    /// <summary>GET /ws/pipeline/progress</summary>
    public string WsPipelineProgress() => http.WebSocketUrl("/ws/pipeline/progress");
    /// <summary>GET /ws/progress/{track_id}</summary>
    public string WsProgressByTrackId(string trackId) => http.WebSocketUrl("/ws/progress/" + EncodePathParam(trackId));
    private static string EncodePathParam(string value) =>
        string.Join("/", value.Split('/').Select(Uri.EscapeDataString));
}
