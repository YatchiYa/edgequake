using System.Text.Json;
using System.Text.Json.Serialization;
using System.Collections.Generic;

namespace EdgeQuakeSDK;

// WHY: Each service maps 1:1 to an API resource for discoverability.
// OODA-36: Enhanced with complete API coverage.

public class HealthService(HttpHelper http)
{
    public Task<HealthResponse> CheckAsync() => http.GetAsync<HealthResponse>("/health");
    public Task<ReadinessResponse> ReadyAsync() => http.GetAsync<ReadinessResponse>("/ready");
    public Task<LivenessResponse> LiveAsync() => http.GetAsync<LivenessResponse>("/live");
    public Task<MetricsResponse> MetricsAsync() => http.GetAsync<MetricsResponse>("/metrics");
}

public class DocumentService(HttpHelper http)
{
    public Task<DocumentListResponse> ListAsync(int page = 1, int pageSize = 20) =>
        http.GetAsync<DocumentListResponse>($"/api/v1/documents?page={page}&page_size={pageSize}");

    public Task<DocumentDetailResponse> GetAsync(string id) =>
        http.GetAsync<DocumentDetailResponse>($"/api/v1/documents/{id}");

    /// <summary>GET /api/v1/documents/track/{track_id} — processing status for an upload track.</summary>
    public Task<TrackStatusResponse> TrackAsync(string trackId) =>
        http.GetAsync<TrackStatusResponse>($"/api/v1/documents/track/{trackId}");

    /// <summary>POST /api/v1/documents — text upload. Parameters are <c>(title, content)</c> for historical SDK compatibility; JSON sends <c>content</c> + <c>title</c> per API.</summary>
    public Task<UploadResponse> UploadTextAsync(string title, string content) =>
        http.PostAsync<UploadResponse>("/api/v1/documents", new { content, title });

    public Task<Dictionary<string, object>> UploadBatchAsync(IEnumerable<string> filePaths) =>
        http.UploadManyAsync<Dictionary<string, object>>("/api/v1/documents/upload/batch", filePaths, "files");

    public Task<Dictionary<string, object>> UploadPdfBatchAsync(IEnumerable<string> filePaths) =>
        http.UploadManyAsync<Dictionary<string, object>>("/api/v1/documents/pdf/batch", filePaths, "files");

    /// <summary>POST /api/v1/documents/reprocess — reprocess failed documents (workspace-scoped via headers).</summary>
    public Task<UploadResponse> ReprocessFailedAsync() =>
        http.PostAsync<UploadResponse>("/api/v1/documents/reprocess", null);

    public Task<UploadResponse> RecoverStuckAsync() =>
        http.PostAsync<UploadResponse>("/api/v1/documents/recover-stuck", null);

    /// <summary>WHY: DELETE returns 204 No Content — no body to deserialize.</summary>
    public Task DeleteAsync(string id) =>
        http.DeleteNoContentAsync($"/api/v1/documents/{id}");
}

/// <summary>Admin API — tenant quotas and server defaults (<c>/api/v1/admin/*</c>).</summary>
public class AdminService(HttpHelper http)
{
    public Task<UpdateTenantQuotaResponse> PatchTenantQuotaAsync(string tenantId, int maxWorkspaces) =>
        http.PatchAsync<UpdateTenantQuotaResponse>($"/api/v1/admin/tenants/{tenantId}/quota",
            new { max_workspaces = maxWorkspaces });

    public Task<ServerDefaultsResponse> GetServerDefaultsAsync() =>
        http.GetAsync<ServerDefaultsResponse>("/api/v1/admin/config/defaults");

    public Task<ServerDefaultsResponse> PatchServerDefaultsAsync(int defaultMaxWorkspaces) =>
        http.PatchAsync<ServerDefaultsResponse>("/api/v1/admin/config/defaults",
            new { default_max_workspaces = defaultMaxWorkspaces });
}

