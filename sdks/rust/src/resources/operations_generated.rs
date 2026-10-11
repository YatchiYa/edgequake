//! Generated OpenAPI operations. Do not edit by hand.
//!
//! Regenerate: `python3 sdks/tools/sdk_coverage.py --generate`

use std::collections::HashMap;

use reqwest::Method;

use crate::client::EdgeQuakeClient;
use crate::error::Result;

pub struct OperationsResource<'a> {
    pub(crate) client: &'a EdgeQuakeClient,
}

impl<'a> OperationsResource<'a> {
    async fn call(
        &self,
        method: Method,
        mut path: String,
        query: Option<&HashMap<String, String>>,
        body: Option<&serde_json::Value>,
        events: bool,
    ) -> Result<String> {
        if let Some(query) = query {
            let qs: String = query
                .iter()
                .map(|(k, v)| format!("{}={}", encode_query_param(k), encode_query_param(v)))
                .collect::<Vec<_>>()
                .join("&");
            if !qs.is_empty() {
                path.push(if path.contains('?') { '&' } else { '?' });
                path.push_str(&qs);
            }
        }
        let text = self.client.exchange_text(method, &path, body).await?;
        if events { Ok(parse_events(&text)) } else { Ok(text) }
    }

    /// GET /.well-known/mcp/server.json
    pub async fn mcp_registry_server_json(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/.well-known/mcp/server.json");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /.well-known/oauth-authorization-server
    pub async fn oauth_authorization_server_metadata(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/.well-known/oauth-authorization-server");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /.well-known/oauth-protected-resource
    pub async fn mcp_oauth_protected_resource(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/.well-known/oauth-protected-resource");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /.well-known/oauth-protected-resource/mcp
    pub async fn mcp_oauth_protected_resource_path(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/.well-known/oauth-protected-resource/mcp");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /.well-known/openid-configuration
    pub async fn openid_configuration(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/.well-known/openid-configuration");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/chat
    pub async fn ollama_chat(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/chat");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /api/generate
    pub async fn ollama_generate(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/generate");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/ps
    pub async fn ollama_ps(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/ps");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/tags
    pub async fn ollama_tags(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/tags");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/admin/ann/warmup
    pub async fn ann_warmup(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/ann/warmup");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/admin/entities/reconcile
    pub async fn entity_reconcile_plan(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/entities/reconcile");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/admin/entities/reconcile
    pub async fn entity_reconcile_execute(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/entities/reconcile");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/admin/identity-providers
    pub async fn admin_list_identity_providers(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/identity-providers");
        self.call(Method::GET, path, query, body, false).await
    }
    /// PUT /api/v1/admin/identity-providers/{slug}
    pub async fn admin_upsert_identity_provider(&self, slug: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/identity-providers/");
        path.push_str(&encode_path_param(slug));
        self.call(Method::PUT, path, query, body, false).await
    }
    /// DELETE /api/v1/admin/identity-providers/{slug}
    pub async fn admin_delete_identity_provider(&self, slug: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/identity-providers/");
        path.push_str(&encode_path_param(slug));
        self.call(Method::DELETE, path, query, body, false).await
    }
    /// GET /api/v1/admin/migration-jobs
    pub async fn list_migration_jobs(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/migration-jobs");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/admin/migration-jobs/{job_id}
    pub async fn get_migration_job(&self, job_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/migration-jobs/");
        path.push_str(&encode_path_param(job_id));
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/admin/migration-jobs/{job_id}/cancel
    pub async fn cancel_migration_job(&self, job_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/migration-jobs/");
        path.push_str(&encode_path_param(job_id));
        path.push_str("/cancel");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /api/v1/admin/migration-jobs/{job_id}/pause
    pub async fn pause_migration_job(&self, job_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/migration-jobs/");
        path.push_str(&encode_path_param(job_id));
        path.push_str("/pause");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /api/v1/admin/migration-jobs/{job_id}/resume
    pub async fn resume_migration_job(&self, job_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/migration-jobs/");
        path.push_str(&encode_path_param(job_id));
        path.push_str("/resume");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/admin/storage/inspect
    pub async fn storage_inspect(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/storage/inspect");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/admin/storage/repair
    pub async fn storage_repair(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/admin/storage/repair");
        self.call(Method::POST, path, query, body, false).await
    }
    /// DELETE /api/v1/api-keys/{key_id}
    pub async fn revoke_api_key(&self, key_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/api-keys/");
        path.push_str(&encode_path_param(key_id));
        self.call(Method::DELETE, path, query, body, false).await
    }
    /// POST /api/v1/auth/handoff
    pub async fn redeem_sso_handoff(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/auth/handoff");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /api/v1/auth/oidc/backchannel-logout
    pub async fn oidc_backchannel_logout(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/auth/oidc/backchannel-logout");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/auth/oidc/callback
    pub async fn oidc_callback(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/auth/oidc/callback");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/auth/oidc/login
    pub async fn oidc_login(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/auth/oidc/login");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/auth/sso/providers
    pub async fn list_sso_providers(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/auth/sso/providers");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/chunks/{chunk_id}
    pub async fn get_chunk_detail(&self, chunk_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/chunks/");
        path.push_str(&encode_path_param(chunk_id));
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/chunks/{chunk_id}/lineage
    pub async fn get_chunk_lineage(&self, chunk_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/chunks/");
        path.push_str(&encode_path_param(chunk_id));
        path.push_str("/lineage");
        self.call(Method::GET, path, query, body, false).await
    }
    /// PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback
    pub async fn set_message_feedback(&self, conversation_id: &str, message_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/conversations/");
        path.push_str(&encode_path_param(conversation_id));
        path.push_str("/messages/");
        path.push_str(&encode_path_param(message_id));
        path.push_str("/feedback");
        self.call(Method::PATCH, path, query, body, false).await
    }
    /// GET /api/v1/decision/models
    pub async fn get_decision_models(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/decision/models");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/decision/status
    pub async fn get_decision_status(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/decision/status");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/documents/batch-delete
    pub async fn batch_delete_documents(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/batch-delete");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/documents/pdf/progress/stream/{track_id}
    pub async fn get_pdf_progress_stream(&self, track_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/pdf/progress/stream/");
        path.push_str(&encode_path_param(track_id));
        self.call(Method::GET, path, query, body, true).await
    }
    /// GET /api/v1/documents/search
    pub async fn search_documents(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/search");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/documents/upload
    pub async fn upload_file(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/upload");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /api/v1/documents/upload/batch
    pub async fn upload_files_batch(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/upload/batch");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/assets
    pub async fn list_document_assets(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/assets");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/documents/{document_id}/assets/include-from-pdf
    pub async fn include_document_assets_from_pdf(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/assets/include-from-pdf");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/assets/{asset_id}
    pub async fn download_document_asset_by_id(&self, document_id: &str, asset_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/assets/");
        path.push_str(&encode_path_param(asset_id));
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/documents/{document_id}/cancel
    pub async fn cancel_document(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/cancel");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/deletion-impact
    pub async fn analyze_deletion_impact(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/deletion-impact");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/download/markdown
    pub async fn download_document_markdown(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/download/markdown");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/download/original
    pub async fn download_document_original(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/download/original");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/failed-chunks
    pub async fn list_failed_chunks(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/failed-chunks");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/metadata
    pub async fn get_document_metadata(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/metadata");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/mm-assets/{asset_path}
    pub async fn download_document_mm_asset(&self, document_id: &str, asset_path: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/mm-assets/");
        path.push_str(&encode_path_param(asset_path));
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/pages
    pub async fn list_document_pages(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/pages");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/pages/health
    pub async fn get_pages_health(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/pages/health");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/documents/{document_id}/pages/reprocess
    pub async fn reprocess_pages(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/pages/reprocess");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/documents/{document_id}/pages/{page_number}/layout
    pub async fn get_document_page_layout(&self, document_id: &str, page_number: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/pages/");
        path.push_str(&encode_path_param(page_number));
        path.push_str("/layout");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/documents/{document_id}/reanalyze
    pub async fn reanalyze_multimodal(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/reanalyze");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /api/v1/documents/{document_id}/retry-chunks
    pub async fn retry_failed_chunks(&self, document_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/documents/");
        path.push_str(&encode_path_param(document_id));
        path.push_str("/retry-chunks");
        self.call(Method::POST, path, query, body, false).await
    }
    /// DELETE /api/v1/folders/{folder_id}
    pub async fn delete_folder(&self, folder_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/folders/");
        path.push_str(&encode_path_param(folder_id));
        self.call(Method::DELETE, path, query, body, false).await
    }
    /// PATCH /api/v1/folders/{folder_id}
    pub async fn update_folder(&self, folder_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/folders/");
        path.push_str(&encode_path_param(folder_id));
        self.call(Method::PATCH, path, query, body, false).await
    }
    /// GET /api/v1/graph/communities
    pub async fn get_graph_communities(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/communities");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/graph/entities/{entity_name}
    pub async fn get_entity(&self, entity_name: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/entities/");
        path.push_str(&encode_path_param(entity_name));
        self.call(Method::GET, path, query, body, false).await
    }
    /// PUT /api/v1/graph/entities/{entity_name}
    pub async fn update_entity(&self, entity_name: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/entities/");
        path.push_str(&encode_path_param(entity_name));
        self.call(Method::PUT, path, query, body, false).await
    }
    /// DELETE /api/v1/graph/entities/{entity_name}
    pub async fn delete_entity(&self, entity_name: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/entities/");
        path.push_str(&encode_path_param(entity_name));
        self.call(Method::DELETE, path, query, body, false).await
    }
    /// GET /api/v1/graph/entities/{entity_name}/neighborhood
    pub async fn get_entity_neighborhood(&self, entity_name: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/entities/");
        path.push_str(&encode_path_param(entity_name));
        path.push_str("/neighborhood");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/graph/facets
    pub async fn get_graph_facets(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/facets");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/graph/relationships/{relationship_id}
    pub async fn get_relationship(&self, relationship_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/relationships/");
        path.push_str(&encode_path_param(relationship_id));
        self.call(Method::GET, path, query, body, false).await
    }
    /// PUT /api/v1/graph/relationships/{relationship_id}
    pub async fn update_relationship(&self, relationship_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/relationships/");
        path.push_str(&encode_path_param(relationship_id));
        self.call(Method::PUT, path, query, body, false).await
    }
    /// DELETE /api/v1/graph/relationships/{relationship_id}
    pub async fn delete_relationship(&self, relationship_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/graph/relationships/");
        path.push_str(&encode_path_param(relationship_id));
        self.call(Method::DELETE, path, query, body, false).await
    }
    /// POST /api/v1/ingestion/progress
    pub async fn post_ingestion_progress_batch(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/ingestion/progress");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/ingestion/{track_id}/progress
    pub async fn get_ingestion_progress(&self, track_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/ingestion/");
        path.push_str(&encode_path_param(track_id));
        path.push_str("/progress");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/mcp
    pub async fn mcp_openapi_v1(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/mcp");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /api/v1/models/discover/refresh
    pub async fn refresh_model_discovery(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/models/discover/refresh");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/models/search
    pub async fn search_models(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/models/search");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/pipeline/activity
    pub async fn get_pipeline_activity(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/pipeline/activity");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/query/context
    pub async fn retrieve_query_context(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/query/context");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id}
    pub async fn get_context_artifact(&self, artifact_type: &str, artifact_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/query/context/artifacts/");
        path.push_str(&encode_path_param(artifact_type));
        path.push_str("/");
        path.push_str(&encode_path_param(artifact_id));
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v1/query/context/search
    pub async fn search_query_context(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/query/context/search");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/query/context/{retrieval_id}
    pub async fn fetch_query_context(&self, retrieval_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/query/context/");
        path.push_str(&encode_path_param(retrieval_id));
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/settings/app-attribution
    pub async fn get_app_attribution_settings(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/settings/app-attribution");
        self.call(Method::GET, path, query, body, false).await
    }
    /// PATCH /api/v1/settings/app-attribution
    pub async fn update_app_attribution(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/settings/app-attribution");
        self.call(Method::PATCH, path, query, body, false).await
    }
    /// GET /api/v1/settings/attribution
    pub async fn get_attribution_settings(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/settings/attribution");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/settings/langfuse
    pub async fn get_langfuse_settings(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/settings/langfuse");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/settings/llm-defaults
    pub async fn get_llm_defaults(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/settings/llm-defaults");
        self.call(Method::GET, path, query, body, false).await
    }
    /// PATCH /api/v1/settings/llm-defaults
    pub async fn update_llm_defaults(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/settings/llm-defaults");
        self.call(Method::PATCH, path, query, body, false).await
    }
    /// POST /api/v1/setup/initialize
    pub async fn setup_initialize(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/setup/initialize");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v1/setup/status
    pub async fn setup_status(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/setup/status");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v1/users/{user_id}
    pub async fn get_user(&self, user_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/users/");
        path.push_str(&encode_path_param(user_id));
        self.call(Method::GET, path, query, body, false).await
    }
    /// DELETE /api/v1/users/{user_id}
    pub async fn delete_user(&self, user_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/users/");
        path.push_str(&encode_path_param(user_id));
        self.call(Method::DELETE, path, query, body, false).await
    }
    /// PATCH /api/v1/users/{user_id}
    pub async fn update_user(&self, user_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v1/users/");
        path.push_str(&encode_path_param(user_id));
        self.call(Method::PATCH, path, query, body, false).await
    }
    /// GET /api/v2/workspaces/{workspace_id}/jobs
    pub async fn list_workspace_jobs(&self, workspace_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v2/workspaces/");
        path.push_str(&encode_path_param(workspace_id));
        path.push_str("/jobs");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /api/v2/workspaces/{workspace_id}/jobs
    pub async fn create_workspace_job(&self, workspace_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v2/workspaces/");
        path.push_str(&encode_path_param(workspace_id));
        path.push_str("/jobs");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /api/v2/workspaces/{workspace_id}/jobs/catalog
    pub async fn list_workspace_job_catalog(&self, workspace_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v2/workspaces/");
        path.push_str(&encode_path_param(workspace_id));
        path.push_str("/jobs/catalog");
        self.call(Method::GET, path, query, body, false).await
    }
    /// GET /api/v2/workspaces/{workspace_id}/jobs/{job_id}
    pub async fn get_workspace_job(&self, workspace_id: &str, job_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v2/workspaces/");
        path.push_str(&encode_path_param(workspace_id));
        path.push_str("/jobs/");
        path.push_str(&encode_path_param(job_id));
        self.call(Method::GET, path, query, body, false).await
    }
    /// DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id}
    pub async fn cancel_workspace_job(&self, workspace_id: &str, job_id: &str, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/v2/workspaces/");
        path.push_str(&encode_path_param(workspace_id));
        path.push_str("/jobs/");
        path.push_str(&encode_path_param(job_id));
        self.call(Method::DELETE, path, query, body, false).await
    }
    /// GET /api/version
    pub async fn ollama_version(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/api/version");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /mcp
    pub async fn mcp_openapi(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/mcp");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /oauth/authorize
    pub async fn oauth_authorize_get(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/oauth/authorize");
        self.call(Method::GET, path, query, body, false).await
    }
    /// POST /oauth/authorize
    pub async fn oauth_authorize_post(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/oauth/authorize");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /oauth/register
    pub async fn oauth_register_post(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/oauth/register");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /oauth/revoke
    pub async fn oauth_revoke_post(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/oauth/revoke");
        self.call(Method::POST, path, query, body, false).await
    }
    /// POST /oauth/token
    pub async fn oauth_token_post(&self, query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {
        let mut path = String::new();
        path.push_str("/oauth/token");
        self.call(Method::POST, path, query, body, false).await
    }
    /// GET /ws/pipeline/progress
    pub fn ws_pipeline_progress(&self) -> String {
        let mut path = String::new();
        path.push_str("/ws/pipeline/progress");
        socket_url(self.client.base_url(), &path)
    }
    /// GET /ws/progress/{track_id}
    pub fn ws_progress_by_track_id(&self, track_id: &str) -> String {
        let mut path = String::new();
        path.push_str("/ws/progress/");
        path.push_str(&encode_path_param(track_id));
        socket_url(self.client.base_url(), &path)
    }
}

fn parse_events(text: &str) -> String {
    let mut events = Vec::new();
    for block in text.split("\n\n") {
        for line in block.lines() {
            let Some(data) = line.strip_prefix("data:") else { continue };
            let data = data.trim();
            if !data.is_empty() && data != "[DONE]" {
                events.push(data.to_string());
            }
        }
    }
    events.join("\n")
}

fn socket_url(base: &str, path: &str) -> String {
    let root = base.trim_end_matches('/');
    let root = if let Some(rest) = root.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = root.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        root.to_string()
    };
    format!("{root}{path}")
}

fn encode_path_param(value: &str) -> String {
    value.split('/').map(encode_query_param).collect::<Vec<_>>().join("/")
}

fn encode_query_param(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{parse_events, socket_url};

    #[test]
    fn sse_data_lines_and_socket_url() {
        let text = "data: {\"a\":1}\n\ndata: [DONE]\n\n: keep-alive\n\ndata: hi\n";
        assert_eq!(parse_events(text), "{\"a\":1}\nhi");
        assert_eq!(
            socket_url("http://localhost:8080/", "/ws/pipeline/progress"),
            "ws://localhost:8080/ws/pipeline/progress"
        );
        assert_eq!(
            socket_url("https://api.example", "/ws/progress/t"),
            "wss://api.example/ws/progress/t"
        );
    }
}
