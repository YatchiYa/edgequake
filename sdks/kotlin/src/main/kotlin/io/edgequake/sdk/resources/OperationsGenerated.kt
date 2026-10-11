package io.edgequake.sdk.resources

import io.edgequake.sdk.internal.HttpHelper

/** Generated OpenAPI operations. Do not edit by hand. */
class OperationsGenerated(private val http: HttpHelper) {
    private fun call(method: String, path: String, query: Map<String, String?>, body: Any?, events: Boolean): String {
        val text = http.exchange(method, appendQuery(path, query), body)
        return if (events) parseEvents(text) else text
    }

    private fun appendQuery(path: String, query: Map<String, String?>): String {
        val qs = query.filterValues { it != null }.entries.joinToString("&") { "${encodeQuery(it.key)}=${encodeQuery(it.value!!)}" }
        if (qs.isEmpty()) return path
        return path + (if (path.contains('?')) "&" else "?") + qs
    }

    private fun parseEvents(text: String): String =
        text.split("\n\n").flatMap { block ->
            block.lineSequence().mapNotNull { line ->
                if (!line.startsWith("data:")) return@mapNotNull null
                line.removePrefix("data:").trim().takeIf { it.isNotEmpty() && it != "[DONE]" }
            }
        }.joinToString("\n")

    private fun socket(path: String): String {
        val root = http.config.baseUrl.trimEnd('/')
        val ws = when {
            root.startsWith("https://") -> "wss://" + root.removePrefix("https://")
            root.startsWith("http://") -> "ws://" + root.removePrefix("http://")
            else -> root
        }
        return ws + path
    }

