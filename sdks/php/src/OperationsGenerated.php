<?php

declare(strict_types=1);

namespace EdgeQuake;

/** Generated OpenAPI operations. Do not edit by hand. */
class OperationsGenerated
{
    public function __construct(private readonly HttpHelper $http) {}

    private function call(string $method, string $path, ?array $query, ?array $body, bool $events): string
    {
        $text = $this->http->raw($method, $this->appendQuery($path, $query), $body);
        return $events ? implode("\n", $this->parseEvents($text)) : $text;
    }

    private function appendQuery(string $path, ?array $query): string
    {
        if (!$query) return $path;
        $qs = http_build_query(array_filter($query, fn($v) => $v !== null));
        if ($qs === '') return $path;
        return $path . (str_contains($path, '?') ? '&' : '?') . $qs;
    }

    private function parseEvents(string $text): array
    {
        $events = [];
        foreach (explode("\n\n", $text) as $block) {
            foreach (explode("\n", $block) as $line) {
                if (!str_starts_with($line, 'data:')) continue;
                $data = trim(substr($line, 5));
                if ($data !== '' && $data !== '[DONE]') $events[] = $data;
            }
        }
        return $events;
    }

    private function socket(string $path): string
    {
        $root = rtrim($this->http->baseUrl(), '/');
        if (str_starts_with($root, 'https://')) $root = 'wss://' . substr($root, 8);
        elseif (str_starts_with($root, 'http://')) $root = 'ws://' . substr($root, 7);
        return $root . $path;
    }

