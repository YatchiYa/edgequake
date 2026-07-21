"""LightRAG adapter for SPEC-001 (Mistral Small + mistral-embed)."""

from __future__ import annotations

import os
import time
from pathlib import Path
from typing import Any

from .paths import cache_root, lightrag_repo
from .profiles import (
    active_pins,
    query_concurrency,
    sut_api_key,
)


def available() -> bool:
    repo = lightrag_repo()
    if repo is None:
        return False
    try:
        import importlib.util

        return importlib.util.find_spec("lightrag") is not None or (repo / "lightrag").exists()
    except Exception:  # noqa: BLE001
        return (repo / "lightrag").exists()


def workspace_dir(stage: str) -> Path:
    d = cache_root() / "lightrag" / stage
    d.mkdir(parents=True, exist_ok=True)
    return d


def _ensure_llm_env() -> tuple[str, Any]:
    pins = active_pins()
    key = sut_api_key(pins)
    if not key:
        raise RuntimeError(
            "API key required for LightRAG SUT pins "
            "(MISTRAL_API_KEY / OPENAI_API_KEY / LLM_API_KEY)"
        )
    os.environ.setdefault("OPENAI_API_KEY", key)
    os.environ.setdefault("OPENAI_BASE_URL", pins.llm_base_url)
    return key, pins


