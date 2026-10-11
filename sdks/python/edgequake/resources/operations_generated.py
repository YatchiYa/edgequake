"""Generated OpenAPI operations. Do not edit by hand.

Regenerate: python3 sdks/tools/sdk_coverage.py --generate
"""

from __future__ import annotations

from typing import Any
from urllib.parse import quote, urlencode

from edgequake.resources._base import AsyncResource, SyncResource


def _path_param(value: object) -> str:
    """Encode one path parameter and keep slashes for wildcard segments."""
    return "/".join(quote(part, safe="") for part in str(value).split("/"))


def _prepare(path: str, query: dict | None) -> str:
    if not query:
        return path
    joined = urlencode({key: value for key, value in query.items() if value is not None})
    if not joined:
        return path
    return path + ('&' if '?' in path else '?') + joined


def _events(text: str) -> list[str]:
    """SSE data lines. One parser for every generated stream route."""
    events: list[str] = []
    for block in text.split("\n\n"):
        for line in block.splitlines():
            if not line.startswith("data:"):
                continue
            data = line[5:].lstrip()
            if data and data != "[DONE]":
                events.append(data)
    return events


def _decode(response: Any, events: bool) -> Any:
    if events:
        return _events(response.text)
    if response.status_code == 204 or not response.content:
        return None
    try:
        return response.json()
    except Exception:
        return response.text


def _socket_url(base: str, path: str) -> str:
    root = str(base).rstrip("/")
    if root.startswith("https://"):
        root = "wss://" + root[len("https://") :]
    elif root.startswith("http://"):
        root = "ws://" + root[len("http://") :]
    return root + path


