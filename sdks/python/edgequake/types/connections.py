"""Provider connection types.

Mirrors `ConnectionView`, `UpsertConnection`, and `ProbeResponse` in edgequake-api.
"""

from __future__ import annotations

from pydantic import BaseModel


class Connection(BaseModel):
    """A stored or env-backed provider connection. The API key is never returned."""

    id: str
    tenant_id: str | None = None
    slug: str
    display_name: str
    api_shape: str
    locality: str
    base_url: str
    auth_scheme: str
    key_fingerprint: str | None = None
    key_configured: bool
    timeout_secs: int
    allow_private_network: bool
    source: str
    last_test_ok: bool | None = None
    last_test_error: str | None = None


class ProbeResult(BaseModel):
    """Result of POST /api/v1/providers/test or POST /api/v1/connections/{id}/test."""

    ok: bool
    kind: str
    latency_ms: int
    message: str
    models: list[str] = []
    embedding_dimension: int | None = None
    chat_ok: bool
    embed_ok: bool
    list_ok: bool