async def _insert_and_query_async(
    *,
    texts: list[str],
    questions: list[dict[str, Any]],
    stage: str,
    mode: str = "mix",
    working_dir: Path,
    force_ingest: bool = False,
    concurrency: int | None = None,
) -> list[dict[str, Any]]:
    """Insert corpus and query with LightRAG mix mode under Mistral pins."""
    import asyncio

    repo = lightrag_repo()
    if repo is not None:
        import sys

        sys.path.insert(0, str(repo))

    key, pins = _ensure_llm_env()

    import numpy as np
    import httpx

    from lightrag import LightRAG, QueryParam
    from lightrag.llm.openai import openai_complete_if_cache
    from lightrag.utils import wrap_embedding_func_with_attrs

    llm_model = pins.llm_model
    emb_model = pins.embedding_model
    emb_dim = pins.embedding_dim
    base_url = pins.llm_base_url

    async def sut_complete(
        prompt,
        system_prompt=None,
        history_messages=None,
        **kwargs,
    ):
        if history_messages is None:
            history_messages = []
        return await openai_complete_if_cache(
            llm_model,
            prompt,
            system_prompt=system_prompt,
            history_messages=history_messages,
            base_url=base_url,
            api_key=key,
            **kwargs,
        )

    @wrap_embedding_func_with_attrs(
        embedding_dim=emb_dim,
        max_token_size=8192,
        model_name=emb_model,
    )
    async def sut_embed(texts: list[str], **kwargs):
        # Direct embeddings HTTP — avoid openai_embed's dimensions reshape
        # (mistral-embed rejects `dimensions`).
        del kwargs
        payload: dict[str, Any] = {"model": emb_model, "input": list(texts)}
        async with httpx.AsyncClient(timeout=120.0) as client:
            r = await client.post(
                f"{base_url.rstrip('/')}/embeddings",
                headers={
                    "Authorization": f"Bearer {key}",
                    "Content-Type": "application/json",
                },
                json=payload,
            )
            r.raise_for_status()
            data = r.json()["data"]
            data = sorted(data, key=lambda d: d["index"])
            vecs = [np.array(d["embedding"], dtype=np.float32) for d in data]
            if len(vecs) != len(texts):
                raise RuntimeError(
                    f"embed count mismatch: expected {len(texts)} got {len(vecs)}"
                )
            if vecs and vecs[0].shape[0] != emb_dim:
                raise RuntimeError(
                    f"embed dim mismatch: expected {emb_dim} got {vecs[0].shape[0]}"
                )
            return np.stack(vecs, axis=0)

    # Acc latency fairness (063): LightRAG defaults enable_llm_cache=True and the
    # warm working_dir retains keywords+query hits across --query-only runs.
    # That makes LR p50 look ~1.5s while EQ pays keyword+generate every time.
    # Pin BENCH001_LR_ENABLE_LLM_CACHE=0 for cold dual-SUT latency peers.
    _cache_raw = (os.environ.get("BENCH001_LR_ENABLE_LLM_CACHE") or "1").strip().lower()
    enable_llm_cache = _cache_raw not in {"0", "false", "off", "no"}
    print(
        f"LR enable_llm_cache={enable_llm_cache} "
        f"(BENCH001_LR_ENABLE_LLM_CACHE={_cache_raw!r})",
        flush=True,
    )
    rag = LightRAG(
        working_dir=str(working_dir),
        embedding_func=sut_embed,
        llm_model_func=sut_complete,
        llm_model_name=llm_model,
        enable_llm_cache=enable_llm_cache,
    )
    # LightRAG ≥ recent: pipeline_status requires initialize_storages()
    await rag.initialize_storages()
    try:
        from lightrag.kg.shared_storage import initialize_pipeline_status

        await initialize_pipeline_status()
    except Exception:  # noqa: BLE001
        pass
    marker = working_dir / ".bench001_ingested"
    if force_ingest or not marker.exists():
        await rag.ainsert(texts)
        marker.write_text("ok", encoding="utf-8")

    # LightRAG mix + nano-vectordb thrashes under high asyncio fan-out on one
    # process (CPU peg, no progress). Cap below EQ ThreadPool concurrency.
    import os as _os

    n_req = query_concurrency(concurrency)
    n_cap = int(_os.environ.get("BENCH001_LR_QUERY_CONCURRENCY", "2"))
    n = max(1, min(n_req, n_cap))
    q_timeout = float(_os.environ.get("BENCH001_LR_QUERY_TIMEOUT_S", "240"))
    print(
        f"LR querying {len(questions)} questions concurrency={n} "
        f"(cap={n_cap} timeout={q_timeout:.0f}s)",
        flush=True,
    )
    sem = asyncio.Semaphore(n)
    done = 0
    done_lock = asyncio.Lock()
    t_query = time.perf_counter()

    async def _one(q: dict[str, Any]) -> dict[str, Any]:
        nonlocal done
        async with sem:
            t0 = time.perf_counter()
            print(f"  LR query start {q.get('id')}", flush=True)
            try:
                # aquery() drops retrieval context (compat wrapper). Official
                # GraphRAG-Bench needs context for Faithfulness — use aquery_llm.
                from .judge_tune import system_prompt_for_style

                from .fair_pins import lr_query_param_overrides

                style_prompt = system_prompt_for_style()
                param_kw: dict[str, Any] = {
                    "mode": mode,
                    **lr_query_param_overrides(),
                }
                if style_prompt:
                    # Prefer user_prompt (appended instruction) over replacing
                    # LightRAG's system template — dual injection can stall mix.
                    param_kw["user_prompt"] = style_prompt
                result = await asyncio.wait_for(
                    rag.aquery_llm(
                        q["question"],
                        param=QueryParam(**param_kw),
                    ),
                    timeout=q_timeout,
                )
                latency_ms = int((time.perf_counter() - t0) * 1000)
                answer, context = _lr_answer_and_context(result)
            except Exception as exc:  # noqa: BLE001
                answer, context, latency_ms = (
                    "",
                    f"error: {exc}",
                    int((time.perf_counter() - t0) * 1000),
                )
            gold = q.get("answer") or ""
            pred = {
                "id": q["id"],
                "question": q["question"],
                "source": q.get("_subset") or str(q.get("source", "")).lower(),
                "context": [context] if isinstance(context, str) else list(context or []),
                "evidence": q.get("evidence") or [],
                "question_type": q["question_type"],
                "generated_answer": str(answer or "").strip(),
                "ground_truth": gold,
                "gold_answer": gold,
                "latency_ms": latency_ms,
            }
            async with done_lock:
                done += 1
                if done == 1 or done % 5 == 0 or done == len(questions):
                    from .progress import mark_phase, print_unit_progress

                    elapsed = max(time.perf_counter() - t_query, 1e-6)
                    qid = str(q.get("id") or "")
                    print_unit_progress(
                        "LR query",
                        done,
                        len(questions),
                        elapsed_s=elapsed,
                        extra=f"id={qid}" if qid else "",
                    )
                    stage_name = _os.environ.get("BENCH001_PROGRESS_STAGE") or "smoke-fast"
                    mark_phase(
                        stage_name,
                        "query_lr",
                        status="running",
                        detail=f"LR query {done}/{len(questions)}",
                        done=done,
                        total=len(questions),
                        phase_elapsed_s=elapsed,
                        quiet=True,
                    )
            return pred

    return list(await asyncio.gather(*[_one(q) for q in questions]))