    /** GET /.well-known/mcp/server.json */
    public function mcp_registry_server_json(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/.well-known/mcp/server.json", $query, $body, false);
    }
    /** GET /.well-known/oauth-authorization-server */
    public function oauth_authorization_server_metadata(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/.well-known/oauth-authorization-server", $query, $body, false);
    }
    /** GET /.well-known/oauth-protected-resource */
    public function mcp_oauth_protected_resource(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/.well-known/oauth-protected-resource", $query, $body, false);
    }
    /** GET /.well-known/oauth-protected-resource/mcp */
    public function mcp_oauth_protected_resource_path(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/.well-known/oauth-protected-resource/mcp", $query, $body, false);
    }
    /** GET /.well-known/openid-configuration */
    public function openid_configuration(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/.well-known/openid-configuration", $query, $body, false);
    }
    /** POST /api/chat */
    public function ollama_chat(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/chat", $query, $body, false);
    }
    /** POST /api/generate */
    public function ollama_generate(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/generate", $query, $body, false);
    }
    /** GET /api/ps */
    public function ollama_ps(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/ps", $query, $body, false);
    }
    /** GET /api/tags */
    public function ollama_tags(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/tags", $query, $body, false);
    }
    /** POST /api/v1/admin/ann/warmup */
    public function ann_warmup(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/admin/ann/warmup", $query, $body, false);
    }
    /** GET /api/v1/admin/config/defaults */
    public function get_server_defaults(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/admin/config/defaults", $query, $body, false);
    }
    /** PATCH /api/v1/admin/config/defaults */
    public function update_server_defaults(?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/admin/config/defaults", $query, $body, false);
    }
    /** GET /api/v1/admin/entities/reconcile */
    public function entity_reconcile_plan(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/admin/entities/reconcile", $query, $body, false);
    }
    /** POST /api/v1/admin/entities/reconcile */
    public function entity_reconcile_execute(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/admin/entities/reconcile", $query, $body, false);
    }
    /** GET /api/v1/admin/identity-providers */
    public function admin_list_identity_providers(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/admin/identity-providers", $query, $body, false);
    }
    /** PUT /api/v1/admin/identity-providers/{slug} */
    public function admin_upsert_identity_provider(string $slug, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PUT", "/api/v1/admin/identity-providers/" . $this->pathParam($slug), $query, $body, false);
    }
    /** DELETE /api/v1/admin/identity-providers/{slug} */
    public function admin_delete_identity_provider(string $slug, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/admin/identity-providers/" . $this->pathParam($slug), $query, $body, false);
    }
    /** GET /api/v1/admin/migration-jobs */
    public function list_migration_jobs(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/admin/migration-jobs", $query, $body, false);
    }
    /** GET /api/v1/admin/migration-jobs/{job_id} */
    public function get_migration_job(string $job_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/admin/migration-jobs/" . $this->pathParam($job_id), $query, $body, false);
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/cancel */
    public function cancel_migration_job(string $job_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/admin/migration-jobs/" . $this->pathParam($job_id) . "/cancel", $query, $body, false);
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/pause */
    public function pause_migration_job(string $job_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/admin/migration-jobs/" . $this->pathParam($job_id) . "/pause", $query, $body, false);
    }
    /** POST /api/v1/admin/migration-jobs/{job_id}/resume */
    public function resume_migration_job(string $job_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/admin/migration-jobs/" . $this->pathParam($job_id) . "/resume", $query, $body, false);
    }
    /** GET /api/v1/admin/storage/inspect */
    public function storage_inspect(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/admin/storage/inspect", $query, $body, false);
    }
    /** POST /api/v1/admin/storage/repair */
    public function storage_repair(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/admin/storage/repair", $query, $body, false);
    }
    /** PATCH /api/v1/admin/tenants/{tenant_id}/quota */
    public function update_tenant_quota(string $tenant_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/admin/tenants/" . $this->pathParam($tenant_id) . "/quota", $query, $body, false);
    }
    /** POST /api/v1/auth/handoff */
    public function redeem_sso_handoff(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/auth/handoff", $query, $body, false);
    }
    /** POST /api/v1/auth/oidc/backchannel-logout */
    public function oidc_backchannel_logout(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/auth/oidc/backchannel-logout", $query, $body, false);
    }
    /** GET /api/v1/auth/oidc/callback */
    public function oidc_callback(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/auth/oidc/callback", $query, $body, false);
    }
    /** GET /api/v1/auth/oidc/login */
    public function oidc_login(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/auth/oidc/login", $query, $body, false);
    }
    /** GET /api/v1/auth/sso/providers */
    public function list_sso_providers(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/auth/sso/providers", $query, $body, false);
    }
    /** GET /api/v1/chunks/{chunk_id} */
    public function get_chunk_detail(string $chunk_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/chunks/" . $this->pathParam($chunk_id), $query, $body, false);
    }
    /** GET /api/v1/chunks/{chunk_id}/lineage */
    public function get_chunk_lineage(string $chunk_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/chunks/" . $this->pathParam($chunk_id) . "/lineage", $query, $body, false);
    }
    /** GET /api/v1/config/effective */
    public function get_effective_config(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/config/effective", $query, $body, false);
    }
    /** PUT /api/v1/connections/{id} */
    public function update_connection(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PUT", "/api/v1/connections/" . $this->pathParam($id), $query, $body, false);
    }
    /** DELETE /api/v1/connections/{id} */
    public function delete_connection(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/connections/" . $this->pathParam($id), $query, $body, false);
    }
    /** POST /api/v1/connections/{id}/test */
    public function test_stored_connection(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/connections/" . $this->pathParam($id) . "/test", $query, $body, false);
    }
    /** POST /api/v1/conversations/bulk/delete */
    public function bulk_delete_conversations(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/conversations/bulk/delete", $query, $body, false);
    }
    /** PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback */
    public function set_message_feedback(string $conversation_id, string $message_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/conversations/" . $this->pathParam($conversation_id) . "/messages/" . $this->pathParam($message_id) . "/feedback", $query, $body, false);
    }
    /** GET /api/v1/conversations/{id} */
    public function get_conversation(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/conversations/" . $this->pathParam($id), $query, $body, false);
    }
    /** DELETE /api/v1/conversations/{id} */
    public function delete_conversation(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/conversations/" . $this->pathParam($id), $query, $body, false);
    }
    /** PATCH /api/v1/conversations/{id} */
    public function update_conversation(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/conversations/" . $this->pathParam($id), $query, $body, false);
    }
    /** GET /api/v1/conversations/{id}/messages */
    public function list_messages(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/conversations/" . $this->pathParam($id) . "/messages", $query, $body, false);
    }
    /** POST /api/v1/conversations/{id}/messages */
    public function create_message(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/conversations/" . $this->pathParam($id) . "/messages", $query, $body, false);
    }
    /** POST /api/v1/conversations/{id}/share */
    public function share_conversation(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/conversations/" . $this->pathParam($id) . "/share", $query, $body, false);
    }
    /** DELETE /api/v1/conversations/{id}/share */
    public function unshare_conversation(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/conversations/" . $this->pathParam($id) . "/share", $query, $body, false);
    }
    /** GET /api/v1/decision/models */
    public function get_decision_models(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/decision/models", $query, $body, false);
    }
    /** GET /api/v1/decision/status */
    public function get_decision_status(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/decision/status", $query, $body, false);
    }
    /** POST /api/v1/documents/batch-delete */
    public function batch_delete_documents(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/documents/batch-delete", $query, $body, false);
    }
    /** GET /api/v1/documents/pdf/progress/stream/{track_id} */
    public function get_pdf_progress_stream(string $track_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/pdf/progress/stream/" . $this->pathParam($track_id), $query, $body, true);
    }
    /** DELETE /api/v1/documents/pdf/{pdf_id}/cancel */
    public function cancel_pdf_processing(string $pdf_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/documents/pdf/" . $this->pathParam($pdf_id) . "/cancel", $query, $body, false);
    }
    /** GET /api/v1/documents/pdf/{pdf_id}/content */
    public function get_pdf_content(string $pdf_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/pdf/" . $this->pathParam($pdf_id) . "/content", $query, $body, false);
    }
    /** GET /api/v1/documents/pdf/{pdf_id}/download */
    public function download_pdf(string $pdf_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/pdf/" . $this->pathParam($pdf_id) . "/download", $query, $body, false);
    }
    /** POST /api/v1/documents/pdf/{pdf_id}/retry */
    public function retry_pdf_processing(string $pdf_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/documents/pdf/" . $this->pathParam($pdf_id) . "/retry", $query, $body, false);
    }
    /** POST /api/v1/documents/scan */
    public function scan_directory(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/documents/scan", $query, $body, false);
    }
    /** GET /api/v1/documents/search */
    public function search_documents(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/search", $query, $body, false);
    }
    /** GET /api/v1/documents/track/{track_id} */
    public function get_track_status(string $track_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/track/" . $this->pathParam($track_id), $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id} */
    public function get_document(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id), $query, $body, false);
    }
    /** DELETE /api/v1/documents/{document_id} */
    public function delete_document(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/documents/" . $this->pathParam($document_id), $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/assets */
    public function list_document_assets(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/assets", $query, $body, false);
    }
    /** POST /api/v1/documents/{document_id}/assets/include-from-pdf */
    public function include_document_assets_from_pdf(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/documents/" . $this->pathParam($document_id) . "/assets/include-from-pdf", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/assets/{asset_id} */
    public function download_document_asset_by_id(string $document_id, string $asset_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/assets/" . $this->pathParam($asset_id), $query, $body, false);
    }
    /** POST /api/v1/documents/{document_id}/cancel */
    public function cancel_document(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/documents/" . $this->pathParam($document_id) . "/cancel", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/deletion-impact */
    public function analyze_deletion_impact(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/deletion-impact", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/download/markdown */
    public function download_document_markdown(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/download/markdown", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/download/original */
    public function download_document_original(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/download/original", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/failed-chunks */
    public function list_failed_chunks(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/failed-chunks", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/lineage */
    public function get_document_full_lineage(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/lineage", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/lineage/export */
    public function export_document_lineage(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/lineage/export", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/metadata */
    public function get_document_metadata(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/metadata", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/mm-assets/{asset_path} */
    public function download_document_mm_asset(string $document_id, string $asset_path, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/mm-assets/" . $this->pathParam($asset_path), $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/pages */
    public function list_document_pages(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/pages", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/pages/health */
    public function get_pages_health(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/pages/health", $query, $body, false);
    }
    /** POST /api/v1/documents/{document_id}/pages/reprocess */
    public function reprocess_pages(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/documents/" . $this->pathParam($document_id) . "/pages/reprocess", $query, $body, false);
    }
    /** GET /api/v1/documents/{document_id}/pages/{page_number}/layout */
    public function get_document_page_layout(string $document_id, string $page_number, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/documents/" . $this->pathParam($document_id) . "/pages/" . $this->pathParam($page_number) . "/layout", $query, $body, false);
    }
    /** POST /api/v1/documents/{document_id}/reanalyze */
    public function reanalyze_multimodal(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/documents/" . $this->pathParam($document_id) . "/reanalyze", $query, $body, false);
    }
    /** POST /api/v1/documents/{document_id}/retry-chunks */
    public function retry_failed_chunks(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/documents/" . $this->pathParam($document_id) . "/retry-chunks", $query, $body, false);
    }
    /** GET /api/v1/entities/{entity_id}/provenance */
    public function get_entity_provenance(string $entity_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/entities/" . $this->pathParam($entity_id) . "/provenance", $query, $body, false);
    }
    /** DELETE /api/v1/folders/{folder_id} */
    public function delete_folder(string $folder_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/folders/" . $this->pathParam($folder_id), $query, $body, false);
    }
    /** PATCH /api/v1/folders/{folder_id} */
    public function update_folder(string $folder_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/folders/" . $this->pathParam($folder_id), $query, $body, false);
    }
    /** GET /api/v1/graph/communities */
    public function get_graph_communities(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/graph/communities", $query, $body, false);
    }
    /** GET /api/v1/graph/entities/{entity_name} */
    public function get_entity(string $entity_name, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/graph/entities/" . $this->pathParam($entity_name), $query, $body, false);
    }
    /** PUT /api/v1/graph/entities/{entity_name} */
    public function update_entity(string $entity_name, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PUT", "/api/v1/graph/entities/" . $this->pathParam($entity_name), $query, $body, false);
    }
    /** DELETE /api/v1/graph/entities/{entity_name} */
    public function delete_entity(string $entity_name, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/graph/entities/" . $this->pathParam($entity_name), $query, $body, false);
    }
    /** GET /api/v1/graph/entities/{entity_name}/neighborhood */
    public function get_entity_neighborhood(string $entity_name, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/graph/entities/" . $this->pathParam($entity_name) . "/neighborhood", $query, $body, false);
    }
    /** GET /api/v1/graph/facets */
    public function get_graph_facets(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/graph/facets", $query, $body, false);
    }
    /** GET /api/v1/graph/nodes/{node_id} */
    public function get_node(string $node_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/graph/nodes/" . $this->pathParam($node_id), $query, $body, false);
    }
    /** GET /api/v1/graph/relationships/{relationship_id} */
    public function get_relationship(string $relationship_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/graph/relationships/" . $this->pathParam($relationship_id), $query, $body, false);
    }
    /** PUT /api/v1/graph/relationships/{relationship_id} */
    public function update_relationship(string $relationship_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PUT", "/api/v1/graph/relationships/" . $this->pathParam($relationship_id), $query, $body, false);
    }
    /** DELETE /api/v1/graph/relationships/{relationship_id} */
    public function delete_relationship(string $relationship_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/graph/relationships/" . $this->pathParam($relationship_id), $query, $body, false);
    }
    /** GET /api/v1/graph/stream */
    public function stream_graph(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/graph/stream", $query, $body, true);
    }
    /** POST /api/v1/ingestion/progress */
    public function post_ingestion_progress_batch(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/ingestion/progress", $query, $body, false);
    }
    /** GET /api/v1/ingestion/{track_id}/progress */
    public function get_ingestion_progress(string $track_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/ingestion/" . $this->pathParam($track_id) . "/progress", $query, $body, false);
    }
    /** GET /api/v1/lineage/documents/{document_id} */
    public function get_document_lineage(string $document_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/lineage/documents/" . $this->pathParam($document_id), $query, $body, false);
    }
    /** GET /api/v1/lineage/entities/{entity_name} */
    public function get_entity_lineage(string $entity_name, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/lineage/entities/" . $this->pathParam($entity_name), $query, $body, false);
    }
    /** POST /api/v1/mcp */
    public function mcp_openapi_v1(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/mcp", $query, $body, false);
    }
    /** POST /api/v1/models/discover/refresh */
    public function refresh_model_discovery(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/models/discover/refresh", $query, $body, false);
    }
    /** GET /api/v1/models/search */
    public function search_models(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/models/search", $query, $body, false);
    }
    /** GET /api/v1/models/{provider} */
    public function get_provider(string $provider, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/models/" . $this->pathParam($provider), $query, $body, false);
    }
    /** GET /api/v1/models/{provider}/{model} */
    public function get_model(string $provider, string $model, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/models/" . $this->pathParam($provider) . "/" . $this->pathParam($model), $query, $body, false);
    }
    /** POST /api/v1/parse */
    public function parse_document(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/parse", $query, $body, false);
    }
    /** GET /api/v1/parse/backends */
    public function list_parse_backends(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/parse/backends", $query, $body, false);
    }
    /** GET /api/v1/parse/jobs/{id} */
    public function get_parse_job(string $id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/parse/jobs/" . $this->pathParam($id), $query, $body, false);
    }
    /** GET /api/v1/pipeline/activity */
    public function get_pipeline_activity(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/pipeline/activity", $query, $body, false);
    }
    /** POST /api/v1/query/context */
    public function retrieve_query_context(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/query/context", $query, $body, false);
    }
    /** GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id} */
    public function get_context_artifact(string $artifact_type, string $artifact_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/query/context/artifacts/" . $this->pathParam($artifact_type) . "/" . $this->pathParam($artifact_id), $query, $body, false);
    }
    /** POST /api/v1/query/context/search */
    public function search_query_context(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/query/context/search", $query, $body, false);
    }
    /** GET /api/v1/query/context/{retrieval_id} */
    public function fetch_query_context(string $retrieval_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/query/context/" . $this->pathParam($retrieval_id), $query, $body, false);
    }
    /** GET /api/v1/settings/app-attribution */
    public function get_app_attribution_settings(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/settings/app-attribution", $query, $body, false);
    }
    /** PATCH /api/v1/settings/app-attribution */
    public function update_app_attribution(?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/settings/app-attribution", $query, $body, false);
    }
    /** GET /api/v1/settings/attribution */
    public function get_attribution_settings(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/settings/attribution", $query, $body, false);
    }
    /** GET /api/v1/settings/langfuse */
    public function get_langfuse_settings(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/settings/langfuse", $query, $body, false);
    }
    /** GET /api/v1/settings/llm-defaults */
    public function get_llm_defaults(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/settings/llm-defaults", $query, $body, false);
    }
    /** PATCH /api/v1/settings/llm-defaults */
    public function update_llm_defaults(?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/settings/llm-defaults", $query, $body, false);
    }
    /** POST /api/v1/setup/initialize */
    public function setup_initialize(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/setup/initialize", $query, $body, false);
    }
    /** GET /api/v1/setup/status */
    public function setup_status(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/setup/status", $query, $body, false);
    }
    /** GET /api/v1/shared/{share_id} */
    public function get_shared_conversation(string $share_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/shared/" . $this->pathParam($share_id), $query, $body, false);
    }
    /** GET /api/v1/tasks/{track_id} */
    public function get_task(string $track_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/tasks/" . $this->pathParam($track_id), $query, $body, false);
    }
    /** POST /api/v1/tasks/{track_id}/cancel */
    public function cancel_task(string $track_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/tasks/" . $this->pathParam($track_id) . "/cancel", $query, $body, false);
    }
    /** POST /api/v1/tasks/{track_id}/retry */
    public function retry_task(string $track_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/tasks/" . $this->pathParam($track_id) . "/retry", $query, $body, false);
    }
    /** GET /api/v1/tenants/{tenant_id} */
    public function get_tenant(string $tenant_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/tenants/" . $this->pathParam($tenant_id), $query, $body, false);
    }
    /** PUT /api/v1/tenants/{tenant_id} */
    public function update_tenant(string $tenant_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PUT", "/api/v1/tenants/" . $this->pathParam($tenant_id), $query, $body, false);
    }
    /** DELETE /api/v1/tenants/{tenant_id} */
    public function delete_tenant(string $tenant_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/tenants/" . $this->pathParam($tenant_id), $query, $body, false);
    }
    /** GET /api/v1/tenants/{tenant_id}/workspaces */
    public function list_workspaces(string $tenant_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/tenants/" . $this->pathParam($tenant_id) . "/workspaces", $query, $body, false);
    }
    /** POST /api/v1/tenants/{tenant_id}/workspaces */
    public function create_workspace(string $tenant_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/tenants/" . $this->pathParam($tenant_id) . "/workspaces", $query, $body, false);
    }
    /** GET /api/v1/tenants/{tenant_id}/workspaces/by-slug/{slug} */
    public function get_workspace_by_slug(string $tenant_id, string $slug, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/tenants/" . $this->pathParam($tenant_id) . "/workspaces/by-slug/" . $this->pathParam($slug), $query, $body, false);
    }
    /** GET /api/v1/users/{user_id} */
    public function get_user(string $user_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/users/" . $this->pathParam($user_id), $query, $body, false);
    }
    /** DELETE /api/v1/users/{user_id} */
    public function delete_user(string $user_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/users/" . $this->pathParam($user_id), $query, $body, false);
    }
    /** PATCH /api/v1/users/{user_id} */
    public function update_user(string $user_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/users/" . $this->pathParam($user_id), $query, $body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id} */
    public function get_workspace(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/workspaces/" . $this->pathParam($workspace_id), $query, $body, false);
    }
    /** PUT /api/v1/workspaces/{workspace_id} */
    public function update_workspace(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PUT", "/api/v1/workspaces/" . $this->pathParam($workspace_id), $query, $body, false);
    }
    /** DELETE /api/v1/workspaces/{workspace_id} */
    public function delete_workspace(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/workspaces/" . $this->pathParam($workspace_id), $query, $body, false);
    }
    /** PUT /api/v1/workspaces/{workspace_id}/injection */
    public function put_injection(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PUT", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/injection", $query, $body, false);
    }
    /** PUT /api/v1/workspaces/{workspace_id}/injection/file */
    public function put_injection_file(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PUT", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/injection/file", $query, $body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id}/injections */
    public function list_injections(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/injections", $query, $body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    public function get_injection(string $workspace_id, string $injection_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/injections/" . $this->pathParam($injection_id), $query, $body, false);
    }
    /** DELETE /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    public function delete_injection(string $workspace_id, string $injection_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/injections/" . $this->pathParam($injection_id), $query, $body, false);
    }
    /** PATCH /api/v1/workspaces/{workspace_id}/injections/{injection_id} */
    public function update_injection(string $workspace_id, string $injection_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("PATCH", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/injections/" . $this->pathParam($injection_id), $query, $body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id}/metrics-history */
    public function get_metrics_history(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/metrics-history", $query, $body, false);
    }
    /** POST /api/v1/workspaces/{workspace_id}/metrics-snapshot */
    public function trigger_metrics_snapshot(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/metrics-snapshot", $query, $body, false);
    }
    /** POST /api/v1/workspaces/{workspace_id}/rebuild-embeddings */
    public function rebuild_embeddings(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/rebuild-embeddings", $query, $body, false);
    }
    /** POST /api/v1/workspaces/{workspace_id}/rebuild-knowledge-graph */
    public function rebuild_knowledge_graph(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/rebuild-knowledge-graph", $query, $body, false);
    }
    /** POST /api/v1/workspaces/{workspace_id}/reprocess-documents */
    public function reprocess_all_documents(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/reprocess-documents", $query, $body, false);
    }
    /** GET /api/v1/workspaces/{workspace_id}/stats */
    public function get_workspace_stats(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v1/workspaces/" . $this->pathParam($workspace_id) . "/stats", $query, $body, false);
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs */
    public function list_workspace_jobs(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v2/workspaces/" . $this->pathParam($workspace_id) . "/jobs", $query, $body, false);
    }
    /** POST /api/v2/workspaces/{workspace_id}/jobs */
    public function create_workspace_job(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/api/v2/workspaces/" . $this->pathParam($workspace_id) . "/jobs", $query, $body, false);
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs/catalog */
    public function list_workspace_job_catalog(string $workspace_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v2/workspaces/" . $this->pathParam($workspace_id) . "/jobs/catalog", $query, $body, false);
    }
    /** GET /api/v2/workspaces/{workspace_id}/jobs/{job_id} */
    public function get_workspace_job(string $workspace_id, string $job_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/v2/workspaces/" . $this->pathParam($workspace_id) . "/jobs/" . $this->pathParam($job_id), $query, $body, false);
    }
    /** DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id} */
    public function cancel_workspace_job(string $workspace_id, string $job_id, ?array $query = null, ?array $body = null): string
    {
        return $this->call("DELETE", "/api/v2/workspaces/" . $this->pathParam($workspace_id) . "/jobs/" . $this->pathParam($job_id), $query, $body, false);
    }
    /** GET /api/version */
    public function ollama_version(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/api/version", $query, $body, false);
    }
    /** POST /mcp */
    public function mcp_openapi(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/mcp", $query, $body, false);
    }
    /** GET /oauth/authorize */
    public function oauth_authorize_get(?array $query = null, ?array $body = null): string
    {
        return $this->call("GET", "/oauth/authorize", $query, $body, false);
    }
    /** POST /oauth/authorize */
    public function oauth_authorize_post(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/oauth/authorize", $query, $body, false);
    }
    /** POST /oauth/register */
    public function oauth_register_post(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/oauth/register", $query, $body, false);
    }
    /** POST /oauth/revoke */
    public function oauth_revoke_post(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/oauth/revoke", $query, $body, false);
    }
    /** POST /oauth/token */
    public function oauth_token_post(?array $query = null, ?array $body = null): string
    {
        return $this->call("POST", "/oauth/token", $query, $body, false);
    }
    /** GET /ws/pipeline/progress */
    public function ws_pipeline_progress(): string
    {
        return $this->socket("/ws/pipeline/progress");
    }
    /** GET /ws/progress/{track_id} */
    public function ws_progress_by_track_id(string $track_id): string
    {
        return $this->socket("/ws/progress/" . $this->pathParam($track_id));
    }
    private function pathParam(string $value): string
    {
        return implode('/', array_map('rawurlencode', explode('/', $value)));
    }
}
