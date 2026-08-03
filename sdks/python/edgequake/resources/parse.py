"""Parse resource — SPEC-094 stateless PDF → Markdown."""

from __future__ import annotations

from pathlib import Path
from typing import Any, BinaryIO

from pydantic import BaseModel, Field

from edgequake.resources._base import AsyncResource, SyncResource


class ParseOptions(BaseModel):
    backend: str | None = None
    provider: str | None = None
    model: str | None = None
    dpi: int | None = None
    concurrency: int | None = None
    pages: str | None = None
    table_method: str | None = None
    emit_assets: bool | None = None
    allow_fallback: bool | None = None
    include_page_timings: bool | None = None
    force_async: bool | None = Field(default=None, alias="async")

    model_config = {"populate_by_name": True}


class ParseMetrics(BaseModel):
    total_ms: int
    render_ms: int | None = None
    ocr_ms: int | None = None
    assemble_ms: int | None = None
    pages_per_second: float | None = None
    prompt_tokens: int | None = None
    completion_tokens: int | None = None
    estimated_cost_usd: float | None = None


class ParseResponse(BaseModel):
    markdown: str
    backend: str
    backend_effective: str
    fallback_applied: bool
    page_count: int
    metrics: ParseMetrics
    page_timings: list[dict[str, Any]] | None = None
    warnings: list[str] = Field(default_factory=list)
    request_id: str


class ParseAsyncAccepted(BaseModel):
    job_id: str
    status: str
    request_id: str


class ParseJobStatusResponse(BaseModel):
    job_id: str
    status: str
    result: ParseResponse | None = None
    error: dict[str, str] | None = None
    request_id: str


class ParseBackendsResponse(BaseModel):
    backends: list[dict[str, Any]]
    limits: dict[str, Any]
    default_backend: str


class ParseResource(SyncResource):
    """Synchronous parse API."""

    def parse(
        self,
        file: Path | BinaryIO,
        *,
        options: ParseOptions | dict[str, Any] | None = None,
        filename: str | None = None,
    ) -> dict[str, Any]:
        """POST /api/v1/parse — returns ParseResponse or ParseAsyncAccepted dict."""
        meta: dict[str, str] = {}
        if options is not None:
            if isinstance(options, ParseOptions):
                payload = options.model_dump(by_alias=True, exclude_none=True)
            else:
                payload = options
            import json

            meta["options"] = json.dumps(payload)
        response = self._transport.upload(
            "/api/v1/parse",
            file=file,
            filename=filename,
            metadata=meta or None,
        )
        return response.json()

    def backends(self) -> ParseBackendsResponse:
        return self._get("/api/v1/parse/backends", response_type=ParseBackendsResponse)

    def job(self, job_id: str) -> ParseJobStatusResponse:
        return self._get(
            f"/api/v1/parse/jobs/{job_id}",
            response_type=ParseJobStatusResponse,
        )


class AsyncParseResource(AsyncResource):
    """Asynchronous parse API."""

    async def parse(
        self,
        file: Path | BinaryIO,
        *,
        options: ParseOptions | dict[str, Any] | None = None,
        filename: str | None = None,
    ) -> dict[str, Any]:
        meta: dict[str, str] = {}
        if options is not None:
            if isinstance(options, ParseOptions):
                payload = options.model_dump(by_alias=True, exclude_none=True)
            else:
                payload = options
            import json

            meta["options"] = json.dumps(payload)
        response = await self._transport.upload(
            "/api/v1/parse",
            file=file,
            filename=filename,
            metadata=meta or None,
        )
        return response.json()

    async def backends(self) -> ParseBackendsResponse:
        return await self._get(
            "/api/v1/parse/backends", response_type=ParseBackendsResponse
        )

    async def job(self, job_id: str) -> ParseJobStatusResponse:
        return await self._get(
            f"/api/v1/parse/jobs/{job_id}",
            response_type=ParseJobStatusResponse,
        )
