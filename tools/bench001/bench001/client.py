"""EdgeQuake REST client for SPEC-001 (auth-disabled local + Mistral pins)."""

from __future__ import annotations

import os
import re
import time
import uuid
from collections.abc import Callable
from pathlib import Path
from typing import Any, Optional

import httpx

from .profiles import ProviderPins, active_pins

ProgressCb = Callable[[dict[str, Any]], None]


DEFAULT_TENANT = "00000000-0000-0000-0000-000000000002"


class EdgeQuakeClient:
    def __init__(
        self,
        base_url: str,
        *,
        workspace: str = "bench001-smoke",
        workspace_id: Optional[str] = None,
        tenant_id: str = DEFAULT_TENANT,
        timeout: float = 300.0,
        api_key: Optional[str] = None,
    ) -> None:
        self.base = base_url.rstrip("/")
        self.workspace_name = workspace
        self.workspace_id: Optional[str] = workspace_id or os.environ.get(
            "BENCH001_EQ_WORKSPACE_ID"
        )
        self.tenant_id = tenant_id
        self.timeout = timeout
        self._http = httpx.Client(timeout=timeout)
        key = api_key or os.environ.get("EDGEQUAKE_API_KEY") or os.environ.get("API_KEY")
        self._api_key = key

    def headers(self) -> dict[str, str]:
        h = {
            "X-Tenant-ID": self.tenant_id,
            "Accept": "application/json",
            "Content-Type": "application/json",
        }
        if self.workspace_id:
            h["X-Workspace-ID"] = self.workspace_id
        if self._api_key:
            h["X-API-Key"] = self._api_key
            h["Authorization"] = f"Bearer {self._api_key}"
        return h

    def health(self) -> dict[str, Any]:
        r = self._http.get(f"{self.base}/health")
        r.raise_for_status()
        return r.json()

    def workspace_exists(self, workspace_id: str | None = None) -> bool:
        """Return True if the workspace is visible to the API.

        Prefer tenant workspace list — GET-by-id can 404 even when the workspace
        is listed (bench Acc warm-index path).
        """
        wid = workspace_id or self.workspace_id
        if not wid:
            return False
        headers = {k: v for k, v in self.headers().items() if k.lower() != "x-workspace-id"}
        r = self._http.get(
            f"{self.base}/api/v1/tenants/{self.tenant_id}/workspaces",
            headers=headers,
        )
        if r.status_code == 200:
            data = r.json()
            items = data if isinstance(data, list) else data.get("items") or data.get("workspaces") or []
            for w in items:
                if not isinstance(w, dict):
                    continue
                if str(w.get("id") or w.get("workspace_id") or "") == str(wid):
                    return True
        # Fallback: GET-by-id (some deployments only expose this).
        r2 = self._http.get(
            f"{self.base}/api/v1/tenants/{self.tenant_id}/workspaces/{wid}",
            headers=headers,
        )
        return r2.status_code == 200

    def ensure_workspace(self, pins: ProviderPins | None = None) -> str:
        """Create a fresh workspace with active provider pins (unique slug).

        If ``BENCH001_EQ_WORKSPACE_ID`` / ``self.workspace_id`` points at a
        missing workspace, recreate instead of silently 404-ing on query.
        """
        if self.workspace_id and self.workspace_exists(self.workspace_id):
            return self.workspace_id
        if self.workspace_id:
            print(
                f"EQ workspace {self.workspace_id} not found — creating a fresh workspace",
                flush=True,
            )
            self.workspace_id = None
        p = pins or active_pins()
        slug = f"{self.workspace_name}-{uuid.uuid4().hex[:10]}"
        payload = {
            "name": self.workspace_name,
            "slug": slug,
            "description": "SPEC-001 GraphRAG-Bench dual-SUT workspace",
            "llm_provider": p.llm_provider,
            "llm_model": p.llm_model,
            "embedding_provider": p.embedding_provider,
            "embedding_model": p.embedding_model,
            "embedding_dimension": p.embedding_dim,
            "vision_llm_provider": p.vision_provider,
            "vision_llm_model": p.vision_model,
        }
        # Avoid sending a stale X-Workspace-ID on create.
        headers = {k: v for k, v in self.headers().items() if k.lower() != "x-workspace-id"}
        r = self._http.post(
            f"{self.base}/api/v1/tenants/{self.tenant_id}/workspaces",
            headers=headers,
            json=payload,
        )
        if r.status_code >= 400:
            raise RuntimeError(f"create_workspace failed: {r.status_code} {r.text[:800]}")
        data = r.json()
        self.workspace_id = str(data.get("id") or data.get("workspace_id"))
        return self.workspace_id

    def upload_text(
        self,
        content: str,
        *,
        title: str,
        chunk_size: int = 1200,
        chunk_overlap: int = 100,
        async_processing: bool = True,
    ) -> dict[str, Any]:
        if not self.workspace_id:
            self.ensure_workspace()
        # API SSOT field is chunk_token_size (alias chunk_size also accepted).
        # 028 B2: Acc force-ingest must pin gleaning + markdown breadcrumbs
        # (defaults match API, but env allows labeled ablations).
        max_gleaning = int(os.environ.get("BENCH001_MAX_GLEANING", "1"))
        enable_gleaning = (
            os.environ.get("BENCH001_ENABLE_GLEANING", "1").strip().lower()
            not in {"0", "false", "off", "no"}
        )
        chunk_strategy = (
            os.environ.get("BENCH001_CHUNK_STRATEGY") or "markdown"
        ).strip().lower()
        payload = {
            "content": content,
            "title": title if title.endswith(".md") else f"{title}.md",
            "async_processing": async_processing,
            "enable_gleaning": enable_gleaning,
            "max_gleaning": max_gleaning,
            "chunk_strategy": chunk_strategy,
            "chunk_options": {
                "chunk_token_size": int(chunk_size),
                "chunk_overlap_token_size": int(chunk_overlap),
            },
        }
        r = self._http.post(
            f"{self.base}/api/v1/documents",
            headers=self.headers(),
            json=payload,
        )
        if r.status_code >= 400:
            raise RuntimeError(f"upload failed: {r.status_code} {r.text[:800]}")
        return r.json()

    def get_document(self, document_id: str) -> dict[str, Any]:
        r = self._http.get(
            f"{self.base}/api/v1/documents/{document_id}",
            headers=self.headers(),
        )
        r.raise_for_status()
        return r.json()

    def list_documents(self, *, page_size: int = 50) -> list[dict[str, Any]]:
        r = self._http.get(
            f"{self.base}/api/v1/documents",
            headers=self.headers(),
            params={"page_size": page_size},
        )
        r.raise_for_status()
        data = r.json()
        docs = data.get("documents") if isinstance(data, dict) else data
        return list(docs or [])

    def resolve_document(self, document_id: str) -> dict[str, Any]:
        """Fetch document; fall back to list match when GET-by-id 404s."""
        bare = document_id.split(":")[-1]
        try:
            return self.get_document(document_id)
        except httpx.HTTPStatusError as exc:
            if exc.response.status_code != 404:
                raise
        try:
            return self.get_document(bare)
        except httpx.HTTPStatusError as exc:
            if exc.response.status_code != 404:
                raise
        for doc in self.list_documents():
            did = str(doc.get("id") or "")
            if did == document_id or did.endswith(bare) or did.split(":")[-1] == bare:
                return doc
        raise RuntimeError(f"Document {document_id} not found (GET + list miss)")

    def get_task(self, task_id: str) -> dict[str, Any]:
        r = self._http.get(
            f"{self.base}/api/v1/tasks/{task_id}",
            headers=self.headers(),
        )
        r.raise_for_status()
        return r.json()

    def wait_document(
        self,
        document_id: str,
        *,
        task_id: Optional[str] = None,
        timeout_s: float = 7200.0,
        poll_s: float = 5.0,
        progress_cb: ProgressCb | None = None,
    ) -> dict[str, Any]:
        """Block until document is fully indexed (not merely task=`indexed`).

        Fail-closed on ``failed``/``error``. Task ``indexed`` alone is not enough
        while ``display_status=storing`` / graph merge is still running.
        Prints a heartbeat every ~10s with pct + ETA when ``stage_progress`` exists.
        """
        deadline = time.time() + timeout_s
        started = time.time()
        last: dict[str, Any] = {}
        last_log = 0.0
        # First principles: `indexed` alone is not ready — graph merge / saga
        # compensation can still roll back vectors (disk-full ENOSPC, etc.).
        terminal_ok = {"completed", "processed", "done", "ready", "duplicate"}
        terminal_bad = {"failed", "error", "cancelled"}
        in_flight = {
            "pending",
            "processing",
            "indexing",
            "indexed",  # treat as in-flight until completed/processed
            "storing",
            "running",
            "queued",
            "partial_failure",
        }

        def _doc_status(doc: dict[str, Any]) -> str:
            return str(
                doc.get("status")
                or doc.get("display_status")
                or doc.get("processing_status")
                or ""
            ).lower()

        def _fail_msg(doc: dict[str, Any], *, kind: str) -> str:
            msg = (
                doc.get("stage_message")
                or doc.get("warning_message")
                or doc.get("error")
                or ""
            )
            return (
                f"{kind} {document_id} failed status={_doc_status(doc)} "
                f"stage={doc.get('current_stage')} msg={str(msg)[:240]}"
            )

        def _storage_error_count(doc: dict[str, Any]) -> int:
            for key in ("storage_error_count", "knowledge_graph_error_count"):
                raw = doc.get(key)
                if raw is None:
                    continue
                try:
                    return int(raw)
                except (TypeError, ValueError):
                    continue
            errs = doc.get("storage_errors") or doc.get("errors")
            if isinstance(errs, list):
                return len(errs)
            return 0

        def _doc_ready(doc: dict[str, Any]) -> bool:
            st = _doc_status(doc)
            if st in terminal_bad:
                raise RuntimeError(_fail_msg(doc, kind="document"))
            # Non-fatal storage errors still mean the graph was rolled back /
            # incomplete — Acc must fail closed (032 B3b disk-full race).
            if _storage_error_count(doc) > 0:
                raise RuntimeError(
                    _fail_msg(doc, kind="document")
                    + f" storage_error_count={_storage_error_count(doc)}"
                )
            warn = str(doc.get("warning_message") or "").lower()
            if "storage error" in warn or "merge error" in warn or "no space left" in warn:
                raise RuntimeError(_fail_msg(doc, kind="document"))
            if st in in_flight or not st:
                return False
            ui = str(doc.get("ui_phase") or "").lower()
            if ui in {"running", "processing"}:
                return False
            if ui == "terminal" and st not in terminal_ok:
                raise RuntimeError(_fail_msg(doc, kind="document"))
            stage = str(doc.get("current_stage") or "").lower()
            if stage in {"storing", "merging", "embedding", "extracting", "chunking"}:
                return False
            return st in terminal_ok

        while time.time() < deadline:
            ts = ""
            task_stage = ""
            if task_id:
                try:
                    t = self.get_task(task_id)
                    ts = str(t.get("status") or "").lower()
                    meta = t.get("metadata") or {}
                    task_stage = str(
                        t.get("stage")
                        or meta.get("stage")
                        or meta.get("pipeline_stage")
                        or ""
                    )
                    if ts in terminal_bad:
                        raise RuntimeError(
                            f"task {task_id} failed status={ts} "
                            f"error={t.get('error') or t.get('message') or t}"
                        )
                except httpx.HTTPStatusError:
                    pass
            try:
                last = self.resolve_document(document_id)
            except (httpx.HTTPStatusError, RuntimeError):
                last = {}

            elapsed = max(time.time() - started, 1e-6)
            pct: float | None = None
            if last:
                try:
                    pct = float(last.get("stage_progress"))
                    if pct > 1.0:
                        pct = pct / 100.0
                    pct = max(0.0, min(pct, 1.0))
                except (TypeError, ValueError):
                    pct = None
            eta_s = None
            if pct is not None and 0.05 < pct < 0.99:
                eta_s = elapsed * (1.0 - pct) / pct

            now = time.time()
            if now - last_log >= 10.0:
                dst = _doc_status(last) if last else "?"
                stage = str((last or {}).get("current_stage") or task_stage or "")
                msg = str(
                    (last or {}).get("stage_message")
                    or (last or {}).get("warning_message")
                    or ""
                )[:90]
                pct_s = f" pct={pct*100:.0f}%" if pct is not None else ""
                from .progress import format_duration

                eta_part = f" eta={format_duration(eta_s)}" if eta_s is not None else ""
                chunks = (last or {}).get("chunk_count")
                chunk_s = f" chunks={chunks}" if chunks not in (None, 0) else ""
                print(
                    f"  ingest heartbeat elapsed={format_duration(elapsed)} "
                    f"task={ts or 'n/a'} doc={dst} stage={stage or 'n/a'}"
                    f"{pct_s}{eta_part}{chunk_s}"
                    + (f" | {msg}" if msg else ""),
                    flush=True,
                )
                if progress_cb is not None:
                    progress_cb(
                        {
                            "elapsed_s": elapsed,
                            "task_status": ts,
                            "doc_status": dst,
                            "stage": stage,
                            "pct": pct,
                            "eta_s": eta_s,
                            "chunk_count": chunks,
                            "message": msg,
                        }
                    )
                last_log = now

            if last and _doc_ready(last):
                from .progress import format_duration

                print(
                    f"  ingest ready elapsed={format_duration(elapsed)} "
                    f"status={_doc_status(last)} "
                    f"chunks={last.get('chunk_count')}",
                    flush=True,
                )
                return last
            time.sleep(poll_s)

        raise TimeoutError(
            f"document {document_id} not ready after {timeout_s}s "
            f"(last_status={_doc_status(last) if last else 'missing'}): "
            f"{str((last or {}).get('stage_message') or last)[:300]}"
        )

    def count_entity_vectors(self) -> int | None:
        """Count entity rows in the workspace vector table (032 density gate)."""
        wid = (self.workspace_id or "").strip()
        if not wid:
            return None
        try:
            import psycopg2
        except ImportError:
            return None

        # Prefer DATABASE_URL; fall back to start-script parse used by audit.
        url = (os.environ.get("DATABASE_URL") or "").strip()
        if not url:
            start = Path("/tmp/edgequake-start.sh")
            if start.is_file():
                for line in start.read_text(encoding="utf-8").splitlines():
                    m = re.match(r'^export\s+DATABASE_URL="([^"]*)"', line)
                    if m:
                        url = m.group(1)
                        break
        if not url:
            return None
        prefix = wid.replace("-", "")[:8]
        table = f"eq_eq_default_ws_{prefix}_vectors"
        conn = None
        try:
            conn = psycopg2.connect(url.split("?")[0])
            with conn.cursor() as cur:
                cur.execute(
                    """
                    SELECT COUNT(*) FROM information_schema.tables
                    WHERE table_schema='public' AND table_name=%s
                    """,
                    (table,),
                )
                if int(cur.fetchone()[0]) == 0:
                    return 0
                cur.execute(
                    f"""
                    SELECT COUNT(*) FROM {table}
                    WHERE metadata->>'type' = 'entity'
                    """
                )
                return int(cur.fetchone()[0])
        except Exception:  # noqa: BLE001
            return None
        finally:
            if conn is not None:
                conn.close()

    def assert_ingest_settled(
        self,
        document_id: str,
        *,
        min_entity_vectors: int = 100,
        settle_s: float = 20.0,
        poll_s: float = 2.0,
    ) -> int:
        """Fail closed if saga rollback wipes vectors after premature ``completed``.

        032 B3b: ENOSPC during relationship merge rolled back 4k+ nodes after
        ``wait_document`` had already returned ``completed``.
        """
        deadline = time.time() + max(settle_s, 1.0)
        last_n: int | None = None
        while time.time() < deadline:
            doc = self.resolve_document(document_id)
            st = str(
                doc.get("status") or doc.get("display_status") or ""
            ).lower()
            if st in {"failed", "error", "cancelled"}:
                raise RuntimeError(
                    f"ingest settle: document {document_id} became {st} "
                    f"msg={str(doc.get('stage_message') or doc.get('warning_message') or '')[:240]}"
                )
            err_n = doc.get("storage_error_count")
            try:
                if err_n is not None and int(err_n) > 0:
                    raise RuntimeError(
                        f"ingest settle: storage_error_count={err_n} on {document_id}"
                    )
            except (TypeError, ValueError):
                pass
            last_n = self.count_entity_vectors()
            time.sleep(poll_s)
        if last_n is None:
            print(
                "  ingest settle: skipped density check (no DATABASE_URL/psycopg2)",
                flush=True,
            )
            return -1
        if last_n < min_entity_vectors:
            raise RuntimeError(
                f"ingest settle: entity vectors={last_n} < {min_entity_vectors} "
                f"for workspace {self.workspace_id} (likely saga rollback / ENOSPC)"
            )
        print(
            f"  ingest settle OK entity_vectors={last_n} "
            f"(min={min_entity_vectors}, window={settle_s:.0f}s)",
            flush=True,
        )
        return last_n

    def query(
        self,
        question: str,
        *,
        mode: str = "mix",
        system_prompt: str | None = None,
        question_type: str | None = None,
        max_results: int | None = None,
        rerank_top_k: int | None = None,
    ) -> dict[str, Any]:
        if not self.workspace_id:
            raise RuntimeError("workspace_id not set — call ensure_workspace() first")
        # GraphRAG-Bench generation_eval needs retrieved context for Faithfulness
        # (Creative). Prefer full chunk text over citation-truncated snippets.
        from .fair_pins import eq_query_overrides

        fair = eq_query_overrides()
        payload: dict[str, Any] = {
            "query": question,
            "mode": mode,
            "include_references": fair["include_references"],
            "content_granularity": fair["content_granularity"],
            "max_results": max_results if max_results is not None else fair["max_results"],
            "rerank_top_k": rerank_top_k if rerank_top_k is not None else fair["rerank_top_k"],
            "enable_rerank": bool(fair.get("enable_rerank", True)),
        }
        # Per-request SUT LLM pin (fair dual-SUT when using larger Mistral).
        try:
            from .profiles import active_pins

            pins = active_pins()
            payload["llm_provider"] = pins.llm_provider
            payload["llm_model"] = pins.llm_model
        except Exception:  # noqa: BLE001
            pass
        if system_prompt:
            payload["system_prompt"] = system_prompt
        if question_type and str(question_type).strip():
            payload["question_type"] = str(question_type).strip()
        r = self._http.post(
            f"{self.base}/api/v1/query",
            headers=self.headers(),
            json=payload,
        )
        if r.status_code >= 400:
            raise RuntimeError(f"query failed: {r.status_code} {r.text[:800]}")
        return r.json()

    def extract_answer(self, resp: dict[str, Any]) -> tuple[str, str]:
        """Return (answer_text, context_blob).

        EdgeQuake ``SourceReference`` stores text in ``snippet`` (not ``content``).
        """
        answer = (
            resp.get("response")
            or resp.get("answer")
            or resp.get("result")
            or ""
        )
        if isinstance(answer, dict):
            answer = answer.get("content") or answer.get("text") or str(answer)
        sources = resp.get("sources") or resp.get("context") or []
        if isinstance(sources, list):
            parts = []
            for s in sources:
                if isinstance(s, str):
                    parts.append(s)
                elif isinstance(s, dict):
                    parts.append(
                        str(
                            s.get("snippet")
                            or s.get("content")
                            or s.get("text")
                            or s.get("chunk")
                            or ""
                        )
                    )
            context = "\n-----\n".join(p for p in parts if p)
        else:
            context = str(sources) if sources else ""
        return str(answer).strip(), context
