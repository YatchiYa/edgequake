"""EdgeQuake HTTP client for bench047 (auth-disabled local mode)."""

from __future__ import annotations

import time
import uuid
from pathlib import Path
from typing import Any, Optional

import httpx

from .paths import api_base


class EdgeQuakeClient:
    def __init__(
        self,
        base_url: Optional[str] = None,
        tenant_id: str = "00000000-0000-0000-0000-000000000002",
        workspace_id: Optional[str] = None,
        timeout: float = 300.0,
    ):
        self.base = (base_url or api_base()).rstrip("/")
        self.tenant_id = tenant_id
        self.workspace_id = workspace_id
        self.client = httpx.Client(timeout=timeout)

    def headers(self) -> dict[str, str]:
        h = {
            "X-Tenant-ID": self.tenant_id,
            "Accept": "application/json",
        }
        if self.workspace_id:
            h["X-Workspace-ID"] = self.workspace_id
        return h

    def health(self) -> dict[str, Any]:
        r = self.client.get(f"{self.base}/health")
        r.raise_for_status()
        return r.json()

    def create_workspace(
        self,
        name: str,
        slug: Optional[str] = None,
        *,
        llm_provider: str = "mistral",
        llm_model: str = "mistral-small-latest",
        embedding_provider: str = "mistral",
        embedding_model: str = "mistral-embed",
        embedding_dimension: int = 1024,
        vision_llm_provider: str = "mistral",
        vision_llm_model: str = "mistral-small-latest",
    ) -> dict[str, Any]:
        slug = slug or f"bench047-{uuid.uuid4().hex[:10]}"
        payload = {
            "name": name,
            "slug": slug,
            "description": "SPEC-047 MMLongBench-Doc RAG evaluation workspace",
            "llm_provider": llm_provider,
            "llm_model": llm_model,
            "embedding_provider": embedding_provider,
            "embedding_model": embedding_model,
            "embedding_dimension": embedding_dimension,
            "vision_llm_provider": vision_llm_provider,
            "vision_llm_model": vision_llm_model,
        }
        r = self.client.post(
            f"{self.base}/api/v1/tenants/{self.tenant_id}/workspaces",
            headers=self.headers(),
            json=payload,
        )
        if r.status_code >= 400:
            # Do not silently reuse an existing workspace — it may pin different models.
            raise RuntimeError(
                f"create_workspace failed: {r.status_code} {r.text[:800]} "
                f"(slug={slug!r}; use a unique slug so LLM/vision pins are not stale)"
            )
        data = r.json()
        self.workspace_id = str(data.get("id") or data.get("workspace_id"))
        return data

    def upload_pdf(
        self,
        path: Path,
        *,
        enable_vision: bool = True,
        vision_provider: str = "mistral",
        vision_model: str = "mistral-small-latest",
        pdf_parser_backend: str = "vision",
        force_reindex: bool = False,
        process_options: str | None = None,
    ) -> dict[str, Any]:
        data = {
            "enable_vision": str(enable_vision).lower(),
            "vision_provider": vision_provider,
            "vision_model": vision_model,
            "pdf_parser_backend": pdf_parser_backend,
            "title": path.name,
            "force_reindex": str(force_reindex).lower(),
        }
        if process_options:
            data["process_options"] = process_options
        with path.open("rb") as f:
            files = {"file": (path.name, f, "application/pdf")}
            r = self.client.post(
                f"{self.base}/api/v1/documents/pdf",
                headers=self.headers(),
                data=data,
                files=files,
            )
        if r.status_code >= 400:
            raise RuntimeError(f"upload_pdf failed: {r.status_code} {r.text[:500]}")
        return r.json()

    def pdf_status(self, pdf_id: str) -> dict[str, Any]:
        r = self.client.get(
            f"{self.base}/api/v1/documents/pdf/{pdf_id}",
            headers=self.headers(),
        )
        r.raise_for_status()
        return r.json()

    def document_status(self, document_id: str) -> dict[str, Any]:
        r = self.client.get(
            f"{self.base}/api/v1/documents/{document_id}",
            headers=self.headers(),
        )
        r.raise_for_status()
        return r.json()

    def recover_stuck(self, *, stuck_threshold_minutes: int = 1) -> dict[str, Any]:
        """Requeue documents stuck in processing (restart_from_scratch=false — keeps markdown)."""
        r = self.client.post(
            f"{self.base}/api/v1/documents/recover-stuck",
            headers=self.headers(),
            json={"stuck_threshold_minutes": stuck_threshold_minutes},
        )
        if r.status_code >= 400:
            raise RuntimeError(f"recover_stuck failed: {r.status_code} {r.text[:500]}")
        return r.json()

    def reprocess_failed(
        self,
        *,
        document_id: str | None = None,
        mode: str = "entities_only",
        max_documents: int = 20,
        force: bool = False,
    ) -> dict[str, Any]:
        """Soft reprocess failed/cancelled docs. Default mode keeps stored markdown.

        Set ``force=True`` with ``document_id`` to requeue a stuck processing doc
        (still entities_only / no re-vision unless mode=full).
        """
        payload: dict[str, Any] = {
            "mode": mode,
            "max_documents": max_documents,
            "force": force,
        }
        if document_id:
            payload["document_id"] = document_id
        r = self.client.post(
            f"{self.base}/api/v1/documents/reprocess",
            headers=self.headers(),
            json=payload,
        )
        if r.status_code >= 400:
            raise RuntimeError(f"reprocess_failed failed: {r.status_code} {r.text[:500]}")
        return r.json()

    def wait_pdf(
        self,
        pdf_id: str,
        *,
        poll_s: float = 10.0,
        timeout_s: float = 7200.0,
    ) -> dict[str, Any]:
        """Wait until PDF conversion *and* document pipeline are terminal.

        PDF status can flip to ``completed`` while the document is still
        chunking/extracting. G-A fidelity and query need the document done.

        Telemetry (SPEC-047 P6): each poll prints stage, progress, entity/rel
        counters, and warning/stage_message so operators can follow embed/merge.
        """
        t0 = time.time()
        last: dict[str, Any] = {}
        last_sig = ""
        while time.time() - t0 < timeout_s:
            last = self.pdf_status(pdf_id)
            pdf_status = (last.get("status") or "").lower()
            elapsed = int(time.time() - t0)
            print(f"  pdf {pdf_id[:8]}… status={pdf_status} elapsed={elapsed}s")
            if pdf_status in {"failed"}:
                return last
            doc_id = last.get("document_id")
            if not doc_id:
                # Some responses nest document_id under metadata
                doc_id = (last.get("metadata") or {}).get("document_id")
            if doc_id:
                try:
                    doc = self.document_status(doc_id)
                    ds = (doc.get("status") or "").lower()
                    stage = (
                        doc.get("current_stage")
                        or (doc.get("metadata") or {}).get("current_stage")
                        or ""
                    )
                    prog = doc.get("stage_progress")
                    msg = (
                        doc.get("stage_message")
                        or doc.get("warning_message")
                        or ""
                    )
                    ents = doc.get("entity_count")
                    chunks = doc.get("chunk_count")
                    rels = doc.get("relationship_count")
                    sig = f"{ds}|{stage}|{prog}|{msg}|{ents}|{chunks}|{rels}"
                    if sig != last_sig:
                        last_sig = sig
                        bits = [f"status={ds}"]
                        if stage:
                            bits.append(f"stage={stage}")
                        if prog is not None:
                            try:
                                bits.append(f"progress={float(prog):.0%}")
                            except (TypeError, ValueError):
                                bits.append(f"progress={prog}")
                        if chunks is not None:
                            bits.append(f"chunks={chunks}")
                        if ents is not None:
                            bits.append(f"ents={ents}")
                        if rels is not None:
                            bits.append(f"rels={rels}")
                        print(f"    doc {str(doc_id)[:8]}… " + " ".join(bits))
                        if msg:
                            # Truncate noisy merge banners but keep counters visible.
                            print(f"      └─ {str(msg)[:160]}")
                    last["document_status"] = ds
                    last["document"] = doc
                    last["document_id"] = doc_id
                    if ds in {"completed", "indexed", "duplicate"}:
                        last["status"] = "completed"
                        return last
                    if ds in {"failed"}:
                        last["status"] = "failed"
                        return last
                    # PDF may already be completed while doc still processes — keep polling.
                except Exception as e:
                    print(f"    doc poll err: {e}")
            elif pdf_status in {"completed", "duplicate"}:
                # No document_id yet — keep waiting briefly for linkage.
                pass
            time.sleep(poll_s)
        raise TimeoutError(f"PDF {pdf_id} not done after {timeout_s}s; last={last}")

    def get_markdown(self, document_id: str) -> str:
        """GET /api/v1/query/context/artifacts/markdown/{document_id}."""
        r = self.client.get(
            f"{self.base}/api/v1/query/context/artifacts/markdown/{document_id}",
            headers=self.headers(),
        )
        if r.status_code >= 400:
            raise RuntimeError(f"get_markdown failed: {r.status_code} {r.text[:500]}")
        data = r.json()
        md = (data.get("markdown") or {}).get("markdown")
        if md is None:
            md = data.get("content") or data.get("markdown")
        if not isinstance(md, str):
            raise RuntimeError(f"get_markdown: unexpected payload keys={list(data.keys())}")
        return md

    def query(
        self,
        question: str,
        mode: str = "hybrid",
        *,
        document_ids: Optional[list[str]] = None,
        include_references: bool = True,
        include_subgraph: bool = False,
    ) -> dict[str, Any]:
        """POST /api/v1/query — returns full JSON including `sources` (page_start).

        document_ids: optional DocumentFilter.document_ids (SPEC-005). Use for
        single-PDF scoped retrieve (W2). Never pass gold evidence_pages.
        """
        payload: dict[str, Any] = {
            "query": question,
            "mode": mode,
            "include_references": include_references,
            "include_subgraph": include_subgraph,
        }
        if document_ids:
            payload["document_filter"] = {"document_ids": document_ids}
        r = self.client.post(
            f"{self.base}/api/v1/query",
            headers={**self.headers(), "Content-Type": "application/json"},
            json=payload,
        )
        if r.status_code >= 400:
            raise RuntimeError(f"query failed: {r.status_code} {r.text[:500]}")
        return r.json()