/// <summary>GET /api/v1/config/effective — merged runtime configuration.</summary>
public class EffectiveConfigService(HttpHelper http)
{
    public async Task<JsonDocument> GetAsync()
    {
        var raw = await http.GetRawAsync("/api/v1/config/effective");
        return JsonDocument.Parse(raw);
    }
}

public class EntityService(HttpHelper http)
{
    public Task<EntityListResponse> ListAsync(int page = 1, int pageSize = 20) =>
        http.GetAsync<EntityListResponse>($"/api/v1/graph/entities?page={page}&page_size={pageSize}");

    public Task<EntityDetailResponse> GetAsync(string name) =>
        http.GetAsync<EntityDetailResponse>($"/api/v1/graph/entities/{name}");

    public Task<EntityNeighborhoodResponse> NeighborhoodAsync(string name, int depth = 1) =>
        http.GetAsync<EntityNeighborhoodResponse>($"/api/v1/graph/entities/{name}/neighborhood?depth={depth}");

    public Task<CreateEntityResponse> CreateAsync(string entityName, string entityType, string description, string sourceId) =>
        http.PostAsync<CreateEntityResponse>("/api/v1/graph/entities",
            new { entity_name = entityName, entity_type = entityType, description, source_id = sourceId });

    public Task<MergeEntitiesResponse> MergeAsync(string sourceName, string targetName) =>
        http.PostAsync<MergeEntitiesResponse>("/api/v1/graph/entities/merge",
            new { source_name = sourceName, target_name = targetName });

    public Task<EntityExistsResponse> ExistsAsync(string entityName) =>
        http.GetAsync<EntityExistsResponse>(
            $"/api/v1/graph/entities/exists?entity_name={Uri.EscapeDataString(entityName)}");

    public Task<EntityDeleteResponse> DeleteAsync(string name) =>
        http.DeleteAsync<EntityDeleteResponse>($"/api/v1/graph/entities/{name}?confirm=true");
}

public class RelationshipService(HttpHelper http)
{
    public Task<RelationshipListResponse> ListAsync(int page = 1, int pageSize = 20) =>
        http.GetAsync<RelationshipListResponse>($"/api/v1/graph/relationships?page={page}&page_size={pageSize}");

    public Task<RelationshipDetailResponse> GetAsync(string relationshipId) =>
        http.GetAsync<RelationshipDetailResponse>(
            $"/api/v1/graph/relationships/{Uri.EscapeDataString(relationshipId)}");

    public Task<CreateRelationshipResponse> CreateAsync(
        string srcId, string tgtId, string keywords, string description,
        string sourceId = "manual_entry", double weight = 0.8) =>
        http.PostAsync<CreateRelationshipResponse>("/api/v1/graph/relationships",
            new { src_id = srcId, tgt_id = tgtId, keywords, description, source_id = sourceId, weight });

    public Task DeleteAsync(string relationshipId) =>
        http.DeleteNoContentAsync($"/api/v1/graph/relationships/{Uri.EscapeDataString(relationshipId)}");
}

public class GraphService(HttpHelper http)
{
    public Task<GraphResponse> GetAsync() => http.GetAsync<GraphResponse>("/api/v1/graph");

    public Task<SearchResponse> SearchAsync(string query) =>
        http.GetAsync<SearchResponse>($"/api/v1/graph/nodes/search?q={Uri.EscapeDataString(query)}");

    public Task<LabelSearchResponse> LabelSearchAsync(string query) =>
        http.GetAsync<LabelSearchResponse>($"/api/v1/graph/labels/search?q={Uri.EscapeDataString(query)}");

    public Task<PopularLabelsResponse> PopularLabelsAsync(int limit = 10) =>
        http.GetAsync<PopularLabelsResponse>($"/api/v1/graph/labels/popular?limit={limit}");

    public Task<BatchDegreesResponse> BatchDegreesAsync(string[] nodeIds) =>
        http.PostAsync<BatchDegreesResponse>("/api/v1/graph/degrees/batch", new { node_ids = nodeIds });
}

