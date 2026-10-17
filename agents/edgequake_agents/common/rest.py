"""Minimal read-only REST client for what EdgeQuake's MCP server lacks
(raw subgraphs with max_nodes, graph facets, popular labels, communities, and the
full RAG pipeline of the Query page: POST /query and /query/context)."""

from __future__ import annotations

from typing import Any

import httpx

from .config import eq_headers, settings


async def post(path: str, body: dict, timeout: float = 90) -> Any:
    """POST /api/v1{path} (JSON) with the current request's credentials. Raises on HTTP
    errors. Only used for read endpoints (query, query/context): they write nothing."""
    async with httpx.AsyncClient(base_url=f"{settings().url}/api/v1", headers=eq_headers(), timeout=timeout) as c:
        r = await c.post(path, json=body)
        r.raise_for_status()
        return r.json()


async def get(path: str, **params: Any) -> Any:
    """GET /api/v1{path} with the current request's credentials (config.eq_headers:
    the user's token on /v1, the configured API key otherwise). Raises on HTTP errors."""
    s = settings()
    params = {k: v for k, v in params.items() if v is not None}
    async with httpx.AsyncClient(base_url=f"{s.url}/api/v1", headers=eq_headers(), timeout=30) as c:
        r = await c.get(path, params=params)
        r.raise_for_status()
        return r.json()
