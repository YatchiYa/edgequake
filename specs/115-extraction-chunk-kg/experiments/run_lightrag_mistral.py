#!/usr/bin/env python3
"""SPEC-115 Arm C — live LightRAG insert with Mistral Small + mistral-embed.

Uses gold MD twin of papers/light_rag_2410.05779v3.pdf for fair extract geometry.
Requires: MISTRAL_API_KEY, LightRAG checkout, network.
"""

from __future__ import annotations

import asyncio
import json
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
LR_ROOT = Path("/Users/raphaelmansuy/Github/03-working/LightRAG")
OUT_DIR = Path(__file__).resolve().parents[1] / "measurements"
WORK = OUT_DIR / "lightrag_workdir"
GOLD = REPO / "zz_test_docs/academic_papers/lighrag_2410.05779v3.pymupdf.gold.md"

LLM_MODEL = os.environ.get("SPEC115_LLM_MODEL", "mistral-small-latest")
EMB_MODEL = os.environ.get("SPEC115_EMBED_MODEL", "mistral-embed")
EMB_DIM = int(os.environ.get("SPEC115_EMBED_DIM", "1024"))
BASE_URL = os.environ.get("SPEC115_BASE_URL", "https://api.mistral.ai/v1")
CHUNK_SIZE = int(os.environ.get("CHUNK_SIZE", "1200"))
CHUNK_OVERLAP = int(os.environ.get("CHUNK_OVERLAP_SIZE", "100"))

sys.path.insert(0, str(LR_ROOT))


async def run() -> dict:
    import numpy as np
    import httpx
    from lightrag import LightRAG
    from lightrag.llm.openai import openai_complete_if_cache
    from lightrag.utils import TiktokenTokenizer, wrap_embedding_func_with_attrs
    from lightrag.chunker import chunking_by_token_size
    from lightrag.kg.shared_storage import initialize_pipeline_status

    key = os.environ.get("MISTRAL_API_KEY") or os.environ.get("LLM_BINDING_API_KEY")
    if not key:
        raise SystemExit("MISTRAL_API_KEY required")

    text = GOLD.read_text(encoding="utf-8")
    tok = TiktokenTokenizer()
    chunks = chunking_by_token_size(
        tok,
        text,
        chunk_token_size=CHUNK_SIZE,
        chunk_overlap_token_size=CHUNK_OVERLAP,
    )
    n_geom = len(chunks)

    if WORK.exists():
        # Fresh workspace for reproducible live run
        import shutil

        shutil.rmtree(WORK)
    WORK.mkdir(parents=True, exist_ok=True)

    async def sut_complete(
        prompt, system_prompt=None, history_messages=None, **kwargs
    ):
        if history_messages is None:
            history_messages = []
        return await openai_complete_if_cache(
            LLM_MODEL,
            prompt,
            system_prompt=system_prompt,
            history_messages=history_messages,
            base_url=BASE_URL,
            api_key=key,
            **kwargs,
        )

    @wrap_embedding_func_with_attrs(
        embedding_dim=EMB_DIM,
        max_token_size=8192,
        model_name=EMB_MODEL,
    )
    async def sut_embed(texts: list[str], **kwargs):
        del kwargs
        payload = {"model": EMB_MODEL, "input": list(texts)}
        async with httpx.AsyncClient(timeout=120.0) as client:
            r = await client.post(
                f"{BASE_URL.rstrip('/')}/embeddings",
                headers={
                    "Authorization": f"Bearer {key}",
                    "Content-Type": "application/json",
                },
                json=payload,
            )
            r.raise_for_status()
            data = sorted(r.json()["data"], key=lambda d: d["index"])
            vecs = [np.array(d["embedding"], dtype=np.float32) for d in data]
            return np.stack(vecs, axis=0)

    os.environ.setdefault("CHUNK_SIZE", str(CHUNK_SIZE))
    os.environ.setdefault("CHUNK_OVERLAP_SIZE", str(CHUNK_OVERLAP))

    rag = LightRAG(
        working_dir=str(WORK),
        embedding_func=sut_embed,
        llm_model_func=sut_complete,
        llm_model_name=LLM_MODEL,
        chunk_token_size=CHUNK_SIZE,
        chunk_overlap_token_size=CHUNK_OVERLAP,
        enable_llm_cache=False,
    )
    await rag.initialize_storages()
    await initialize_pipeline_status()

    t0 = time.perf_counter()
    await rag.ainsert([text])
    elapsed = time.perf_counter() - t0

    nodes = await rag.chunk_entity_relation_graph.get_all_nodes()
    # edges: networkx / json impl
    unique_nodes = len(nodes) if nodes is not None else None
    unique_edges = None
    try:
        graph = rag.chunk_entity_relation_graph
        if hasattr(graph, "get_all_edges"):
            edges = await graph.get_all_edges()
            unique_edges = len(edges)
        elif hasattr(graph, "_graph"):
            unique_edges = graph._graph.number_of_edges()
        elif hasattr(graph, "graph"):
            g = graph.graph
            unique_edges = g.number_of_edges() if hasattr(g, "number_of_edges") else None
    except Exception as exc:  # noqa: BLE001
        unique_edges = f"error:{exc}"

    # Chunk count from text_chunks storage if available
    n_stored = None
    try:
        # LightRAG stores chunks in text_chunks KV
        all_keys = await rag.text_chunks.all_keys()
        n_stored = len(all_keys)
    except Exception:  # noqa: BLE001
        n_stored = n_geom

    result = {
        "utc": datetime.now(timezone.utc).isoformat(),
        "arm": "C",
        "sut": "lightrag",
        "mode": "live-mistral",
        "sample_id": "S1-md",
        "sample": str(GOLD.relative_to(REPO)),
        "chars": len(text),
        "doc_tokens": len(tok.encode(text)),
        "chunk_size_pin": CHUNK_SIZE,
        "overlap_pin": CHUNK_OVERLAP,
        "chunk_count_geometry": n_geom,
        "chunk_count_stored": n_stored,
        "unique_nodes": unique_nodes,
        "unique_edges": unique_edges,
        "llm_model": LLM_MODEL,
        "embed_model": EMB_MODEL,
        "embed_dim": EMB_DIM,
        "elapsed_s": round(elapsed, 2),
        "workdir": str(WORK),
    }
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "lightrag_live.json").write_text(
        json.dumps(result, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(result, indent=2))
    await rag.finalize_storages()
    return result


if __name__ == "__main__":
    asyncio.run(run())