public class QueryService(HttpHelper http)
{
    public Task<QueryResponse> ExecuteAsync(string query, string? mode = null, int? maxResults = null, bool? enableRerank = null, string? systemPrompt = null)
    {
        var body = new Dictionary<string, object?> { ["query"] = query };
        if (mode != null) body["mode"] = mode;
        if (maxResults != null) body["max_results"] = maxResults;
        if (enableRerank != null) body["enable_rerank"] = enableRerank;
        if (systemPrompt != null) body["system_prompt"] = systemPrompt;
        return http.PostAsync<QueryResponse>("/api/v1/query", body);
    }

    public Task<string> StreamAsync(string query, string? mode = null, string? systemPrompt = null)
    {
        var body = new Dictionary<string, object?> { ["query"] = query, ["stream"] = true };
        if (mode != null) body["mode"] = mode;
        if (systemPrompt != null) body["system_prompt"] = systemPrompt;
        return http.PostRawAsync("/api/v1/query/stream", body);
    }
}

public class ChatService(HttpHelper http)
{
    public Task<ChatCompletionResponse> CompletionsAsync(string message, string mode = "hybrid", bool stream = false, string? systemPrompt = null, int? topK = null)
    {
        var body = new Dictionary<string, object?>
        {
            ["message"] = message,
            ["mode"] = mode,
            ["stream"] = stream,
            ["system_prompt"] = systemPrompt,
        };
        if (topK != null) body["top_k"] = topK;
        return http.PostAsync<ChatCompletionResponse>("/api/v1/chat/completions", body);
    }

    public Task<ChatCompletionResponse> CompletionsWithConversationAsync(string conversationId, string message, string mode = "hybrid", string? systemPrompt = null) =>
        http.PostAsync<ChatCompletionResponse>("/api/v1/chat/completions",
            new { conversation_id = conversationId, message, mode, system_prompt = systemPrompt });

    public Task<string> StreamAsync(string message, string mode = "hybrid", string? systemPrompt = null) =>
        http.PostRawAsync("/api/v1/chat/completions/stream", new { message, mode, stream = true, system_prompt = systemPrompt });
}

public class TenantService(HttpHelper http)
{
    public Task<TenantListResponse> ListAsync() => http.GetAsync<TenantListResponse>("/api/v1/tenants");

    public Task<TenantInfo> GetAsync(string id) =>
        http.GetAsync<TenantInfo>($"/api/v1/tenants/{id}");

    public Task<TenantInfo> CreateAsync(string name, string displayName) =>
        http.PostAsync<TenantInfo>("/api/v1/tenants", new { name, display_name = displayName });

    public Task<TenantInfo> UpdateAsync(string id, string displayName) =>
        http.PutAsync<TenantInfo>($"/api/v1/tenants/{id}", new { display_name = displayName });

    public Task DeleteAsync(string id) =>
        http.DeleteNoContentAsync($"/api/v1/tenants/{id}");
}

public class UserService(HttpHelper http)
{
    public Task<UserListResponse> ListAsync() => http.GetAsync<UserListResponse>("/api/v1/users");

    public Task<UserInfo> GetAsync(string id) =>
        http.GetAsync<UserInfo>($"/api/v1/users/{id}");

    public Task<UserInfo> CreateAsync(string email, string name, string role = "user") =>
        http.PostAsync<UserInfo>("/api/v1/users", new { email, name, role });

    public Task<UserInfo> UpdateAsync(string id, string name, string role) =>
        http.PatchAsync<UserInfo>($"/api/v1/users/{id}", new { name, role });

    public Task DeleteAsync(string id) =>
        http.DeleteNoContentAsync($"/api/v1/users/{id}");
}

public class ApiKeyService(HttpHelper http)
{
    public Task<ApiKeyListResponse> ListAsync() => http.GetAsync<ApiKeyListResponse>("/api/v1/api-keys");

    public Task<CreateApiKeyResponse> CreateAsync(string name, string[] scopes) =>
        http.PostAsync<CreateApiKeyResponse>("/api/v1/api-keys", new { name, scopes });

