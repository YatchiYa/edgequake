import Foundation

/// Generated OpenAPI operations. Do not edit by hand.
public final class OperationsGenerated: @unchecked Sendable {
    private let http: HttpHelper
    init(_ http: HttpHelper) { self.http = http }

    private func call(_ method: String, path: String, query: [String: String], body: [String: String]?, events: Bool) async throws -> String {
        let text = try await http.call(method, path: appendQuery(path, query), body: body)
        return events ? parseEvents(text).joined(separator: "\n") : text
    }

    private func appendQuery(_ path: String, _ query: [String: String]) -> String {
        if query.isEmpty { return path }
        let qs = query.map { "\(encodeQuery($0.key))=\(encodeQuery($0.value))" }.joined(separator: "&")
        return path + (path.contains("?") ? "&" : "?") + qs
    }

    private func parseEvents(_ text: String) -> [String] {
        text.components(separatedBy: "\n\n").flatMap { block in
            block.components(separatedBy: "\n").compactMap { line -> String? in
                guard line.hasPrefix("data:") else { return nil }
                let data = String(line.dropFirst(5)).trimmingCharacters(in: .whitespaces)
                return (data.isEmpty || data == "[DONE]") ? nil : data
            }
        }
    }

    private func socket(_ path: String) -> String {
        var root = http.config.baseUrl
        while root.hasSuffix("/") { root.removeLast() }
        if root.hasPrefix("https://") { root = "wss://" + root.dropFirst(8) }
        else if root.hasPrefix("http://") { root = "ws://" + root.dropFirst(7) }
        return root + path
    }