class OperationsResource(SyncResource):
    """Every OpenAPI operation that the hand-written resources do not already mention."""

    def _invoke(self, method: str, path: str, query: dict | None, body: dict | None, events: bool) -> Any:
        response = self._transport.request(method, _prepare(path, query), json=body, params=None)
        return _decode(response, events)

    def mcp_registry_server_json(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/mcp/server.json"""
        return self._invoke("GET", "/.well-known/mcp/server.json", query, body, False)

    def oauth_authorization_server_metadata(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/oauth-authorization-server"""
        return self._invoke("GET", "/.well-known/oauth-authorization-server", query, body, False)

    def mcp_oauth_protected_resource(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/oauth-protected-resource"""
        return self._invoke("GET", "/.well-known/oauth-protected-resource", query, body, False)

    def mcp_oauth_protected_resource_path(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/oauth-protected-resource/mcp"""
        return self._invoke("GET", "/.well-known/oauth-protected-resource/mcp", query, body, False)

    def openid_configuration(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/openid-configuration"""
        return self._invoke("GET", "/.well-known/openid-configuration", query, body, False)

    def ollama_chat(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/chat"""
        return self._invoke("POST", "/api/chat", query, body, False)

    def ollama_generate(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/generate"""
        return self._invoke("POST", "/api/generate", query, body, False)

    def ollama_ps(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/ps"""
        return self._invoke("GET", "/api/ps", query, body, False)

    def ollama_tags(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/tags"""
        return self._invoke("GET", "/api/tags", query, body, False)

    def ann_warmup(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/ann/warmup"""
        return self._invoke("POST", "/api/v1/admin/ann/warmup", query, body, False)

    def entity_reconcile_plan(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/entities/reconcile"""
        return self._invoke("GET", "/api/v1/admin/entities/reconcile", query, body, False)

    def entity_reconcile_execute(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/entities/reconcile"""
        return self._invoke("POST", "/api/v1/admin/entities/reconcile", query, body, False)

    def admin_list_identity_providers(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/identity-providers"""
        return self._invoke("GET", "/api/v1/admin/identity-providers", query, body, False)

    def admin_upsert_identity_provider(self, slug: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """PUT /api/v1/admin/identity-providers/{slug}"""
        return self._invoke("PUT", "/api/v1/admin/identity-providers/" + _path_param(slug), query, body, False)

    def admin_delete_identity_provider(self, slug: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """DELETE /api/v1/admin/identity-providers/{slug}"""
        return self._invoke("DELETE", "/api/v1/admin/identity-providers/" + _path_param(slug), query, body, False)

    def list_migration_jobs(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/migration-jobs"""
        return self._invoke("GET", "/api/v1/admin/migration-jobs", query, body, False)

    def get_migration_job(self, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/migration-jobs/{job_id}"""
        return self._invoke("GET", "/api/v1/admin/migration-jobs/" + _path_param(job_id), query, body, False)

    def cancel_migration_job(self, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/migration-jobs/{job_id}/cancel"""
        return self._invoke("POST", "/api/v1/admin/migration-jobs/" + _path_param(job_id) + "/cancel", query, body, False)

    def pause_migration_job(self, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/migration-jobs/{job_id}/pause"""
        return self._invoke("POST", "/api/v1/admin/migration-jobs/" + _path_param(job_id) + "/pause", query, body, False)

    def resume_migration_job(self, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/migration-jobs/{job_id}/resume"""
        return self._invoke("POST", "/api/v1/admin/migration-jobs/" + _path_param(job_id) + "/resume", query, body, False)

    def storage_inspect(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/storage/inspect"""
        return self._invoke("GET", "/api/v1/admin/storage/inspect", query, body, False)

    def storage_repair(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/storage/repair"""
        return self._invoke("POST", "/api/v1/admin/storage/repair", query, body, False)

    def redeem_sso_handoff(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/auth/handoff"""
        return self._invoke("POST", "/api/v1/auth/handoff", query, body, False)

    def oidc_backchannel_logout(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/auth/oidc/backchannel-logout"""
        return self._invoke("POST", "/api/v1/auth/oidc/backchannel-logout", query, body, False)

    def oidc_callback(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/auth/oidc/callback"""
        return self._invoke("GET", "/api/v1/auth/oidc/callback", query, body, False)

    def oidc_login(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/auth/oidc/login"""
        return self._invoke("GET", "/api/v1/auth/oidc/login", query, body, False)

    def list_sso_providers(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/auth/sso/providers"""
        return self._invoke("GET", "/api/v1/auth/sso/providers", query, body, False)

    def set_message_feedback(self, conversation_id: str, message_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback"""
        return self._invoke("PATCH", "/api/v1/conversations/" + _path_param(conversation_id) + "/messages/" + _path_param(message_id) + "/feedback", query, body, False)

    def get_decision_models(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/decision/models"""
        return self._invoke("GET", "/api/v1/decision/models", query, body, False)

    def get_decision_status(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/decision/status"""
        return self._invoke("GET", "/api/v1/decision/status", query, body, False)

    def batch_delete_documents(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/batch-delete"""
        return self._invoke("POST", "/api/v1/documents/batch-delete", query, body, False)

    def get_pdf_progress_stream(self, track_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/pdf/progress/stream/{track_id}"""
        return self._invoke("GET", "/api/v1/documents/pdf/progress/stream/" + _path_param(track_id), query, body, True)

    def search_documents(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/search"""
        return self._invoke("GET", "/api/v1/documents/search", query, body, False)

    def list_document_assets(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/assets"""
        return self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/assets", query, body, False)

    def include_document_assets_from_pdf(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/{document_id}/assets/include-from-pdf"""
        return self._invoke("POST", "/api/v1/documents/" + _path_param(document_id) + "/assets/include-from-pdf", query, body, False)

    def download_document_asset_by_id(self, document_id: str, asset_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/assets/{asset_id}"""
        return self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/assets/" + _path_param(asset_id), query, body, False)

    def cancel_document(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/{document_id}/cancel"""
        return self._invoke("POST", "/api/v1/documents/" + _path_param(document_id) + "/cancel", query, body, False)

    def download_document_markdown(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/download/markdown"""
        return self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/download/markdown", query, body, False)

    def download_document_original(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/download/original"""
        return self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/download/original", query, body, False)

    def download_document_mm_asset(self, document_id: str, asset_path: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/mm-assets/{asset_path}"""
        return self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/mm-assets/" + _path_param(asset_path), query, body, False)

    def list_document_pages(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/pages"""
        return self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/pages", query, body, False)

    def get_pages_health(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/pages/health"""
        return self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/pages/health", query, body, False)

    def reprocess_pages(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/{document_id}/pages/reprocess"""
        return self._invoke("POST", "/api/v1/documents/" + _path_param(document_id) + "/pages/reprocess", query, body, False)

    def get_document_page_layout(self, document_id: str, page_number: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/pages/{page_number}/layout"""
        return self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/pages/" + _path_param(page_number) + "/layout", query, body, False)

    def reanalyze_multimodal(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/{document_id}/reanalyze"""
        return self._invoke("POST", "/api/v1/documents/" + _path_param(document_id) + "/reanalyze", query, body, False)

    def get_graph_communities(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/graph/communities"""
        return self._invoke("GET", "/api/v1/graph/communities", query, body, False)

    def get_graph_facets(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/graph/facets"""
        return self._invoke("GET", "/api/v1/graph/facets", query, body, False)

    def post_ingestion_progress_batch(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/ingestion/progress"""
        return self._invoke("POST", "/api/v1/ingestion/progress", query, body, False)

    def get_ingestion_progress(self, track_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/ingestion/{track_id}/progress"""
        return self._invoke("GET", "/api/v1/ingestion/" + _path_param(track_id) + "/progress", query, body, False)

    def mcp_openapi_v1(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/mcp"""
        return self._invoke("POST", "/api/v1/mcp", query, body, False)

    def refresh_model_discovery(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/models/discover/refresh"""
        return self._invoke("POST", "/api/v1/models/discover/refresh", query, body, False)

    def search_models(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/models/search"""
        return self._invoke("GET", "/api/v1/models/search", query, body, False)

    def get_parse_job(self, id_: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/parse/jobs/{id}"""
        return self._invoke("GET", "/api/v1/parse/jobs/" + _path_param(id_), query, body, False)

    def get_pipeline_activity(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/pipeline/activity"""
        return self._invoke("GET", "/api/v1/pipeline/activity", query, body, False)

    def retrieve_query_context(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/query/context"""
        return self._invoke("POST", "/api/v1/query/context", query, body, False)

    def get_context_artifact(self, artifact_type: str, artifact_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id}"""
        return self._invoke("GET", "/api/v1/query/context/artifacts/" + _path_param(artifact_type) + "/" + _path_param(artifact_id), query, body, False)

    def search_query_context(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/query/context/search"""
        return self._invoke("POST", "/api/v1/query/context/search", query, body, False)

    def fetch_query_context(self, retrieval_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/query/context/{retrieval_id}"""
        return self._invoke("GET", "/api/v1/query/context/" + _path_param(retrieval_id), query, body, False)

    def get_app_attribution_settings(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/settings/app-attribution"""
        return self._invoke("GET", "/api/v1/settings/app-attribution", query, body, False)

    def update_app_attribution(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """PATCH /api/v1/settings/app-attribution"""
        return self._invoke("PATCH", "/api/v1/settings/app-attribution", query, body, False)

    def get_attribution_settings(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/settings/attribution"""
        return self._invoke("GET", "/api/v1/settings/attribution", query, body, False)

    def get_langfuse_settings(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/settings/langfuse"""
        return self._invoke("GET", "/api/v1/settings/langfuse", query, body, False)

    def get_llm_defaults(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/settings/llm-defaults"""
        return self._invoke("GET", "/api/v1/settings/llm-defaults", query, body, False)

    def update_llm_defaults(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """PATCH /api/v1/settings/llm-defaults"""
        return self._invoke("PATCH", "/api/v1/settings/llm-defaults", query, body, False)

    def setup_initialize(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/setup/initialize"""
        return self._invoke("POST", "/api/v1/setup/initialize", query, body, False)

    def setup_status(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/setup/status"""
        return self._invoke("GET", "/api/v1/setup/status", query, body, False)

    def list_workspace_jobs(self, workspace_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v2/workspaces/{workspace_id}/jobs"""
        return self._invoke("GET", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs", query, body, False)

    def create_workspace_job(self, workspace_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v2/workspaces/{workspace_id}/jobs"""
        return self._invoke("POST", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs", query, body, False)

    def list_workspace_job_catalog(self, workspace_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v2/workspaces/{workspace_id}/jobs/catalog"""
        return self._invoke("GET", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs/catalog", query, body, False)

    def get_workspace_job(self, workspace_id: str, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v2/workspaces/{workspace_id}/jobs/{job_id}"""
        return self._invoke("GET", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs/" + _path_param(job_id), query, body, False)

    def cancel_workspace_job(self, workspace_id: str, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id}"""
        return self._invoke("DELETE", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs/" + _path_param(job_id), query, body, False)

    def ollama_version(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/version"""
        return self._invoke("GET", "/api/version", query, body, False)

    def liveness_check(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /live"""
        return self._invoke("GET", "/live", query, body, False)

    def mcp_openapi(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /mcp"""
        return self._invoke("POST", "/mcp", query, body, False)

    def oauth_authorize_get(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /oauth/authorize"""
        return self._invoke("GET", "/oauth/authorize", query, body, False)

    def oauth_authorize_post(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /oauth/authorize"""
        return self._invoke("POST", "/oauth/authorize", query, body, False)

    def oauth_register_post(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /oauth/register"""
        return self._invoke("POST", "/oauth/register", query, body, False)

    def oauth_revoke_post(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /oauth/revoke"""
        return self._invoke("POST", "/oauth/revoke", query, body, False)

    def oauth_token_post(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /oauth/token"""
        return self._invoke("POST", "/oauth/token", query, body, False)

    def ws_pipeline_progress(self) -> str:
        """GET /ws/pipeline/progress"""
        return _socket_url(self._transport.base_url, "/ws/pipeline/progress")

    def ws_progress_by_track_id(self, track_id: str) -> str:
        """GET /ws/progress/{track_id}"""
        return _socket_url(self._transport.base_url, "/ws/progress/" + _path_param(track_id))


class AsyncOperationsResource(AsyncResource):
    """Async twin of OperationsResource."""

    async def _invoke(self, method: str, path: str, query: dict | None, body: dict | None, events: bool) -> Any:
        response = await self._transport.request(method, _prepare(path, query), json=body, params=None)
        return _decode(response, events)

    async def mcp_registry_server_json(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/mcp/server.json"""
        return await self._invoke("GET", "/.well-known/mcp/server.json", query, body, False)

    async def oauth_authorization_server_metadata(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/oauth-authorization-server"""
        return await self._invoke("GET", "/.well-known/oauth-authorization-server", query, body, False)

    async def mcp_oauth_protected_resource(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/oauth-protected-resource"""
        return await self._invoke("GET", "/.well-known/oauth-protected-resource", query, body, False)

    async def mcp_oauth_protected_resource_path(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/oauth-protected-resource/mcp"""
        return await self._invoke("GET", "/.well-known/oauth-protected-resource/mcp", query, body, False)

    async def openid_configuration(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /.well-known/openid-configuration"""
        return await self._invoke("GET", "/.well-known/openid-configuration", query, body, False)

    async def ollama_chat(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/chat"""
        return await self._invoke("POST", "/api/chat", query, body, False)

    async def ollama_generate(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/generate"""
        return await self._invoke("POST", "/api/generate", query, body, False)

    async def ollama_ps(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/ps"""
        return await self._invoke("GET", "/api/ps", query, body, False)

    async def ollama_tags(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/tags"""
        return await self._invoke("GET", "/api/tags", query, body, False)

    async def ann_warmup(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/ann/warmup"""
        return await self._invoke("POST", "/api/v1/admin/ann/warmup", query, body, False)

    async def entity_reconcile_plan(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/entities/reconcile"""
        return await self._invoke("GET", "/api/v1/admin/entities/reconcile", query, body, False)

    async def entity_reconcile_execute(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/entities/reconcile"""
        return await self._invoke("POST", "/api/v1/admin/entities/reconcile", query, body, False)

    async def admin_list_identity_providers(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/identity-providers"""
        return await self._invoke("GET", "/api/v1/admin/identity-providers", query, body, False)

    async def admin_upsert_identity_provider(self, slug: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """PUT /api/v1/admin/identity-providers/{slug}"""
        return await self._invoke("PUT", "/api/v1/admin/identity-providers/" + _path_param(slug), query, body, False)

    async def admin_delete_identity_provider(self, slug: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """DELETE /api/v1/admin/identity-providers/{slug}"""
        return await self._invoke("DELETE", "/api/v1/admin/identity-providers/" + _path_param(slug), query, body, False)

    async def list_migration_jobs(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/migration-jobs"""
        return await self._invoke("GET", "/api/v1/admin/migration-jobs", query, body, False)

    async def get_migration_job(self, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/migration-jobs/{job_id}"""
        return await self._invoke("GET", "/api/v1/admin/migration-jobs/" + _path_param(job_id), query, body, False)

    async def cancel_migration_job(self, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/migration-jobs/{job_id}/cancel"""
        return await self._invoke("POST", "/api/v1/admin/migration-jobs/" + _path_param(job_id) + "/cancel", query, body, False)

    async def pause_migration_job(self, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/migration-jobs/{job_id}/pause"""
        return await self._invoke("POST", "/api/v1/admin/migration-jobs/" + _path_param(job_id) + "/pause", query, body, False)

    async def resume_migration_job(self, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/migration-jobs/{job_id}/resume"""
        return await self._invoke("POST", "/api/v1/admin/migration-jobs/" + _path_param(job_id) + "/resume", query, body, False)

    async def storage_inspect(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/admin/storage/inspect"""
        return await self._invoke("GET", "/api/v1/admin/storage/inspect", query, body, False)

    async def storage_repair(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/admin/storage/repair"""
        return await self._invoke("POST", "/api/v1/admin/storage/repair", query, body, False)

    async def redeem_sso_handoff(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/auth/handoff"""
        return await self._invoke("POST", "/api/v1/auth/handoff", query, body, False)

    async def oidc_backchannel_logout(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/auth/oidc/backchannel-logout"""
        return await self._invoke("POST", "/api/v1/auth/oidc/backchannel-logout", query, body, False)

    async def oidc_callback(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/auth/oidc/callback"""
        return await self._invoke("GET", "/api/v1/auth/oidc/callback", query, body, False)

    async def oidc_login(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/auth/oidc/login"""
        return await self._invoke("GET", "/api/v1/auth/oidc/login", query, body, False)

    async def list_sso_providers(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/auth/sso/providers"""
        return await self._invoke("GET", "/api/v1/auth/sso/providers", query, body, False)

    async def set_message_feedback(self, conversation_id: str, message_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """PATCH /api/v1/conversations/{conversation_id}/messages/{message_id}/feedback"""
        return await self._invoke("PATCH", "/api/v1/conversations/" + _path_param(conversation_id) + "/messages/" + _path_param(message_id) + "/feedback", query, body, False)

    async def get_decision_models(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/decision/models"""
        return await self._invoke("GET", "/api/v1/decision/models", query, body, False)

    async def get_decision_status(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/decision/status"""
        return await self._invoke("GET", "/api/v1/decision/status", query, body, False)

    async def batch_delete_documents(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/batch-delete"""
        return await self._invoke("POST", "/api/v1/documents/batch-delete", query, body, False)

    async def get_pdf_progress_stream(self, track_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/pdf/progress/stream/{track_id}"""
        return await self._invoke("GET", "/api/v1/documents/pdf/progress/stream/" + _path_param(track_id), query, body, True)

    async def search_documents(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/search"""
        return await self._invoke("GET", "/api/v1/documents/search", query, body, False)

    async def list_document_assets(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/assets"""
        return await self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/assets", query, body, False)

    async def include_document_assets_from_pdf(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/{document_id}/assets/include-from-pdf"""
        return await self._invoke("POST", "/api/v1/documents/" + _path_param(document_id) + "/assets/include-from-pdf", query, body, False)

    async def download_document_asset_by_id(self, document_id: str, asset_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/assets/{asset_id}"""
        return await self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/assets/" + _path_param(asset_id), query, body, False)

    async def cancel_document(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/{document_id}/cancel"""
        return await self._invoke("POST", "/api/v1/documents/" + _path_param(document_id) + "/cancel", query, body, False)

    async def download_document_markdown(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/download/markdown"""
        return await self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/download/markdown", query, body, False)

    async def download_document_original(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/download/original"""
        return await self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/download/original", query, body, False)

    async def download_document_mm_asset(self, document_id: str, asset_path: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/mm-assets/{asset_path}"""
        return await self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/mm-assets/" + _path_param(asset_path), query, body, False)

    async def list_document_pages(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/pages"""
        return await self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/pages", query, body, False)

    async def get_pages_health(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/pages/health"""
        return await self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/pages/health", query, body, False)

    async def reprocess_pages(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/{document_id}/pages/reprocess"""
        return await self._invoke("POST", "/api/v1/documents/" + _path_param(document_id) + "/pages/reprocess", query, body, False)

    async def get_document_page_layout(self, document_id: str, page_number: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/documents/{document_id}/pages/{page_number}/layout"""
        return await self._invoke("GET", "/api/v1/documents/" + _path_param(document_id) + "/pages/" + _path_param(page_number) + "/layout", query, body, False)

    async def reanalyze_multimodal(self, document_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/documents/{document_id}/reanalyze"""
        return await self._invoke("POST", "/api/v1/documents/" + _path_param(document_id) + "/reanalyze", query, body, False)

    async def get_graph_communities(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/graph/communities"""
        return await self._invoke("GET", "/api/v1/graph/communities", query, body, False)

    async def get_graph_facets(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/graph/facets"""
        return await self._invoke("GET", "/api/v1/graph/facets", query, body, False)

    async def post_ingestion_progress_batch(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/ingestion/progress"""
        return await self._invoke("POST", "/api/v1/ingestion/progress", query, body, False)

    async def get_ingestion_progress(self, track_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/ingestion/{track_id}/progress"""
        return await self._invoke("GET", "/api/v1/ingestion/" + _path_param(track_id) + "/progress", query, body, False)

    async def mcp_openapi_v1(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/mcp"""
        return await self._invoke("POST", "/api/v1/mcp", query, body, False)

    async def refresh_model_discovery(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/models/discover/refresh"""
        return await self._invoke("POST", "/api/v1/models/discover/refresh", query, body, False)

    async def search_models(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/models/search"""
        return await self._invoke("GET", "/api/v1/models/search", query, body, False)

    async def get_parse_job(self, id_: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/parse/jobs/{id}"""
        return await self._invoke("GET", "/api/v1/parse/jobs/" + _path_param(id_), query, body, False)

    async def get_pipeline_activity(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/pipeline/activity"""
        return await self._invoke("GET", "/api/v1/pipeline/activity", query, body, False)

    async def retrieve_query_context(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/query/context"""
        return await self._invoke("POST", "/api/v1/query/context", query, body, False)

    async def get_context_artifact(self, artifact_type: str, artifact_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/query/context/artifacts/{artifact_type}/{artifact_id}"""
        return await self._invoke("GET", "/api/v1/query/context/artifacts/" + _path_param(artifact_type) + "/" + _path_param(artifact_id), query, body, False)

    async def search_query_context(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/query/context/search"""
        return await self._invoke("POST", "/api/v1/query/context/search", query, body, False)

    async def fetch_query_context(self, retrieval_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/query/context/{retrieval_id}"""
        return await self._invoke("GET", "/api/v1/query/context/" + _path_param(retrieval_id), query, body, False)

    async def get_app_attribution_settings(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/settings/app-attribution"""
        return await self._invoke("GET", "/api/v1/settings/app-attribution", query, body, False)

    async def update_app_attribution(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """PATCH /api/v1/settings/app-attribution"""
        return await self._invoke("PATCH", "/api/v1/settings/app-attribution", query, body, False)

    async def get_attribution_settings(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/settings/attribution"""
        return await self._invoke("GET", "/api/v1/settings/attribution", query, body, False)

    async def get_langfuse_settings(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/settings/langfuse"""
        return await self._invoke("GET", "/api/v1/settings/langfuse", query, body, False)

    async def get_llm_defaults(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/settings/llm-defaults"""
        return await self._invoke("GET", "/api/v1/settings/llm-defaults", query, body, False)

    async def update_llm_defaults(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """PATCH /api/v1/settings/llm-defaults"""
        return await self._invoke("PATCH", "/api/v1/settings/llm-defaults", query, body, False)

    async def setup_initialize(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v1/setup/initialize"""
        return await self._invoke("POST", "/api/v1/setup/initialize", query, body, False)

    async def setup_status(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v1/setup/status"""
        return await self._invoke("GET", "/api/v1/setup/status", query, body, False)

    async def list_workspace_jobs(self, workspace_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v2/workspaces/{workspace_id}/jobs"""
        return await self._invoke("GET", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs", query, body, False)

    async def create_workspace_job(self, workspace_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /api/v2/workspaces/{workspace_id}/jobs"""
        return await self._invoke("POST", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs", query, body, False)

    async def list_workspace_job_catalog(self, workspace_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v2/workspaces/{workspace_id}/jobs/catalog"""
        return await self._invoke("GET", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs/catalog", query, body, False)

    async def get_workspace_job(self, workspace_id: str, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/v2/workspaces/{workspace_id}/jobs/{job_id}"""
        return await self._invoke("GET", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs/" + _path_param(job_id), query, body, False)

    async def cancel_workspace_job(self, workspace_id: str, job_id: str, *, query: dict | None = None, body: dict | None = None) -> Any:
        """DELETE /api/v2/workspaces/{workspace_id}/jobs/{job_id}"""
        return await self._invoke("DELETE", "/api/v2/workspaces/" + _path_param(workspace_id) + "/jobs/" + _path_param(job_id), query, body, False)

    async def ollama_version(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /api/version"""
        return await self._invoke("GET", "/api/version", query, body, False)

    async def liveness_check(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /live"""
        return await self._invoke("GET", "/live", query, body, False)

    async def mcp_openapi(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /mcp"""
        return await self._invoke("POST", "/mcp", query, body, False)

    async def oauth_authorize_get(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """GET /oauth/authorize"""
        return await self._invoke("GET", "/oauth/authorize", query, body, False)

    async def oauth_authorize_post(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /oauth/authorize"""
        return await self._invoke("POST", "/oauth/authorize", query, body, False)

    async def oauth_register_post(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /oauth/register"""
        return await self._invoke("POST", "/oauth/register", query, body, False)

    async def oauth_revoke_post(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /oauth/revoke"""
        return await self._invoke("POST", "/oauth/revoke", query, body, False)

    async def oauth_token_post(self, *, query: dict | None = None, body: dict | None = None) -> Any:
        """POST /oauth/token"""
        return await self._invoke("POST", "/oauth/token", query, body, False)

    async def ws_pipeline_progress(self) -> str:
        """GET /ws/pipeline/progress"""
        return _socket_url(self._transport.base_url, "/ws/pipeline/progress")

    async def ws_progress_by_track_id(self, track_id: str) -> str:
        """GET /ws/progress/{track_id}"""
        return _socket_url(self._transport.base_url, "/ws/progress/" + _path_param(track_id))

