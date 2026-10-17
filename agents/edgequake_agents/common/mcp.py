"""ADK toolset for EdgeQuake's native MCP endpoint (POST {EDGEQUAKE_URL}/mcp).

Tools exposed by EdgeQuake (tools/list): eq_search, eq_fetch, eq_retrieve,
eq_entity_search, eq_entity_get, eq_neighborhood, eq_graph_image,
eq_document_list, eq_document_get, eq_document_download, eq_asset_get,
eq_workspace_list, eq_workspace_stats, eq_task_get, plus write tools
(eq_ingest, eq_upload_*, eq_document_delete, eq_workspace_delete) and
edgequake_* aliases. Agents here only ever get read tools.
"""

from __future__ import annotations

import base64
import hashlib
import secrets
import sqlite3
import time
from contextlib import closing
from typing import Any
from urllib.parse import parse_qs, urlparse

import httpx
from google.adk.tools.mcp_tool import McpToolset, StreamableHTTPConnectionParams

from .config import data_dir, eq_headers, request_headers, settings

READ_TOOLS = frozenset({
    "eq_search", "eq_fetch", "eq_retrieve", "eq_entity_search", "eq_entity_get",
    "eq_neighborhood", "eq_graph_image", "eq_document_list", "eq_document_get",
    "eq_document_download", "eq_asset_get", "eq_workspace_stats",
})


def edgequake_toolset(tool_filter: list[str]) -> McpToolset:
    unsafe = set(tool_filter) - READ_TOOLS
    if unsafe:
        raise ValueError(f"Not a read-only EdgeQuake tool: {sorted(unsafe)}")
    # Headers come from header_provider on every list/call (never baked into the
    # connection), so one toolset serves many users: ADK 2.11 pools MCP sessions by
    # a hash of the merged headers, so each user/scope gets its own session (idle
    # ones are evicted after 15 min) and nobody reuses another user's headers.
    return McpToolset(
        connection_params=StreamableHTTPConnectionParams(url=f"{settings().url}/mcp", timeout=30),
        tool_filter=tool_filter,
        header_provider=mcp_headers,
    )


# ---------------------------------------------------------------- per-user MCP tokens
# EdgeQuake's /mcp only accepts API keys or OAuth tokens whose audience is the MCP
# resource (SPEC-154); a web-session JWT is refused. For /v1 runs we turn the
# user's session JWT into a read+query MCP token through EdgeQuake's own OAuth 2.1
# flow (public client, PKCE S256, consent approved with the user's bearer), so MCP
# calls still run with the user's identity and scope. Tokens stay in this process'
# memory only, keyed by a hash of the session JWT, until shortly before expiry.

REDIRECT_URI = "http://127.0.0.1/edgequake-agents/oauth/callback"  # never visited
MCP_SCOPES = "edgequake:read edgequake:query"
_resource: dict[str, str] = {}  # base url -> MCP resource (from PRM, per process)
_tokens: dict[str, tuple[str, float]] = {}  # sha256(session jwt) -> (mcp token, expires_at)


class _ClientRejected(Exception):
    pass


def _is_jwt(token: str) -> bool:
    return token.count(".") == 2 and not token.startswith("eq_")


# The registered OAuth client is kept in agents.db (EQ_DATA_DIR) per EdgeQuake base
# URL and reused across restarts: EdgeQuake has no API to delete clients, so
# registering on every start would leak rows. Re-registered only when EdgeQuake
# rejects it (unknown/invalid client) or the base URL changes.
def _clients_db() -> sqlite3.Connection:
    db = sqlite3.connect(data_dir() / "agents.db")
    db.execute("CREATE TABLE IF NOT EXISTS oauth_clients (base_url TEXT PRIMARY KEY, client_id TEXT NOT NULL, "
               "client_secret TEXT, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)")
    return db


def _stored_client(base: str) -> dict | None:
    with closing(_clients_db()) as db:
        row = db.execute("SELECT client_id, client_secret FROM oauth_clients WHERE base_url = ?", (base,)).fetchone()
    return {"client_id": row[0], "client_secret": row[1]} if row else None