    /** GET /.well-known/mcp/server.json */
    fun mcpRegistryServerJson(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/.well-known/mcp/server.json", query, body, false)
    }
    /** GET /.well-known/oauth-authorization-server */
    fun oauthAuthorizationServerMetadata(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/.well-known/oauth-authorization-server", query, body, false)
    }
    /** GET /.well-known/oauth-protected-resource */
    fun mcpOauthProtectedResource(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/.well-known/oauth-protected-resource", query, body, false)
    }
    /** GET /.well-known/oauth-protected-resource/mcp */
    fun mcpOauthProtectedResourcePath(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/.well-known/oauth-protected-resource/mcp", query, body, false)
    }
    /** GET /.well-known/openid-configuration */
    fun openidConfiguration(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/.well-known/openid-configuration", query, body, false)
    }
    /** POST /api/chat */
    fun ollamaChat(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/chat", query, body, false)
    }
    /** POST /api/generate */
    fun ollamaGenerate(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/generate", query, body, false)
    }
    /** GET /api/ps */
    fun ollamaPs(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/ps", query, body, false)
    }
    /** GET /api/tags */
    fun ollamaTags(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/tags", query, body, false)
    }
    /** POST /api/v1/admin/ann/warmup */
    fun annWarmup(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/admin/ann/warmup", query, body, false)
    }
    /** GET /api/v1/admin/entities/reconcile */
    fun entityReconcilePlan(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/admin/entities/reconcile", query, body, false)
    }
    /** POST /api/v1/admin/entities/reconcile */
    fun entityReconcileExecute(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/admin/entities/reconcile", query, body, false)
    }
    /** GET /api/v1/admin/identity-providers */
    fun adminListIdentityProviders(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/admin/identity-providers", query, body, false)
    }
    /** PUT /api/v1/admin/identity-providers/{slug} */
    fun adminUpsertIdentityProvider(slug: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PUT", "/api/v1/admin/identity-providers/${encodePathParam(slug)}", query, body, false)
    }
    /** DELETE /api/v1/admin/identity-providers/{slug} */
    fun adminDeleteIdentityProvider(slug: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/admin/identity-providers/${encodePathParam(slug)}", query, body, false)
    }
    /** GET /api/v1/admin/migration-jobs */
    fun listMigrationJobs(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/admin/migration-jobs", query, body, false)
    }
    /** GET /api/v1/admin/migration-jobs/{job_id} */
    fun getMigrationJob(jobId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/admin/migration-jobs/${encodePathParam(jobId)}", query, body, false)
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/cancel */
    fun cancelMigrationJob(jobId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/admin/migration-jobs/${encodePathParam(jobId)}/cancel", query, body, false)
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/pause */
    fun pauseMigrationJob(jobId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/admin/migration-jobs/${encodePathParam(jobId)}/pause", query, body, false)
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/resume */
    fun resumeMigrationJob(jobId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/admin/migration-jobs/${encodePathParam(jobId)}/resume", query, body, false)
    }
    /** GET /api/v1/admin/storage/inspect */
    fun storageInspect(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/admin/storage/inspect", query, body, false)
    }
    /** POST /api/v1/admin/storage/repair */
    fun storageRepair(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/admin/storage/repair", query, body, false)
    }
    /** PATCH /api/v1/admin/tenants/{tenant_id}/quota */
    fun updateTenantQuota(tenantId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PATCH", "/api/v1/admin/tenants/${encodePathParam(tenantId)}/quota", query, body, false)
    }
    /** DELETE /api/v1/api-keys/{key_id} */
    fun revokeApiKey(keyId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/api-keys/${encodePathParam(keyId)}", query, body, false)
    }
    /** POST /api/v1/auth/handoff */
    fun redeemSsoHandoff(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/auth/handoff", query, body, false)
    }
    /** POST /api/v1/auth/oidc/backchannel-logout */
    fun oidcBackchannelLogout(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/auth/oidc/backchannel-logout", query, body, false)
    }
    /** GET /api/v1/auth/oidc/callback */
    fun oidcCallback(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/auth/oidc/callback", query, body, false)
    }
    /** GET /api/v1/auth/oidc/login */
    fun oidcLogin(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/auth/oidc/login", query, body, false)
    }
    /** GET /api/v1/auth/sso/providers */
    fun listSsoProviders(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/auth/sso/providers", query, body, false)
    }
    /** GET /api/v1/chunks/{chunk_id} */
    fun getChunkDetail(chunkId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/chunks/${encodePathParam(chunkId)}", query, body, false)
    }
    /** GET /api/v1/chunks/{chunk_id}/lineage */
    fun getChunkLineage(chunkId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/chunks/${encodePathParam(chunkId)}/lineage", query, body, false)
    }
    /** PUT /api/v1/connections/{id} */
    fun updateConnection(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PUT", "/api/v1/connections/${encodePathParam(id)}", query, body, false)
    }
    /** DELETE /api/v1/connections/{id} */
    fun deleteConnection(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/connections/${encodePathParam(id)}", query, body, false)
    }
    /** POST /api/v1/connections/{id}/test */
    fun testStoredConnection(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/connections/${encodePathParam(id)}/test", query, body, false)
    }
    /** POST /api/v1/conversations/bulk/archive */
    fun bulkArchiveConversations(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/conversations/bulk/archive", query, body, false)
    }
    /** POST /api/v1/conversations/bulk/move */
    fun bulkMoveConversations(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/conversations/bulk/move", query, body, false)
    }
    /** PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback */
    fun setMessageFeedback(conversationId: String, messageId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PATCH", "/api/v1/conversations/${encodePathParam(conversationId)}/messages/${encodePathParam(messageId)}/feedback", query, body, false)
    }
    /** GET /api/v1/conversations/{id} */
    fun getConversation(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/conversations/${encodePathParam(id)}", query, body, false)
    }
    /** DELETE /api/v1/conversations/{id} */
    fun deleteConversation(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/conversations/${encodePathParam(id)}", query, body, false)
    }
    /** PATCH /api/v1/conversations/{id} */
    fun updateConversation(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PATCH", "/api/v1/conversations/${encodePathParam(id)}", query, body, false)
    }
    /** GET /api/v1/conversations/{id}/messages */
    fun listMessages(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/conversations/${encodePathParam(id)}/messages", query, body, false)
    }
    /** POST /api/v1/conversations/{id}/messages */
    fun createMessage(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/conversations/${encodePathParam(id)}/messages", query, body, false)
    }
    /** POST /api/v1/conversations/{id}/share */
    fun shareConversation(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/conversations/${encodePathParam(id)}/share", query, body, false)
    }
    /** DELETE /api/v1/conversations/{id}/share */
    fun unshareConversation(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/conversations/${encodePathParam(id)}/share", query, body, false)
    }
    /** GET /api/v1/decision/models */
    fun getDecisionModels(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/decision/models", query, body, false)
    }
    /** GET /api/v1/decision/status */
    fun getDecisionStatus(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/decision/status", query, body, false)
    }
    /** POST /api/v1/documents/batch-delete */
    fun batchDeleteDocuments(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/documents/batch-delete", query, body, false)
    }
    /** GET /api/v1/documents/pdf/progress/stream/{track_id} */
    fun getPdfProgressStream(trackId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/pdf/progress/stream/${encodePathParam(trackId)}", query, body, true)
    }
    /** GET /api/v1/documents/pdf/progress/{track_id} */
    fun getPdfProgress(trackId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/pdf/progress/${encodePathParam(trackId)}", query, body, false)
    }
    /** GET /api/v1/documents/pdf/{pdf_id} */
    fun getPdfStatus(pdfId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/pdf/${encodePathParam(pdfId)}", query, body, false)
    }
    /** DELETE /api/v1/documents/pdf/{pdf_id} */
    fun deletePdf(pdfId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/documents/pdf/${encodePathParam(pdfId)}", query, body, false)
    }
    /** DELETE /api/v1/documents/pdf/{pdf_id}/cancel */
    fun cancelPdfProcessing(pdfId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/documents/pdf/${encodePathParam(pdfId)}/cancel", query, body, false)
    }
    /** GET /api/v1/documents/pdf/{pdf_id}/content */
    fun getPdfContent(pdfId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/pdf/${encodePathParam(pdfId)}/content", query, body, false)
    }
    /** GET /api/v1/documents/pdf/{pdf_id}/download */
    fun downloadPdf(pdfId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/pdf/${encodePathParam(pdfId)}/download", query, body, false)
    }
    /** POST /api/v1/documents/pdf/{pdf_id}/retry */
    fun retryPdfProcessing(pdfId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/documents/pdf/${encodePathParam(pdfId)}/retry", query, body, false)
    }
    /** GET /api/v1/documents/search */
    fun searchDocuments(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/search", query, body, false)
    }
    /** GET /api/v1/documents/{document_id} */
    fun getDocument(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}", query, body, false)
    }
    /** DELETE /api/v1/documents/{document_id} */
    fun deleteDocument(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/documents/${encodePathParam(documentId)}", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/assets */
    fun listDocumentAssets(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/assets", query, body, false)
    }
    /** POST /api/v1/documents/{document_id}/assets/include-from-pdf */
    fun includeDocumentAssetsFromPdf(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/documents/${encodePathParam(documentId)}/assets/include-from-pdf", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/assets/{asset_id} */
    fun downloadDocumentAssetById(documentId: String, assetId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/assets/${encodePathParam(assetId)}", query, body, false)
    }
    /** POST /api/v1/documents/{document_id}/cancel */
    fun cancelDocument(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/documents/${encodePathParam(documentId)}/cancel", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/deletion-impact */
    fun analyzeDeletionImpact(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/deletion-impact", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/download/markdown */
    fun downloadDocumentMarkdown(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/download/markdown", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/download/original */
    fun downloadDocumentOriginal(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/download/original", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/failed-chunks */
    fun listFailedChunks(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/failed-chunks", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/lineage */
    fun getDocumentFullLineage(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/lineage", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/lineage/export */
    fun exportDocumentLineage(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/lineage/export", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/metadata */
    fun getDocumentMetadata(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/metadata", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/mm-assets/{asset_path} */
    fun downloadDocumentMmAsset(documentId: String, assetPath: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/mm-assets/${encodePathParam(assetPath)}", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/pages */
    fun listDocumentPages(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/pages", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/pages/health */
    fun getPagesHealth(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/pages/health", query, body, false)
    }
    /** POST /api/v1/documents/{document_id}/pages/reprocess */
    fun reprocessPages(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/documents/${encodePathParam(documentId)}/pages/reprocess", query, body, false)
    }
    /** GET /api/v1/documents/{document_id}/pages/{page_number}/layout */
    fun getDocumentPageLayout(documentId: String, pageNumber: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/documents/${encodePathParam(documentId)}/pages/${encodePathParam(pageNumber)}/layout", query, body, false)
    }
    /** POST /api/v1/documents/{document_id}/reanalyze */
    fun reanalyzeMultimodal(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/documents/${encodePathParam(documentId)}/reanalyze", query, body, false)
    }
    /** POST /api/v1/documents/{document_id}/retry-chunks */
    fun retryFailedChunks(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/documents/${encodePathParam(documentId)}/retry-chunks", query, body, false)
    }
    /** GET /api/v1/entities/{entity_id}/provenance */
    fun getEntityProvenance(entityId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/entities/${encodePathParam(entityId)}/provenance", query, body, false)
    }
    /** DELETE /api/v1/folders/{folder_id} */
    fun deleteFolder(folderId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/folders/${encodePathParam(folderId)}", query, body, false)
    }
    /** PATCH /api/v1/folders/{folder_id} */
    fun updateFolder(folderId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PATCH", "/api/v1/folders/${encodePathParam(folderId)}", query, body, false)
    }
    /** GET /api/v1/graph/communities */
    fun getGraphCommunities(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/graph/communities", query, body, false)
    }
    /** GET /api/v1/graph/entities/{entity_name} */
    fun getEntity(entityName: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/graph/entities/${encodePathParam(entityName)}", query, body, false)
    }
    /** PUT /api/v1/graph/entities/{entity_name} */
    fun updateEntity(entityName: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PUT", "/api/v1/graph/entities/${encodePathParam(entityName)}", query, body, false)
    }
    /** DELETE /api/v1/graph/entities/{entity_name} */
    fun deleteEntity(entityName: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/graph/entities/${encodePathParam(entityName)}", query, body, false)
    }
    /** GET /api/v1/graph/entities/{entity_name}/neighborhood */
    fun getEntityNeighborhood(entityName: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/graph/entities/${encodePathParam(entityName)}/neighborhood", query, body, false)
    }
    /** GET /api/v1/graph/facets */
    fun getGraphFacets(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/graph/facets", query, body, false)
    }
    /** GET /api/v1/graph/nodes/{node_id} */
    fun getNode(nodeId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/graph/nodes/${encodePathParam(nodeId)}", query, body, false)
    }
    /** GET /api/v1/graph/relationships/{relationship_id} */
    fun getRelationship(relationshipId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/graph/relationships/${encodePathParam(relationshipId)}", query, body, false)
    }
    /** PUT /api/v1/graph/relationships/{relationship_id} */
    fun updateRelationship(relationshipId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PUT", "/api/v1/graph/relationships/${encodePathParam(relationshipId)}", query, body, false)
    }
    /** DELETE /api/v1/graph/relationships/{relationship_id} */
    fun deleteRelationship(relationshipId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/graph/relationships/${encodePathParam(relationshipId)}", query, body, false)
    }
    /** GET /api/v1/graph/stream */
    fun streamGraph(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/graph/stream", query, body, true)
    }
    /** POST /api/v1/ingestion/progress */
    fun postIngestionProgressBatch(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/ingestion/progress", query, body, false)
    }
    /** GET /api/v1/ingestion/{track_id}/progress */
    fun getIngestionProgress(trackId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/ingestion/${encodePathParam(trackId)}/progress", query, body, false)
    }
    /** GET /api/v1/lineage/documents/{document_id} */
    fun getDocumentLineage(documentId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/lineage/documents/${encodePathParam(documentId)}", query, body, false)
    }
    /** GET /api/v1/lineage/entities/{entity_name} */
    fun getEntityLineage(entityName: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/lineage/entities/${encodePathParam(entityName)}", query, body, false)
    }
    /** POST /api/v1/mcp */
    fun mcpOpenapiV1(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/mcp", query, body, false)
    }
    /** POST /api/v1/models/discover/refresh */
    fun refreshModelDiscovery(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/models/discover/refresh", query, body, false)
    }
    /** GET /api/v1/models/embedding */
    fun listEmbeddingModels(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/models/embedding", query, body, false)
    }
    /** GET /api/v1/models/llm */
    fun listLlmModels(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/models/llm", query, body, false)
    }
    /** GET /api/v1/models/search */
    fun searchModels(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/models/search", query, body, false)
    }
    /** GET /api/v1/models/{provider} */
    fun getProvider(provider: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/models/${encodePathParam(provider)}", query, body, false)
    }
    /** GET /api/v1/models/{provider}/{model} */
    fun getModel(provider: String, model: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/models/${encodePathParam(provider)}/${encodePathParam(model)}", query, body, false)
    }
    /** POST /api/v1/parse */
    fun parseDocument(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/parse", query, body, false)
    }
    /** GET /api/v1/parse/backends */
    fun listParseBackends(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/parse/backends", query, body, false)
    }
    /** GET /api/v1/parse/jobs/{id} */
    fun getParseJob(id: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/parse/jobs/${encodePathParam(id)}", query, body, false)
    }
    /** GET /api/v1/pipeline/activity */
    fun getPipelineActivity(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/pipeline/activity", query, body, false)
    }
    /** GET /api/v1/pipeline/costs/pricing */
    fun getModelPricing(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/pipeline/costs/pricing", query, body, false)
    }
    /** POST /api/v1/query/context */
    fun retrieveQueryContext(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/query/context", query, body, false)
    }
    /** GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id} */
    fun getContextArtifact(artifactType: String, artifactId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/query/context/artifacts/${encodePathParam(artifactType)}/${encodePathParam(artifactId)}", query, body, false)
    }
    /** POST /api/v1/query/context/search */
    fun searchQueryContext(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/query/context/search", query, body, false)
    }
    /** GET /api/v1/query/context/{retrieval_id} */
    fun fetchQueryContext(retrievalId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/query/context/${encodePathParam(retrievalId)}", query, body, false)
    }
    /** GET /api/v1/settings/app-attribution */
    fun getAppAttributionSettings(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/settings/app-attribution", query, body, false)
    }
    /** PATCH /api/v1/settings/app-attribution */
    fun updateAppAttribution(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PATCH", "/api/v1/settings/app-attribution", query, body, false)
    }
    /** GET /api/v1/settings/attribution */
    fun getAttributionSettings(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/settings/attribution", query, body, false)
    }
    /** GET /api/v1/settings/langfuse */
    fun getLangfuseSettings(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/settings/langfuse", query, body, false)
    }
    /** GET /api/v1/settings/llm-defaults */
    fun getLlmDefaults(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/settings/llm-defaults", query, body, false)
    }
    /** PATCH /api/v1/settings/llm-defaults */
    fun updateLlmDefaults(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PATCH", "/api/v1/settings/llm-defaults", query, body, false)
    }
    /** POST /api/v1/setup/initialize */
    fun setupInitialize(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/setup/initialize", query, body, false)
    }
    /** GET /api/v1/setup/status */
    fun setupStatus(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/setup/status", query, body, false)
    }
    /** GET /api/v1/tasks/{track_id} */
    fun getTask(trackId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/tasks/${encodePathParam(trackId)}", query, body, false)
    }
    /** POST /api/v1/tasks/{track_id}/cancel */
    fun cancelTask(trackId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/tasks/${encodePathParam(trackId)}/cancel", query, body, false)
    }
    /** POST /api/v1/tasks/{track_id}/retry */
    fun retryTask(trackId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/tasks/${encodePathParam(trackId)}/retry", query, body, false)
    }
    /** GET /api/v1/tenants/{tenant_id}/workspaces/by-slug/{slug} */
    fun getWorkspaceBySlug(tenantId: String, slug: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/tenants/${encodePathParam(tenantId)}/workspaces/by-slug/${encodePathParam(slug)}", query, body, false)
    }
    /** GET /api/v1/users/{user_id} */
    fun getUser(userId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/users/${encodePathParam(userId)}", query, body, false)
    }
    /** DELETE /api/v1/users/{user_id} */
    fun deleteUser(userId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/users/${encodePathParam(userId)}", query, body, false)
    }
    /** PATCH /api/v1/users/{user_id} */
    fun updateUser(userId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PATCH", "/api/v1/users/${encodePathParam(userId)}", query, body, false)
    }
    /** GET /api/v1/workspaces/{workspace_id} */
    fun getWorkspace(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/workspaces/${encodePathParam(workspaceId)}", query, body, false)
    }
    /** PUT /api/v1/workspaces/{workspace_id} */
    fun updateWorkspace(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PUT", "/api/v1/workspaces/${encodePathParam(workspaceId)}", query, body, false)
    }
    /** DELETE /api/v1/workspaces/{workspace_id} */
    fun deleteWorkspace(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/workspaces/${encodePathParam(workspaceId)}", query, body, false)
    }
    /** PUT /api/v1/workspaces/{workspace_id}/injection */
    fun putInjection(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PUT", "/api/v1/workspaces/${encodePathParam(workspaceId)}/injection", query, body, false)
    }
    /** PUT /api/v1/workspaces/{workspace_id}/injection/file */
    fun putInjectionFile(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PUT", "/api/v1/workspaces/${encodePathParam(workspaceId)}/injection/file", query, body, false)
    }
    /** GET /api/v1/workspaces/{workspace_id}/injections */
    fun listInjections(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/workspaces/${encodePathParam(workspaceId)}/injections", query, body, false)
    }
    /** GET /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    fun getInjection(workspaceId: String, injectionId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/workspaces/${encodePathParam(workspaceId)}/injections/${encodePathParam(injectionId)}", query, body, false)
    }
    /** DELETE /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    fun deleteInjection(workspaceId: String, injectionId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v1/workspaces/${encodePathParam(workspaceId)}/injections/${encodePathParam(injectionId)}", query, body, false)
    }
    /** PATCH /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    fun updateInjection(workspaceId: String, injectionId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("PATCH", "/api/v1/workspaces/${encodePathParam(workspaceId)}/injections/${encodePathParam(injectionId)}", query, body, false)
    }
    /** GET /api/v1/workspaces/{workspace_id}/metrics-history */
    fun getMetricsHistory(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/workspaces/${encodePathParam(workspaceId)}/metrics-history", query, body, false)
    }
    /** POST /api/v1/workspaces/{workspace_id}/metrics-snapshot */
    fun triggerMetricsSnapshot(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/workspaces/${encodePathParam(workspaceId)}/metrics-snapshot", query, body, false)
    }
    /** POST /api/v1/workspaces/{workspace_id}/rebuild-embeddings */
    fun rebuildEmbeddings(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/workspaces/${encodePathParam(workspaceId)}/rebuild-embeddings", query, body, false)
    }
    /** POST /api/v1/workspaces/{workspace_id}/rebuild-knowledge-graph */
    fun rebuildKnowledgeGraph(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/workspaces/${encodePathParam(workspaceId)}/rebuild-knowledge-graph", query, body, false)
    }
    /** POST /api/v1/workspaces/{workspace_id}/reprocess-documents */
    fun reprocessAllDocuments(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v1/workspaces/${encodePathParam(workspaceId)}/reprocess-documents", query, body, false)
    }
    /** GET /api/v1/workspaces/{workspace_id}/stats */
    fun getWorkspaceStats(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v1/workspaces/${encodePathParam(workspaceId)}/stats", query, body, false)
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs */
    fun listWorkspaceJobs(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v2/workspaces/${encodePathParam(workspaceId)}/jobs", query, body, false)
    }
    /** POST /api/v2/workspaces/{workspace_id}/jobs */
    fun createWorkspaceJob(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/api/v2/workspaces/${encodePathParam(workspaceId)}/jobs", query, body, false)
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs/catalog */
    fun listWorkspaceJobCatalog(workspaceId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v2/workspaces/${encodePathParam(workspaceId)}/jobs/catalog", query, body, false)
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs/{job_id} */
    fun getWorkspaceJob(workspaceId: String, jobId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/v2/workspaces/${encodePathParam(workspaceId)}/jobs/${encodePathParam(jobId)}", query, body, false)
    }
    /** DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id} */
    fun cancelWorkspaceJob(workspaceId: String, jobId: String, query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("DELETE", "/api/v2/workspaces/${encodePathParam(workspaceId)}/jobs/${encodePathParam(jobId)}", query, body, false)
    }
    /** GET /api/version */
    fun ollamaVersion(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/api/version", query, body, false)
    }
    /** POST /mcp */
    fun mcpOpenapi(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/mcp", query, body, false)
    }
    /** GET /oauth/authorize */
    fun oauthAuthorizeGet(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("GET", "/oauth/authorize", query, body, false)
    }
    /** POST /oauth/authorize */
    fun oauthAuthorizePost(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/oauth/authorize", query, body, false)
    }
    /** POST /oauth/register */
    fun oauthRegisterPost(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/oauth/register", query, body, false)
    }
    /** POST /oauth/revoke */
    fun oauthRevokePost(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/oauth/revoke", query, body, false)
    }
    /** POST /oauth/token */
    fun oauthTokenPost(query: Map<String, String?> = emptyMap(), body: Any? = null): String {
        return call("POST", "/oauth/token", query, body, false)
    }
    /** GET /ws/pipeline/progress */
    fun wsPipelineProgress(): String {
        return socket("/ws/pipeline/progress")
    }
    /** GET /ws/progress/{track_id} */
    fun wsProgressByTrackId(trackId: String): String {
        return socket("/ws/progress/${encodePathParam(trackId)}")
    }
    private fun encodePathParam(value: String): String =
        value.split("/").joinToString("/") { java.net.URLEncoder.encode(it, Charsets.UTF_8).replace("+", "%20") }

    private fun encodeQuery(value: String): String =
        java.net.URLEncoder.encode(value, Charsets.UTF_8)
}