    public Task RevokeAsync(string id) =>
        http.DeleteNoContentAsync($"/api/v1/api-keys/{id}");
}

public class TaskService(HttpHelper http)
{
    public Task<TaskListResponse> ListAsync() => http.GetAsync<TaskListResponse>("/api/v1/tasks");

    public Task<TaskInfo> GetAsync(string id) =>
        http.GetAsync<TaskInfo>($"/api/v1/tasks/{id}");

    public Task<StatusResponse> CancelAsync(string id) =>
        http.PostAsync<StatusResponse>($"/api/v1/tasks/{id}/cancel", null);

    public Task<TaskInfo> RetryAsync(string id) =>
        http.PostAsync<TaskInfo>($"/api/v1/tasks/{id}/retry", null);
}

public class PipelineService(HttpHelper http)
{
    public Task<PipelineStatusResponse> StatusAsync() =>
        http.GetAsync<PipelineStatusResponse>("/api/v1/pipeline/status");

    public Task<QueueMetricsResponse> QueueMetricsAsync() =>
        http.GetAsync<QueueMetricsResponse>("/api/v1/pipeline/queue-metrics");

    public Task<StatusResponse> CancelAsync(string documentId) =>
        http.PostAsync<StatusResponse>("/api/v1/pipeline/cancel", new { document_id = documentId });

    public Task<CostEstimateResponse> CostEstimateAsync(int contentLength, string? model = null) =>
        http.PostAsync<CostEstimateResponse>("/api/v1/pipeline/costs/estimate", new { content_length = contentLength, model });
}

public class ModelService(HttpHelper http)
{
    public Task<ProviderCatalog> CatalogAsync() =>
        http.GetAsync<ProviderCatalog>("/api/v1/models");

    public Task<ModelInfo> GetAsync(string modelId) =>
        http.GetAsync<ModelInfo>($"/api/v1/models/{modelId}");

    public async Task<List<ProviderHealthInfo>> HealthAsync()
    {
        var raw = await http.GetRawAsync("/api/v1/models/health");
        return JsonSerializer.Deserialize<List<ProviderHealthInfo>>(raw, HttpHelper.JsonOptions)
            ?? new List<ProviderHealthInfo>();
    }

    public Task<ProviderStatus> ProviderStatusAsync() =>
        http.GetAsync<ProviderStatus>("/api/v1/settings/provider/status");

    public Task<ProviderListResponse> ProvidersAsync() =>
        http.GetAsync<ProviderListResponse>("/api/v1/settings/providers");
}

public class CostService(HttpHelper http)
{
    public Task<CostSummary> SummaryAsync() =>
        http.GetAsync<CostSummary>("/api/v1/costs/summary");

    public Task<CostHistoryResponse> HistoryAsync(int page = 1, int pageSize = 20) =>
        http.GetAsync<CostHistoryResponse>($"/api/v1/costs/history?page={page}&page_size={pageSize}");

    public Task<BudgetInfo> BudgetAsync() =>
        http.GetAsync<BudgetInfo>("/api/v1/costs/budget");

    public Task<BudgetInfo> SetBudgetAsync(double limit, string period = "monthly") =>
        http.PatchAsync<BudgetInfo>("/api/v1/costs/budget", new { limit, period });
}

public class ConversationService(HttpHelper http)
{
    /// <summary>WHY: GET /api/v1/conversations returns {"items":[...]} wrapper.</summary>
    public async Task<List<ConversationInfo>> ListAsync()
    {
        var wrapper = await http.GetAsync<ConversationListResponse>("/api/v1/conversations");
        return wrapper.Items ?? new List<ConversationInfo>();
    }

    public Task<ConversationInfo> CreateAsync(string title) =>
        http.PostAsync<ConversationInfo>("/api/v1/conversations", new { title });

    public Task<ConversationDetail> GetAsync(string id) =>
        http.GetAsync<ConversationDetail>($"/api/v1/conversations/{id}");

    public Task<ConversationInfo> UpdateAsync(string id, string title, bool isPinned = false) =>
        http.PatchAsync<ConversationInfo>($"/api/v1/conversations/{id}", new { title, is_pinned = isPinned });