async def _register(c: httpx.AsyncClient, base: str) -> dict:
    reg = await c.post("/oauth/register", json={"client_name": "edgequake-agents", "redirect_uris": [REDIRECT_URI],
                                                "token_endpoint_auth_method": "none"})
    body = reg.raise_for_status().json()
    client = {"client_id": body["client_id"], "client_secret": body.get("client_secret")}
    with closing(_clients_db()) as db, db:
        db.execute("INSERT OR REPLACE INTO oauth_clients (base_url, client_id, client_secret) VALUES (?, ?, ?)",
                   (base, client["client_id"], client["client_secret"]))
    return client


def _rejects_client(r: httpx.Response) -> bool:
    text = r.text.lower()
    return r.status_code in (400, 401) and any(
        s in text for s in ("invalid_client", "unknown oauth client", "redirect_uri not registered"))


async def _authorize(c: httpx.AsyncClient, client: dict, resource: str, session_jwt: str) -> tuple[str, float]:
    verifier = secrets.token_urlsafe(48)
    challenge = base64.urlsafe_b64encode(hashlib.sha256(verifier.encode()).digest()).rstrip(b"=").decode()
    common = {"client_id": client["client_id"], "redirect_uri": REDIRECT_URI, "resource": resource}
    r = await c.post("/oauth/authorize", headers={"Authorization": f"Bearer {session_jwt}"}, data={
        **common, "scope": MCP_SCOPES, "state": secrets.token_urlsafe(8), "code_challenge": challenge,
        "code_challenge_method": "S256", "approve": "1"})
    code = parse_qs(urlparse(r.headers.get("location", "")).query).get("code")
    if not r.is_redirect or not code:
        if _rejects_client(r):
            raise _ClientRejected
        raise PermissionError(f"EdgeQuake MCP authorization failed (HTTP {r.status_code})")
    form = {**common, "grant_type": "authorization_code", "code": code[0], "code_verifier": verifier}
    if client.get("client_secret"):
        form["client_secret"] = client["client_secret"]
    t = await c.post("/oauth/token", data=form)
    if t.status_code != 200:
        if _rejects_client(t):
            raise _ClientRejected
        raise PermissionError(f"EdgeQuake MCP token exchange failed (HTTP {t.status_code})")
    body = t.json()
    return body["access_token"], time.monotonic() + int(body.get("expires_in", 900)) - 60


async def _exchange(session_jwt: str) -> tuple[str, float]:
    base = settings().url
    async with httpx.AsyncClient(base_url=base, timeout=30) as c:
        if base not in _resource:
            prm = (await c.get("/.well-known/oauth-protected-resource/mcp")).raise_for_status().json()
            _resource[base] = prm["resource"]
        stored = _stored_client(base)
        client = stored or await _register(c, base)
        try:
            return await _authorize(c, client, _resource[base], session_jwt)
        except _ClientRejected:
            if not stored:
                raise PermissionError("EdgeQuake rejected the freshly registered MCP OAuth client") from None
            # stored client unknown to this EdgeQuake (e.g. database reset): register once more
            return await _authorize(c, await _register(c, base), _resource[base], session_jwt)


async def mcp_bearer(session_jwt: str) -> str:
    key, now = hashlib.sha256(session_jwt.encode()).hexdigest(), time.monotonic()
    hit = _tokens.get(key)
    if hit and hit[1] > now:
        return hit[0]
    # ponytail: no per-user lock, two concurrent first calls may both exchange; harmless.
    for k in [k for k, (_, exp) in _tokens.items() if exp <= now]:
        del _tokens[k]
    _tokens[key] = await _exchange(session_jwt)
    return _tokens[key][0]


async def mcp_headers(_ctx=None) -> dict[str, str]:
    """header_provider: the current request's headers; on /v1 runs the user's session
    JWT is swapped for their MCP-audience token. Legacy routes keep the API key."""
    h = eq_headers()
    auth = h.get("Authorization", "")
    if request_headers.get() is not None and auth.startswith("Bearer ") and _is_jwt(auth[7:]):
        h["Authorization"] = f"Bearer {await mcp_bearer(auth[7:])}"
    return h


def pin_workspace(tool, args: dict[str, Any], tool_context) -> None:
    """before_tool_callback: drop any model-supplied workspace_id so every MCP call
    stays on the workspace pinned by the X-Workspace-ID header."""
    args.pop("workspace_id", None)


def toolset_from_config(args) -> McpToolset:
    """YAML Agent Config hook: `tools: [{name: edgequake_agents.common.mcp.toolset_from_config,
    args: {tool_filter: [...]}}]`."""
    return edgequake_toolset(list(args.tool_filter))