    /// GET /.well-known/mcp/server.json
    public func mcpRegistryServerJson(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/.well-known/mcp/server.json", query: query, body: body, events: false)
    }
    /// GET /.well-known/oauth-authorization-server
    public func oauthAuthorizationServerMetadata(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/.well-known/oauth-authorization-server", query: query, body: body, events: false)
    }
    /// GET /.well-known/oauth-protected-resource
    public func mcpOauthProtectedResource(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/.well-known/oauth-protected-resource", query: query, body: body, events: false)
    }
    /// GET /.well-known/oauth-protected-resource/mcp
    public func mcpOauthProtectedResourcePath(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/.well-known/oauth-protected-resource/mcp", query: query, body: body, events: false)
    }
    /// GET /.well-known/openid-configuration
    public func openidConfiguration(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/.well-known/openid-configuration", query: query, body: body, events: false)
    }
    /// POST /api/chat
    public func ollamaChat(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/chat", query: query, body: body, events: false)
    }
    /// POST /api/generate
    public func ollamaGenerate(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/generate", query: query, body: body, events: false)
    }
    /// GET /api/ps
    public func ollamaPs(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/ps", query: query, body: body, events: false)
    }
    /// GET /api/tags
    public func ollamaTags(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/tags", query: query, body: body, events: false)
    }
    /// POST /api/v1/admin/ann/warmup
    public func annWarmup(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/admin/ann/warmup", query: query, body: body, events: false)
    }
    /// GET /api/v1/admin/config/defaults
    public func getServerDefaults(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/admin/config/defaults", query: query, body: body, events: false)
    }
    /// PATCH /api/v1/admin/config/defaults
    public func updateServerDefaults(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/admin/config/defaults", query: query, body: body, events: false)
    }
    /// GET /api/v1/admin/entities/reconcile
    public func entityReconcilePlan(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/admin/entities/reconcile", query: query, body: body, events: false)
    }
    /// POST /api/v1/admin/entities/reconcile
    public func entityReconcileExecute(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/admin/entities/reconcile", query: query, body: body, events: false)
    }
    /// GET /api/v1/admin/identity-providers
    public func adminListIdentityProviders(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/admin/identity-providers", query: query, body: body, events: false)
    }
    /// PUT /api/v1/admin/identity-providers/{slug}
    public func adminUpsertIdentityProvider(slug: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PUT", path: "/api/v1/admin/identity-providers/" + encodePathParam(slug), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/admin/identity-providers/{slug}
    public func adminDeleteIdentityProvider(slug: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/admin/identity-providers/" + encodePathParam(slug), query: query, body: body, events: false)
    }
    /// GET /api/v1/admin/migration-jobs
    public func listMigrationJobs(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/admin/migration-jobs", query: query, body: body, events: false)
    }
    /// GET /api/v1/admin/migration-jobs/{job_id}
    public func getMigrationJob(jobId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/admin/migration-jobs/" + encodePathParam(jobId), query: query, body: body, events: false)
    }
    /// POST /api/v1/admin/migration-jobs/{job_id}/cancel
    public func cancelMigrationJob(jobId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/cancel", query: query, body: body, events: false)
    }
    /// POST /api/v1/admin/migration-jobs/{job_id}/pause
    public func pauseMigrationJob(jobId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/pause", query: query, body: body, events: false)
    }
    /// POST /api/v1/admin/migration-jobs/{job_id}/resume
    public func resumeMigrationJob(jobId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/admin/migration-jobs/" + encodePathParam(jobId) + "/resume", query: query, body: body, events: false)
    }
    /// GET /api/v1/admin/storage/inspect
    public func storageInspect(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/admin/storage/inspect", query: query, body: body, events: false)
    }
    /// POST /api/v1/admin/storage/repair
    public func storageRepair(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/admin/storage/repair", query: query, body: body, events: false)
    }
    /// PATCH /api/v1/admin/tenants/{tenant_id}/quota
    public func updateTenantQuota(tenantId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/admin/tenants/" + encodePathParam(tenantId) + "/quota", query: query, body: body, events: false)
    }
    /// DELETE /api/v1/api-keys/{key_id}
    public func revokeApiKey(keyId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/api-keys/" + encodePathParam(keyId), query: query, body: body, events: false)
    }
    /// POST /api/v1/auth/handoff
    public func redeemSsoHandoff(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/auth/handoff", query: query, body: body, events: false)
    }
    /// POST /api/v1/auth/oidc/backchannel-logout
    public func oidcBackchannelLogout(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/auth/oidc/backchannel-logout", query: query, body: body, events: false)
    }
    /// GET /api/v1/auth/oidc/callback
    public func oidcCallback(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/auth/oidc/callback", query: query, body: body, events: false)
    }
    /// GET /api/v1/auth/oidc/login
    public func oidcLogin(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/auth/oidc/login", query: query, body: body, events: false)
    }
    /// GET /api/v1/auth/sso/providers
    public func listSsoProviders(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/auth/sso/providers", query: query, body: body, events: false)
    }
    /// POST /api/v1/chat/completions/stream
    public func chatCompletionStream(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/chat/completions/stream", query: query, body: body, events: true)
    }
    /// GET /api/v1/chunks/{chunk_id}
    public func getChunkDetail(chunkId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/chunks/" + encodePathParam(chunkId), query: query, body: body, events: false)
    }
    /// GET /api/v1/chunks/{chunk_id}/lineage
    public func getChunkLineage(chunkId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/chunks/" + encodePathParam(chunkId) + "/lineage", query: query, body: body, events: false)
    }
    /// GET /api/v1/config/effective
    public func getEffectiveConfig(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/config/effective", query: query, body: body, events: false)
    }
    /// PUT /api/v1/connections/{id}
    public func updateConnection(id: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PUT", path: "/api/v1/connections/" + encodePathParam(id), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/connections/{id}
    public func deleteConnection(id: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/connections/" + encodePathParam(id), query: query, body: body, events: false)
    }
    /// POST /api/v1/connections/{id}/test
    public func testStoredConnection(id: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/connections/" + encodePathParam(id) + "/test", query: query, body: body, events: false)
    }
    /// POST /api/v1/conversations/bulk/archive
    public func bulkArchiveConversations(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/conversations/bulk/archive", query: query, body: body, events: false)
    }
    /// POST /api/v1/conversations/bulk/move
    public func bulkMoveConversations(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/conversations/bulk/move", query: query, body: body, events: false)
    }
    /// POST /api/v1/conversations/import
    public func importConversations(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/conversations/import", query: query, body: body, events: false)
    }
    /// PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback
    public func setMessageFeedback(conversationId: String, messageId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/conversations/" + encodePathParam(conversationId) + "/messages/" + encodePathParam(messageId) + "/feedback", query: query, body: body, events: false)
    }
    /// GET /api/v1/conversations/{id}/messages
    public func listMessages(id: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/conversations/" + encodePathParam(id) + "/messages", query: query, body: body, events: false)
    }
    /// POST /api/v1/conversations/{id}/messages
    public func createMessage(id: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/conversations/" + encodePathParam(id) + "/messages", query: query, body: body, events: false)
    }
    /// POST /api/v1/conversations/{id}/share
    public func shareConversation(id: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/conversations/" + encodePathParam(id) + "/share", query: query, body: body, events: false)
    }
    /// DELETE /api/v1/conversations/{id}/share
    public func unshareConversation(id: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/conversations/" + encodePathParam(id) + "/share", query: query, body: body, events: false)
    }
    /// GET /api/v1/decision/models
    public func getDecisionModels(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/decision/models", query: query, body: body, events: false)
    }
    /// GET /api/v1/decision/status
    public func getDecisionStatus(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/decision/status", query: query, body: body, events: false)
    }
    /// POST /api/v1/documents/batch-delete
    public func batchDeleteDocuments(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/documents/batch-delete", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/pdf/progress/stream/{track_id}
    public func getPdfProgressStream(trackId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/pdf/progress/stream/" + encodePathParam(trackId), query: query, body: body, events: true)
    }
    /// GET /api/v1/documents/pdf/progress/{track_id}
    public func getPdfProgress(trackId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/pdf/progress/" + encodePathParam(trackId), query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/pdf/{pdf_id}
    public func getPdfStatus(pdfId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/pdf/" + encodePathParam(pdfId), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/documents/pdf/{pdf_id}
    public func deletePdf(pdfId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/documents/pdf/" + encodePathParam(pdfId), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/documents/pdf/{pdf_id}/cancel
    public func cancelPdfProcessing(pdfId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/cancel", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/pdf/{pdf_id}/content
    public func getPdfContent(pdfId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/content", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/pdf/{pdf_id}/download
    public func downloadPdf(pdfId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/download", query: query, body: body, events: false)
    }
    /// POST /api/v1/documents/pdf/{pdf_id}/retry
    public func retryPdfProcessing(pdfId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/documents/pdf/" + encodePathParam(pdfId) + "/retry", query: query, body: body, events: false)
    }
    /// POST /api/v1/documents/scan
    public func scanDirectory(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/documents/scan", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/search
    public func searchDocuments(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/search", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/track/{track_id}
    public func getTrackStatus(trackId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/track/" + encodePathParam(trackId), query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}
    public func getDocument(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/documents/{document_id}
    public func deleteDocument(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/documents/" + encodePathParam(documentId), query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/assets
    public func listDocumentAssets(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/assets", query: query, body: body, events: false)
    }
    /// POST /api/v1/documents/{document_id}/assets/include-from-pdf
    public func includeDocumentAssetsFromPdf(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/documents/" + encodePathParam(documentId) + "/assets/include-from-pdf", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/assets/{asset_id}
    public func downloadDocumentAssetById(documentId: String, assetId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/assets/" + encodePathParam(assetId), query: query, body: body, events: false)
    }
    /// POST /api/v1/documents/{document_id}/cancel
    public func cancelDocument(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/documents/" + encodePathParam(documentId) + "/cancel", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/deletion-impact
    public func analyzeDeletionImpact(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/deletion-impact", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/download/markdown
    public func downloadDocumentMarkdown(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/download/markdown", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/download/original
    public func downloadDocumentOriginal(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/download/original", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/failed-chunks
    public func listFailedChunks(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/failed-chunks", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/lineage
    public func getDocumentFullLineage(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/lineage", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/lineage/export
    public func exportDocumentLineage(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/lineage/export", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/metadata
    public func getDocumentMetadata(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/metadata", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/mm-assets/{asset_path}
    public func downloadDocumentMmAsset(documentId: String, assetPath: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/mm-assets/" + encodePathParam(assetPath), query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/pages
    public func listDocumentPages(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/pages", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/pages/health
    public func getPagesHealth(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/pages/health", query: query, body: body, events: false)
    }
    /// POST /api/v1/documents/{document_id}/pages/reprocess
    public func reprocessPages(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/documents/" + encodePathParam(documentId) + "/pages/reprocess", query: query, body: body, events: false)
    }
    /// GET /api/v1/documents/{document_id}/pages/{page_number}/layout
    public func getDocumentPageLayout(documentId: String, pageNumber: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/documents/" + encodePathParam(documentId) + "/pages/" + encodePathParam(pageNumber) + "/layout", query: query, body: body, events: false)
    }
    /// POST /api/v1/documents/{document_id}/reanalyze
    public func reanalyzeMultimodal(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/documents/" + encodePathParam(documentId) + "/reanalyze", query: query, body: body, events: false)
    }
    /// POST /api/v1/documents/{document_id}/retry-chunks
    public func retryFailedChunks(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/documents/" + encodePathParam(documentId) + "/retry-chunks", query: query, body: body, events: false)
    }
    /// GET /api/v1/entities/{entity_id}/provenance
    public func getEntityProvenance(entityId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/entities/" + encodePathParam(entityId) + "/provenance", query: query, body: body, events: false)
    }
    /// DELETE /api/v1/folders/{folder_id}
    public func deleteFolder(folderId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/folders/" + encodePathParam(folderId), query: query, body: body, events: false)
    }
    /// PATCH /api/v1/folders/{folder_id}
    public func updateFolder(folderId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/folders/" + encodePathParam(folderId), query: query, body: body, events: false)
    }
    /// GET /api/v1/graph/communities
    public func getGraphCommunities(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/graph/communities", query: query, body: body, events: false)
    }
    /// GET /api/v1/graph/entities/{entity_name}
    public func getEntity(entityName: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/graph/entities/" + encodePathParam(entityName), query: query, body: body, events: false)
    }
    /// PUT /api/v1/graph/entities/{entity_name}
    public func updateEntity(entityName: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PUT", path: "/api/v1/graph/entities/" + encodePathParam(entityName), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/graph/entities/{entity_name}
    public func deleteEntity(entityName: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/graph/entities/" + encodePathParam(entityName), query: query, body: body, events: false)
    }
    /// GET /api/v1/graph/entities/{entity_name}/neighborhood
    public func getEntityNeighborhood(entityName: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/graph/entities/" + encodePathParam(entityName) + "/neighborhood", query: query, body: body, events: false)
    }
    /// GET /api/v1/graph/facets
    public func getGraphFacets(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/graph/facets", query: query, body: body, events: false)
    }
    /// GET /api/v1/graph/nodes/{node_id}
    public func getNode(nodeId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/graph/nodes/" + encodePathParam(nodeId), query: query, body: body, events: false)
    }
    /// GET /api/v1/graph/relationships/{relationship_id}
    public func getRelationship(relationshipId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query: query, body: body, events: false)
    }
    /// PUT /api/v1/graph/relationships/{relationship_id}
    public func updateRelationship(relationshipId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PUT", path: "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/graph/relationships/{relationship_id}
    public func deleteRelationship(relationshipId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/graph/relationships/" + encodePathParam(relationshipId), query: query, body: body, events: false)
    }
    /// GET /api/v1/graph/stream
    public func streamGraph(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/graph/stream", query: query, body: body, events: true)
    }
    /// POST /api/v1/ingestion/progress
    public func postIngestionProgressBatch(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/ingestion/progress", query: query, body: body, events: false)
    }
    /// GET /api/v1/ingestion/{track_id}/progress
    public func getIngestionProgress(trackId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/ingestion/" + encodePathParam(trackId) + "/progress", query: query, body: body, events: false)
    }
    /// GET /api/v1/lineage/documents/{document_id}
    public func getDocumentLineage(documentId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/lineage/documents/" + encodePathParam(documentId), query: query, body: body, events: false)
    }
    /// GET /api/v1/lineage/entities/{entity_name}
    public func getEntityLineage(entityName: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/lineage/entities/" + encodePathParam(entityName), query: query, body: body, events: false)
    }
    /// POST /api/v1/mcp
    public func mcpOpenapiV1(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/mcp", query: query, body: body, events: false)
    }
    /// DELETE /api/v1/messages/{message_id}
    public func deleteMessage(messageId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/messages/" + encodePathParam(messageId), query: query, body: body, events: false)
    }
    /// PATCH /api/v1/messages/{message_id}
    public func updateMessage(messageId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/messages/" + encodePathParam(messageId), query: query, body: body, events: false)
    }
    /// POST /api/v1/models/discover/refresh
    public func refreshModelDiscovery(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/models/discover/refresh", query: query, body: body, events: false)
    }
    /// GET /api/v1/models/search
    public func searchModels(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/models/search", query: query, body: body, events: false)
    }
    /// GET /api/v1/models/{provider}
    public func getProvider(provider: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/models/" + encodePathParam(provider), query: query, body: body, events: false)
    }
    /// GET /api/v1/models/{provider}/{model}
    public func getModel(provider: String, model: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/models/" + encodePathParam(provider) + "/" + encodePathParam(model), query: query, body: body, events: false)
    }
    /// POST /api/v1/parse
    public func parseDocument(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/parse", query: query, body: body, events: false)
    }
    /// GET /api/v1/parse/backends
    public func listParseBackends(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/parse/backends", query: query, body: body, events: false)
    }
    /// GET /api/v1/parse/jobs/{id}
    public func getParseJob(id: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/parse/jobs/" + encodePathParam(id), query: query, body: body, events: false)
    }
    /// GET /api/v1/pipeline/activity
    public func getPipelineActivity(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/pipeline/activity", query: query, body: body, events: false)
    }
    /// POST /api/v1/pipeline/costs/estimate
    public func estimateCost(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/pipeline/costs/estimate", query: query, body: body, events: false)
    }
    /// GET /api/v1/pipeline/costs/pricing
    public func getModelPricing(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/pipeline/costs/pricing", query: query, body: body, events: false)
    }
    /// POST /api/v1/query/context
    public func retrieveQueryContext(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/query/context", query: query, body: body, events: false)
    }
    /// GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id}
    public func getContextArtifact(artifactType: String, artifactId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/query/context/artifacts/" + encodePathParam(artifactType) + "/" + encodePathParam(artifactId), query: query, body: body, events: false)
    }
    /// POST /api/v1/query/context/search
    public func searchQueryContext(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/query/context/search", query: query, body: body, events: false)
    }
    /// GET /api/v1/query/context/{retrieval_id}
    public func fetchQueryContext(retrievalId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/query/context/" + encodePathParam(retrievalId), query: query, body: body, events: false)
    }
    /// POST /api/v1/query/stream
    public func streamQuery(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/query/stream", query: query, body: body, events: true)
    }
    /// GET /api/v1/settings/app-attribution
    public func getAppAttributionSettings(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/settings/app-attribution", query: query, body: body, events: false)
    }
    /// PATCH /api/v1/settings/app-attribution
    public func updateAppAttribution(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/settings/app-attribution", query: query, body: body, events: false)
    }
    /// GET /api/v1/settings/attribution
    public func getAttributionSettings(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/settings/attribution", query: query, body: body, events: false)
    }
    /// GET /api/v1/settings/langfuse
    public func getLangfuseSettings(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/settings/langfuse", query: query, body: body, events: false)
    }
    /// GET /api/v1/settings/llm-defaults
    public func getLlmDefaults(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/settings/llm-defaults", query: query, body: body, events: false)
    }
    /// PATCH /api/v1/settings/llm-defaults
    public func updateLlmDefaults(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/settings/llm-defaults", query: query, body: body, events: false)
    }
    /// POST /api/v1/setup/initialize
    public func setupInitialize(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/setup/initialize", query: query, body: body, events: false)
    }
    /// GET /api/v1/setup/status
    public func setupStatus(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/setup/status", query: query, body: body, events: false)
    }
    /// GET /api/v1/tasks/{track_id}
    public func getTask(trackId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/tasks/" + encodePathParam(trackId), query: query, body: body, events: false)
    }
    /// POST /api/v1/tasks/{track_id}/cancel
    public func cancelTask(trackId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/tasks/" + encodePathParam(trackId) + "/cancel", query: query, body: body, events: false)
    }
    /// POST /api/v1/tasks/{track_id}/retry
    public func retryTask(trackId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/tasks/" + encodePathParam(trackId) + "/retry", query: query, body: body, events: false)
    }
    /// GET /api/v1/tenants/{tenant_id}
    public func getTenant(tenantId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/tenants/" + encodePathParam(tenantId), query: query, body: body, events: false)
    }
    /// PUT /api/v1/tenants/{tenant_id}
    public func updateTenant(tenantId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PUT", path: "/api/v1/tenants/" + encodePathParam(tenantId), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/tenants/{tenant_id}
    public func deleteTenant(tenantId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/tenants/" + encodePathParam(tenantId), query: query, body: body, events: false)
    }
    /// GET /api/v1/tenants/{tenant_id}/workspaces
    public func listWorkspaces(tenantId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/tenants/" + encodePathParam(tenantId) + "/workspaces", query: query, body: body, events: false)
    }
    /// POST /api/v1/tenants/{tenant_id}/workspaces
    public func createWorkspace(tenantId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/tenants/" + encodePathParam(tenantId) + "/workspaces", query: query, body: body, events: false)
    }
    /// GET /api/v1/tenants/{tenant_id}/workspaces/by-slug/{slug}
    public func getWorkspaceBySlug(tenantId: String, slug: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/tenants/" + encodePathParam(tenantId) + "/workspaces/by-slug/" + encodePathParam(slug), query: query, body: body, events: false)
    }
    /// GET /api/v1/users/{user_id}
    public func getUser(userId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/users/" + encodePathParam(userId), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/users/{user_id}
    public func deleteUser(userId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/users/" + encodePathParam(userId), query: query, body: body, events: false)
    }
    /// PATCH /api/v1/users/{user_id}
    public func updateUser(userId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/users/" + encodePathParam(userId), query: query, body: body, events: false)
    }
    /// GET /api/v1/workspaces/{workspace_id}
    public func getWorkspace(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/workspaces/" + encodePathParam(workspaceId), query: query, body: body, events: false)
    }
    /// PUT /api/v1/workspaces/{workspace_id}
    public func updateWorkspace(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PUT", path: "/api/v1/workspaces/" + encodePathParam(workspaceId), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/workspaces/{workspace_id}
    public func deleteWorkspace(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/workspaces/" + encodePathParam(workspaceId), query: query, body: body, events: false)
    }
    /// PUT /api/v1/workspaces/{workspace_id}/injection
    public func putInjection(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PUT", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injection", query: query, body: body, events: false)
    }
    /// PUT /api/v1/workspaces/{workspace_id}/injection/file
    public func putInjectionFile(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PUT", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injection/file", query: query, body: body, events: false)
    }
    /// GET /api/v1/workspaces/{workspace_id}/injections
    public func listInjections(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections", query: query, body: body, events: false)
    }
    /// GET /api/v1/workspaces/{workspace_id}/injections/{injection_id}
    public func getInjection(workspaceId: String, injectionId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query: query, body: body, events: false)
    }
    /// DELETE /api/v1/workspaces/{workspace_id}/injections/{injection_id}
    public func deleteInjection(workspaceId: String, injectionId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query: query, body: body, events: false)
    }
    /// PATCH /api/v1/workspaces/{workspace_id}/injections/{injection_id}
    public func updateInjection(workspaceId: String, injectionId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("PATCH", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/injections/" + encodePathParam(injectionId), query: query, body: body, events: false)
    }
    /// GET /api/v1/workspaces/{workspace_id}/metrics-history
    public func getMetricsHistory(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/metrics-history", query: query, body: body, events: false)
    }
    /// POST /api/v1/workspaces/{workspace_id}/metrics-snapshot
    public func triggerMetricsSnapshot(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/metrics-snapshot", query: query, body: body, events: false)
    }
    /// POST /api/v1/workspaces/{workspace_id}/rebuild-embeddings
    public func rebuildEmbeddings(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/rebuild-embeddings", query: query, body: body, events: false)
    }
    /// POST /api/v1/workspaces/{workspace_id}/rebuild-knowledge-graph
    public func rebuildKnowledgeGraph(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/rebuild-knowledge-graph", query: query, body: body, events: false)
    }
    /// POST /api/v1/workspaces/{workspace_id}/reprocess-documents
    public func reprocessAllDocuments(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/reprocess-documents", query: query, body: body, events: false)
    }
    /// GET /api/v1/workspaces/{workspace_id}/stats
    public func getWorkspaceStats(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v1/workspaces/" + encodePathParam(workspaceId) + "/stats", query: query, body: body, events: false)
    }
    /// GET /api/v2/workspaces/{workspace_id}/jobs
    public func listWorkspaceJobs(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs", query: query, body: body, events: false)
    }
    /// POST /api/v2/workspaces/{workspace_id}/jobs
    public func createWorkspaceJob(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs", query: query, body: body, events: false)
    }
    /// GET /api/v2/workspaces/{workspace_id}/jobs/catalog
    public func listWorkspaceJobCatalog(workspaceId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/catalog", query: query, body: body, events: false)
    }
    /// GET /api/v2/workspaces/{workspace_id}/jobs/{job_id}
    public func getWorkspaceJob(workspaceId: String, jobId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/" + encodePathParam(jobId), query: query, body: body, events: false)
    }
    /// DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id}
    public func cancelWorkspaceJob(workspaceId: String, jobId: String, query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("DELETE", path: "/api/v2/workspaces/" + encodePathParam(workspaceId) + "/jobs/" + encodePathParam(jobId), query: query, body: body, events: false)
    }
    /// GET /api/version
    public func ollamaVersion(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/api/version", query: query, body: body, events: false)
    }
    /// POST /mcp
    public func mcpOpenapi(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/mcp", query: query, body: body, events: false)
    }
    /// GET /oauth/authorize
    public func oauthAuthorizeGet(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("GET", path: "/oauth/authorize", query: query, body: body, events: false)
    }
    /// POST /oauth/authorize
    public func oauthAuthorizePost(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/oauth/authorize", query: query, body: body, events: false)
    }
    /// POST /oauth/register
    public func oauthRegisterPost(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/oauth/register", query: query, body: body, events: false)
    }
    /// POST /oauth/revoke
    public func oauthRevokePost(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/oauth/revoke", query: query, body: body, events: false)
    }
    /// POST /oauth/token
    public func oauthTokenPost(query: [String: String] = [:], body: [String: String]? = nil) async throws -> String {
        try await call("POST", path: "/oauth/token", query: query, body: body, events: false)
    }
    /// GET /ws/pipeline/progress
    public func wsPipelineProgress() -> String {
        socket("/ws/pipeline/progress")
    }
    /// GET /ws/progress/{track_id}
    public func wsProgressByTrackId(trackId: String) -> String {
        socket("/ws/progress/" + encodePathParam(trackId))
    }
    private func encodePathParam(_ value: String) -> String {
        value.split(separator: "/", omittingEmptySubsequences: false).map { encodeQuery(String($0)) }.joined(separator: "/")
    }

    private func encodeQuery(_ value: String) -> String {
        var allowed = CharacterSet.alphanumerics
        allowed.insert(charactersIn: "-._~")
        return value.addingPercentEncoding(withAllowedCharacters: allowed) ?? value
    }
}