def _lr_answer_and_context(result: Any) -> tuple[str, str]:
    """Normalize LightRAG aquery_llm / legacy tuple / plain-string responses."""
    if isinstance(result, tuple):
        answer = str(result[0] or "")
        context = str(result[1] or "") if len(result) > 1 else ""
        return answer.strip(), context.strip()
    if isinstance(result, str):
        return result.strip(), ""
    if not isinstance(result, dict):
        return str(result or "").strip(), ""

    llm = result.get("llm_response") or {}
    answer = ""
    if isinstance(llm, dict):
        answer = str(llm.get("content") or "")
    if not answer:
        answer = str(result.get("content") or result.get("response") or "")

    data = result.get("data") if isinstance(result.get("data"), dict) else result
    parts: list[str] = []
    chunks = (data or {}).get("chunks") or []
    if isinstance(chunks, list):
        for c in chunks:
            if isinstance(c, str) and c.strip():
                parts.append(c.strip())
            elif isinstance(c, dict):
                text = (
                    c.get("content")
                    or c.get("text")
                    or c.get("snippet")
                    or c.get("chunk")
                    or ""
                )
                if str(text).strip():
                    parts.append(str(text).strip())
    if not parts:
        refs = (data or {}).get("references") or result.get("references") or []
        if isinstance(refs, list):
            for r in refs:
                if isinstance(r, str) and r.strip():
                    parts.append(r.strip())
                elif isinstance(r, dict):
                    text = r.get("content") or r.get("text") or r.get("snippet") or ""
                    if str(text).strip():
                        parts.append(str(text).strip())
    context = "\n-----\n".join(parts)
    return answer.strip(), context


def run_lightrag(
    *,
    corpus_texts: list[str],
    questions: list[dict[str, Any]],
    stage: str,
    mode: str = "mix",
    force_ingest: bool = False,
    query_only: bool = False,
    concurrency: int | None = None,
) -> list[dict[str, Any]]:
    """Sync wrapper around async LightRAG path."""
    import asyncio

    if not available() and lightrag_repo() is None:
        raise RuntimeError(
            "LightRAG not available. Set BENCH001_LIGHTRAG_REPO or install lightrag."
        )
    wd = workspace_dir(stage)
    if query_only and not (wd / ".bench001_ingested").exists():
        raise RuntimeError(f"query-only but no LightRAG index at {wd}")
    return asyncio.run(
        _insert_and_query_async(
            texts=corpus_texts,
            questions=questions,
            stage=stage,
            mode=mode,
            working_dir=wd,
            force_ingest=force_ingest and not query_only,
            concurrency=concurrency,
        )
    )


def stub_predictions(questions: list[dict[str, Any]], *, label: str = "lr") -> list[dict[str, Any]]:
    """Dry-run / missing-SUT stub.

    Failure labels (``*_fail``) must **not** echo gold — that inflates Acc and
    can look like a win when the SUT never ran. Dry-run stubs keep a partial
    gold echo for offline plumbing checks only.
    """
    fail_closed = label.endswith("_fail") or label in {"skipped", "eq_fail", "lr_fail"}
    out = []
    for q in questions:
        gold = q.get("answer") or ""
        if fail_closed:
            answer = ""
            context: list[str] = [f"stub:{label}"]
        else:
            toks = gold.split()
            keep = max(3, len(toks) // 2)
            answer = " ".join(toks[:keep])
            context = [gold[:500]]
        out.append(
            {
                "id": q["id"],
                "question": q["question"],
                "source": q.get("_subset") or str(q.get("source", "")).lower(),
                "context": context,
                "evidence": q.get("evidence") or [],
                "question_type": q["question_type"],
                "generated_answer": answer,
                "ground_truth": gold,
                "gold_answer": gold,
                "latency_ms": 0,
                "stub": label,
            }
        )
    return out