    /// <summary>WHY: DELETE returns 204 No Content — no body to deserialize.</summary>
    public Task DeleteAsync(string id) =>
        http.DeleteNoContentAsync($"/api/v1/conversations/{id}");

    public Task<BulkDeleteResponse> BulkDeleteAsync(List<string> ids) =>
        http.PostAsync<BulkDeleteResponse>(
            "/api/v1/conversations/bulk/delete",
            new Dictionary<string, object> { ["conversation_ids"] = ids });

    public Task<MessageListResponse> MessagesAsync(string conversationId) =>
        http.GetAsync<MessageListResponse>($"/api/v1/conversations/{conversationId}/messages");

    public Task<ConversationMessage> AddMessageAsync(string conversationId, string role, string content) =>
        http.PostAsync<ConversationMessage>($"/api/v1/conversations/{conversationId}/messages",
            new { role, content });

    public Task DeleteMessageAsync(string messageId) =>
        http.DeleteNoContentAsync($"/api/v1/messages/{messageId}");

    public Task<ShareLinkResponse> ShareAsync(string id) =>
        http.PostAsync<ShareLinkResponse>($"/api/v1/conversations/{id}/share", null);

    public Task<ImportResponse> ImportAsync(List<ConversationImport> conversations) =>
        http.PostAsync<ImportResponse>("/api/v1/conversations/import", new { conversations });
}

public class FolderService(HttpHelper http)
{
    public Task<List<FolderInfo>> ListAsync() =>
        http.GetAsync<List<FolderInfo>>("/api/v1/folders");

    public Task<FolderInfo> CreateAsync(string name) =>
        http.PostAsync<FolderInfo>("/api/v1/folders", new { name });

    public Task<FolderInfo> UpdateAsync(string id, string name) =>
        http.PatchAsync<FolderInfo>($"/api/v1/folders/{id}", new { name });

    /// <summary>WHY: DELETE returns 204 No Content — no body to deserialize.</summary>
    public Task DeleteAsync(string id) =>
        http.DeleteNoContentAsync($"/api/v1/folders/{id}");
}

// ── Auth Service (OODA-36) ─────────────────────────────────────

public class AuthService(HttpHelper http)
{
    public Task<AuthTokenResponse> LoginAsync(string email, string password) =>
        http.PostAsync<AuthTokenResponse>("/api/v1/auth/login", new { email, password });

    public Task LogoutAsync() =>
        http.PostRawAsync("/api/v1/auth/logout", null);

    public Task<AuthTokenResponse> RefreshAsync(string refreshToken) =>
        http.PostAsync<AuthTokenResponse>("/api/v1/auth/refresh", new { refresh_token = refreshToken });

    public Task<AuthUserResponse> MeAsync() =>
        http.GetAsync<AuthUserResponse>("/api/v1/auth/me");
}

// ── Workspace Service (OODA-36) ────────────────────────────────

public class WorkspaceService(HttpHelper http)
{
    public Task<WorkspaceInfo> GetAsync(string id) =>
        http.GetAsync<WorkspaceInfo>($"/api/v1/workspaces/{id}");

    public Task<WorkspaceInfo> UpdateAsync(string id, string name, string description) =>
        http.PutAsync<WorkspaceInfo>($"/api/v1/workspaces/{id}", new { name, description });

    public Task DeleteAsync(string id) =>
        http.DeleteNoContentAsync($"/api/v1/workspaces/{id}");

    public Task<WorkspaceStatsResponse> StatsAsync(string id) =>
        http.GetAsync<WorkspaceStatsResponse>($"/api/v1/workspaces/{id}/stats");
}

// ── Shared Service (OODA-36) ───────────────────────────────────

public class SharedService(HttpHelper http)
{
    public Task<SharedLinkResponse> GetLinkAsync(string shareId) =>
        http.GetAsync<SharedLinkResponse>($"/api/v1/shared/{shareId}");
}
