"""Query resource — RAG query execution for EdgeQuake.

WHY: Maps to /api/v1/query endpoints. Supports both direct query execution
and streaming via SSE.
"""

from __future__ import annotations

from typing import Any, Literal

from edgequake._streaming import AsyncSSEStream, SSEStream
from edgequake.resources._base import AsyncResource, SyncResource
from edgequake.types.query import QueryResponse, QueryStreamEvent

QueryMode = Literal["local", "global", "hybrid", "naive", "mix", "bypass"]


def _query_body(
    query: str,
    *,
    mode: QueryMode | None = None,
    max_results: int | None = None,
    enable_rerank: bool | None = None,
    top_k: int | None = None,
    rerank: bool | None = None,
    provider: str | None = None,
    model: str | None = None,
    llm_provider: str | None = None,
    llm_model: str | None = None,
    conversation_id: str | None = None,
    session_id: str | None = None,
) -> dict[str, Any]:
    """Build a POST /api/v1/query body using the server field names.

    `top_k` and `rerank` are accepted as aliases of `max_results` and
    `enable_rerank`. The explicit server names win when both are set.
    `mode` is omitted unless the caller sets it, so the server default (`mix`) applies.
    """
    body: dict[str, Any] = {"query": query}
    if mode is not None:
        body["mode"] = mode
    resolved_max = max_results if max_results is not None else top_k
    if resolved_max is not None:
        body["max_results"] = resolved_max
    resolved_rerank = enable_rerank if enable_rerank is not None else rerank
    if resolved_rerank is not None:
        body["enable_rerank"] = resolved_rerank
    resolved_provider = llm_provider or provider
    if resolved_provider:
        body["llm_provider"] = resolved_provider
    resolved_model = llm_model or model
    if resolved_model:
        body["llm_model"] = resolved_model
    resolved_session = session_id or conversation_id
    if resolved_session:
        body["session_id"] = resolved_session
    return body


class QueryResource(SyncResource):
    """Synchronous Query API."""

    def execute(
        self,
        query: str,
        *,
        mode: QueryMode | None = None,
        max_results: int | None = None,
        enable_rerank: bool | None = None,
        top_k: int | None = None,
        rerank: bool | None = None,
        provider: str | None = None,
        model: str | None = None,
        llm_provider: str | None = None,
        llm_model: str | None = None,
        conversation_id: str | None = None,
        session_id: str | None = None,
    ) -> QueryResponse:
        """Execute a RAG query.

        POST /api/v1/query
        """
        body = _query_body(
            query,
            mode=mode,
            max_results=max_results,
            enable_rerank=enable_rerank,
            top_k=top_k,
            rerank=rerank,
            provider=provider,
            model=model,
            llm_provider=llm_provider,
            llm_model=llm_model,
            conversation_id=conversation_id,
            session_id=session_id,
        )
        return self._post("/api/v1/query", json=body, response_type=QueryResponse)

    def stream(
        self,
        query: str,
        *,
        mode: QueryMode | None = None,
        max_results: int | None = None,
        enable_rerank: bool | None = None,
        top_k: int | None = None,
        rerank: bool | None = None,
    ) -> SSEStream[QueryStreamEvent]:
        """Execute a streaming RAG query via SSE.

        POST /api/v1/query/stream
        """
        body = _query_body(
            query,
            mode=mode,
            max_results=max_results,
            enable_rerank=enable_rerank,
            top_k=top_k,
            rerank=rerank,
        )
        response = self._transport.stream("POST", "/api/v1/query/stream", json=body)
        return SSEStream(response, QueryStreamEvent)


class AsyncQueryResource(AsyncResource):
    """Asynchronous Query API."""

    async def execute(
        self,
        query: str,
        *,
        mode: QueryMode | None = None,
        max_results: int | None = None,
        enable_rerank: bool | None = None,
        top_k: int | None = None,
        rerank: bool | None = None,
        provider: str | None = None,
        model: str | None = None,
        llm_provider: str | None = None,
        llm_model: str | None = None,
        conversation_id: str | None = None,
        session_id: str | None = None,
    ) -> QueryResponse:
        body = _query_body(
            query,
            mode=mode,
            max_results=max_results,
            enable_rerank=enable_rerank,
            top_k=top_k,
            rerank=rerank,
            provider=provider,
            model=model,
            llm_provider=llm_provider,
            llm_model=llm_model,
            conversation_id=conversation_id,
            session_id=session_id,
        )
        return await self._post("/api/v1/query", json=body, response_type=QueryResponse)

    async def stream(
        self,
        query: str,
        *,
        mode: QueryMode | None = None,
        max_results: int | None = None,
        enable_rerank: bool | None = None,
        top_k: int | None = None,
        rerank: bool | None = None,
    ) -> AsyncSSEStream[QueryStreamEvent]:
        body = _query_body(
            query,
            mode=mode,
            max_results=max_results,
            enable_rerank=enable_rerank,
            top_k=top_k,
            rerank=rerank,
        )
        response = await self._transport.stream(
            "POST", "/api/v1/query/stream", json=body
        )
        return AsyncSSEStream(response, QueryStreamEvent)
