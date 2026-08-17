"""Acc-loop environment hygiene (fast + reliable SPEC-001 runs).

Agent / CI shells sometimes inject placeholder API keys (``LLM_API_KEY=FAKE…``)
that poison official GraphRAG-Bench judges via ``os.environ.setdefault``.
This module scrubs placeholders and, when needed, reloads a real Mistral key
from ``/tmp/edgequake-start.sh`` (Makefile Acc backend start script).

Publication Acc pins (forced, not setdefault):
  text LLM / vision / judge = mistral-small-latest
  embedding = mistral-embed (Mistral embedding API — not a chat model)
<<<<<<< HEAD
  chunk = 1200 / overlap 100, adaptive off, fusion rrf
=======
  chunk = 1200 / overlap 100, adaptive off
  Mix Acc law (SPEC-086) = E2-occ: round_robin · rerank off · bfs · retrieval
  rank · LR VECTOR budget · occurrence_sort · Fact L2 fact_replace
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
"""

from __future__ import annotations

import os
import re
from pathlib import Path
from typing import Any

START_SH_CANDIDATES = (
    Path("/tmp/edgequake-start.sh"),
    Path("/tmp/eq-bench-start-trap.sh"),
)

_KEY_EXPORT_RE = re.compile(
    r'^export\s+(MISTRAL_API_KEY|LLM_API_KEY|OPENAI_API_KEY)="([^"]*)"',
    re.MULTILINE,
)

# Canonical Acc / publication stack (chat ≠ embed).
ACC_LLM_MODEL = "mistral-small-latest"
ACC_VISION_MODEL = "mistral-small-latest"
ACC_JUDGE_MODEL = "mistral-small-latest"
ACC_EMBED_MODEL = "mistral-embed"
ACC_EMBED_DIM = "1024"
ACC_CHUNK_SIZE = "1200"
ACC_CHUNK_OVERLAP = "100"

# Force-overwrite map for publication Acc (shell bleed cannot win).
PUBLICATION_ENV: dict[str, str] = {
    "BENCH001_LLM_PROVIDER": "mistral",
    "BENCH001_LLM_MODEL": ACC_LLM_MODEL,
    "BENCH001_VISION_PROVIDER": "mistral",
    "BENCH001_VISION_MODEL": ACC_VISION_MODEL,
    "BENCH001_EMBEDDING_PROVIDER": "mistral",
    "BENCH001_EMBEDDING_MODEL": ACC_EMBED_MODEL,
    "BENCH001_EMBEDDING_DIM": ACC_EMBED_DIM,
    "BENCH001_JUDGE_PROVIDER": "mistral",
    "BENCH001_JUDGE_MODEL": ACC_JUDGE_MODEL,
    "BENCH001_JUDGE_EMBEDDING_MODEL": ACC_EMBED_MODEL,
    "BENCH001_ANSWER_STYLE": "gold",
    "BENCH001_PUBLISH_FAIRNESS": "1",
    "EDGEQUAKE_LLM_PROVIDER": "mistral",
    "EDGEQUAKE_LLM_MODEL": ACC_LLM_MODEL,
    "MISTRAL_MODEL": ACC_LLM_MODEL,
    "EDGEQUAKE_VISION_PROVIDER": "mistral",
    "EDGEQUAKE_VISION_MODEL": ACC_VISION_MODEL,
    "EDGEQUAKE_EMBEDDING_PROVIDER": "mistral",
    "EDGEQUAKE_EMBEDDING_MODEL": ACC_EMBED_MODEL,
    "MISTRAL_EMBEDDING_MODEL": ACC_EMBED_MODEL,
    "EDGEQUAKE_DEFAULT_LLM_PROVIDER": "mistral",
    "EDGEQUAKE_DEFAULT_LLM_MODEL": ACC_LLM_MODEL,
    "EDGEQUAKE_DEFAULT_EMBEDDING_PROVIDER": "mistral",
    "EDGEQUAKE_DEFAULT_EMBEDDING_MODEL": ACC_EMBED_MODEL,
    "EDGEQUAKE_ADAPTIVE_CHUNKING": "0",
    "EDGEQUAKE_CHUNK_SIZE": ACC_CHUNK_SIZE,
    "EDGEQUAKE_CHUNK_OVERLAP": ACC_CHUNK_OVERLAP,
    "EDGEQUAKE_MIX_ARM_GATE": "false",
<<<<<<< HEAD
    "EDGEQUAKE_MIX_FUSION": "rrf",
    "EDGEQUAKE_HYBRID_FUSION": "rrf",
=======
    # SPEC-086 Acc law = E2-occ (prior P0 rrf is labeled peer only).
    "EDGEQUAKE_MIX_FUSION": "round_robin",
    "EDGEQUAKE_HYBRID_FUSION": "round_robin",
    "BENCH001_ALLOW_ROUND_ROBIN": "1",
    "BENCH001_EQ_ENABLE_RERANK": "0",
    "EDGEQUAKE_ENTITY_RANK": "retrieval",
    "EDGEQUAKE_GRAPH_WALK": "bfs",
    "EDGEQUAKE_KG_CHUNK_PICK": "vector",
    "EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET": "1",
    "EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT": "1",
    "EDGEQUAKE_BM25_RETRIEVAL": "1",
    "EDGEQUAKE_L2_BM25_UNION": "1",
    "EDGEQUAKE_L2_BM25_MODE": "fact_replace",
    "EDGEQUAKE_L2_BM25_MIX_TOP_K": "30",
    # SPEC-103 LAW-C7: Acc cold peer — warm LLM cache is never a Beat claim.
    "EDGEQUAKE_LLM_CACHE": "0",
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    # SPEC-001 Phase 1 relevancy prune — OFF for Acc headline (gate not met).
    # Cosine ablation: set PRUNE=1 and SCORE=cosine on the Acc server process.
    "EDGEQUAKE_MIX_RELEVANCY_PRUNE": "0",
    "EDGEQUAKE_MIX_RELEVANCY_SCORE": "rrf",
    "EDGEQUAKE_MIX_RELEVANCY_KEEP": "12",
    "EDGEQUAKE_MIX_RELEVANCY_MIN_KEEP": "8",
    "EDGEQUAKE_MIX_RELEVANCY_SCORE_FLOOR": "0.25",
    "EDGEQUAKE_MIX_GRAPH_SOFT_PRUNE": "0",
<<<<<<< HEAD
    # CE / PathRAG Acc headline: BM25 + PATH_PRUNE off (T011703Z BM25+path=0.4 Acc loss).
=======
    # CE / PathRAG Acc headline: PATH_PRUNE off (T011703Z BM25+path=0.4 Acc loss).
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    # Soft path only with labeled CE+protect (F2a / path_pack_v1).
    "EDGEQUAKE_RERANKER": "bm25",
    "EDGEQUAKE_PATH_PRUNE": "0",
    "EDGEQUAKE_PATH_PRUNE_FRACTION": "0",
    "EDGEQUAKE_PATH_PRUNE_ORPHAN_ENTITIES": "0",
    # 022 P5: arm pool ≥3× typical Acc query concurrency.
    "EDGEQUAKE_QUERY_ARM_CONCURRENCY": "16",
    # 022 P3: disable hub-noise popular-node fallback on Acc Mix.
    "EDGEQUAKE_POPULAR_NODE_FALLBACK": "0",
    # 022 P1: graph-walk compress off for Acc headline (labeled gw_compress_v1).
    "EDGEQUAKE_GRAPH_WALK_COMPRESS": "0",
}


def is_placeholder_api_key(value: str | None) -> bool:
    """True when *value* is empty or an agent-injected FAKE* placeholder."""
    if value is None:
        return True
    v = value.strip()
    if not v:
        return True
    return v.upper().startswith("FAKE")


def scrub_placeholder_api_keys(
    *,
    env: dict[str, str] | None = None,
) -> list[str]:
    """Remove FAKE*/empty API key env vars. Returns names that were cleared."""
    target = env if env is not None else os.environ
    cleared: list[str] = []
    for name in ("LLM_API_KEY", "MISTRAL_API_KEY", "OPENAI_API_KEY", "ANTHROPIC_API_KEY"):
        if name not in target:
            continue
        if is_placeholder_api_key(target.get(name)):
            del target[name]
            cleared.append(name)
    return cleared


def load_mistral_key_from_start_sh(
    paths: tuple[Path, ...] | None = None,
) -> str | None:
    """Parse a non-placeholder Mistral key from Acc start scripts."""
    candidates = paths if paths is not None else START_SH_CANDIDATES
    for path in candidates:
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except OSError:
            continue
        found: dict[str, str] = {}
        for m in _KEY_EXPORT_RE.finditer(text):
            found[m.group(1)] = m.group(2)
        for name in ("MISTRAL_API_KEY", "LLM_API_KEY", "OPENAI_API_KEY"):
            val = found.get(name)
            if val and not is_placeholder_api_key(val):
                return val
    return None


def ensure_acc_api_keys(*, verbose: bool = False) -> bool:
    """Scrub placeholders and ensure a usable Mistral key is exported."""
    cleared = scrub_placeholder_api_keys()
    if verbose and cleared:
        print(f"acc_env: cleared placeholder keys: {', '.join(cleared)}", flush=True)

    from .profiles import mistral_api_key

    if mistral_api_key():
        key = mistral_api_key()
        assert key is not None
        os.environ["MISTRAL_API_KEY"] = key
        os.environ["LLM_API_KEY"] = key
        os.environ["OPENAI_API_KEY"] = key
        return True

    loaded = load_mistral_key_from_start_sh()
    if not loaded:
        if verbose:
            print(
                "acc_env: no usable Mistral key (env + /tmp/edgequake-start.sh)",
                flush=True,
            )
        return False

    os.environ["MISTRAL_API_KEY"] = loaded
    os.environ["LLM_API_KEY"] = loaded
    os.environ["OPENAI_API_KEY"] = loaded
    if verbose:
        print("acc_env: loaded Mistral key from Acc start script", flush=True)
    return True


def apply_acc_speed_defaults() -> None:
    """Pin env defaults that make Acc smoke-fast cheap and stable."""
    os.environ.setdefault("BENCH001_ANSWER_STYLE", "gold")
    os.environ.setdefault("BENCH001_PUBLISH_FAIRNESS", "1")
    os.environ.setdefault("BENCH001_INGEST_MAX_CHARS", "100000")
    os.environ.setdefault("BENCH001_INGEST_TIMEOUT_S", "1800")
    os.environ.setdefault("BENCH001_ACC_LLM_MODEL", ACC_LLM_MODEL)
    os.environ.setdefault("BENCH001_ACC_JUDGE_MODEL", ACC_JUDGE_MODEL)
    if (os.environ.get("EDGEQUAKE_MIX_FUSION") or "").strip().lower() == "round_robin":
        if os.environ.get("BENCH001_ALLOW_ROUND_ROBIN", "").strip() not in {"1", "true", "yes"}:
            os.environ["EDGEQUAKE_MIX_FUSION"] = "rrf"
            os.environ["EDGEQUAKE_HYBRID_FUSION"] = "rrf"


def apply_acc_publication_pins(
    *,
    full_corpus: bool = True,
    clear_capped_workspace: bool = True,
    verbose: bool = True,
) -> None:
    """Force Acc publication pins (overwrite shell bleed).

    Embeddings stay on ``mistral-embed`` (dedicated embed API). Chat/vision/judge
    use ``mistral-small-latest``. Full corpus forces ``BENCH001_INGEST_MAX_CHARS=0``
    and arms fail-closed publication checks.
    """
    # Relevancy-prune ablation keys: preserve non-empty shell overrides so
    # labeled Acc runs (e.g. SCORE=cosine) stay visible in scorecard.pins.
    _preserve_if_set = {
        "EDGEQUAKE_MIX_RELEVANCY_PRUNE",
        "EDGEQUAKE_MIX_RELEVANCY_SCORE",
        "EDGEQUAKE_MIX_RELEVANCY_KEEP",
        "EDGEQUAKE_MIX_RELEVANCY_MIN_KEEP",
        "EDGEQUAKE_MIX_RELEVANCY_SCORE_FLOOR",
        "EDGEQUAKE_MIX_GRAPH_SOFT_PRUNE",
        "EDGEQUAKE_MIX_RELEVANCY_EMBED_CHARS",
        "EDGEQUAKE_RERANKER",
        "EDGEQUAKE_RERANKER_PROVIDER",
        "EDGEQUAKE_RERANKER_MODEL",
        "EDGEQUAKE_RERANKER_BASE_URL",
        "EDGEQUAKE_PATH_PRUNE_FRACTION",
        "EDGEQUAKE_PATH_PRUNE_ORPHAN_ENTITIES",
        "EDGEQUAKE_PATH_PRUNE_ENTITY_MIN_KEEP",
        "EDGEQUAKE_PATH_PRUNE",
        "EDGEQUAKE_RERANK_PROTECT_FIRST",
        "EDGEQUAKE_ENTITY_RANK",
        "EDGEQUAKE_RELATED_CHUNK_NUMBER",
        "EDGEQUAKE_MIX_LOCAL_WEIGHT",
        "EDGEQUAKE_MIX_GLOBAL_WEIGHT",
        "EDGEQUAKE_MIX_NAIVE_WEIGHT",
        "EDGEQUAKE_CONTEXT_FORMAT",
        "EDGEQUAKE_PASSAGE_PACK",
        "EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO",
        "EDGEQUAKE_QUERY_ARM_CONCURRENCY",
        "EDGEQUAKE_PPR_DAMPING",
        "EDGEQUAKE_PPR_MAX_ITERS",
        "EDGEQUAKE_GRAPH_WALK",
        "EDGEQUAKE_GRAPH_WALK_COMPRESS",
        "EDGEQUAKE_POPULAR_NODE_FALLBACK",
        "EDGEQUAKE_CONTENT_HEADINGS",
        "EDGEQUAKE_KEYWORD_LEXICAL_BOOST",
        "EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT",
        "EDGEQUAKE_KG_CHUNK_PICK",
        "EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET",
        "EDGEQUAKE_BM25_RETRIEVAL",
        "EDGEQUAKE_KG_CHUNK_PICK_TIMING",
        "BENCH001_EQ_ENABLE_RERANK",
        "EDGEQUAKE_MIN_RERANK_SCORE",
        "EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO",
        "EDGEQUAKE_L2_SOURCES_UNION",
        "EDGEQUAKE_L2_SOURCES_MIX_TOP_K",
        "EDGEQUAKE_FACT_RERANKER",
        "EDGEQUAKE_FACT_PROTECT_BM25",
        "EDGEQUAKE_COVERAGE_PROTECT_FIRST",
        "EDGEQUAKE_TOPIC_ENTITY_ADMIT",
        "EDGEQUAKE_TOPIC_CE_PROTECT",
        "EDGEQUAKE_TOPIC_TRUNC_PROTECT",
        "EDGEQUAKE_TOPIC_TRUNC_PROTECT_MAX",
        "EDGEQUAKE_TOPIC_MATERIALIZE",
        "EDGEQUAKE_TOPIC_MATERIALIZE_MAX",
        "EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT",
        "EDGEQUAKE_TOPIC_MATERIALIZE_TYPES",
        "EDGEQUAKE_INTENT_RERANK",
        "EDGEQUAKE_L2_BM25_UNION",
        "EDGEQUAKE_L2_BM25_MIX_TOP_K",
        "EDGEQUAKE_L2_BM25_MODE",
        "EDGEQUAKE_MIX_FUSION",
        "EDGEQUAKE_RR_ORDER",
        "EDGEQUAKE_MIX_INTENT_WEIGHTS",
        "EDGEQUAKE_RELATION_SELECT",
        "EDGEQUAKE_INTENT_FACTUAL_BIAS",
        "EDGEQUAKE_ANSWER_PROMPT",
        "EDGEQUAKE_ANSWER_SPECIFIC_TYPES",
        "BENCH001_EQ_RERANK_TOP_K",
    }
    for key, value in PUBLICATION_ENV.items():
        if key in _preserve_if_set and (os.environ.get(key) or "").strip():
            continue
        os.environ[key] = value
    os.environ["BENCH001_ACC_LLM_MODEL"] = ACC_LLM_MODEL
    os.environ["BENCH001_ACC_JUDGE_MODEL"] = ACC_JUDGE_MODEL
    if full_corpus:
        os.environ["BENCH001_PUBLICATION"] = "1"
        os.environ["BENCH001_INGEST_MAX_CHARS"] = "0"
        os.environ.setdefault("BENCH001_INGEST_TIMEOUT_S", "7200")
        if clear_capped_workspace:
            ws = (os.environ.get("BENCH001_EQ_WORKSPACE_ID") or "").strip()
            if ws and ("c100000" in ws or "c10000" in ws):
                os.environ.pop("BENCH001_EQ_WORKSPACE_ID", None)
                if verbose:
                    print(
                        "acc_env: cleared capped BENCH001_EQ_WORKSPACE_ID for full corpus",
                        flush=True,
                    )
    if verbose:
        print(
            "acc_env: Acc pins "
            f"llm={ACC_LLM_MODEL} vision={ACC_VISION_MODEL} "
            f"embed={ACC_EMBED_MODEL} judge={ACC_JUDGE_MODEL} "
            f"chunk={ACC_CHUNK_SIZE}/{ACC_CHUNK_OVERLAP} "
            f"ingest_max_chars={os.environ.get('BENCH001_INGEST_MAX_CHARS')} "
            f"publication={os.environ.get('BENCH001_PUBLICATION', '0')}",
            flush=True,
        )


def backend_pin_mismatches(health: dict[str, Any]) -> list[str]:
    """Return human-readable mismatches vs Acc publication pins from /health."""
    providers = health.get("providers") or {}
    llm = providers.get("llm") or {}
    emb = providers.get("embedding") or {}
    vision = providers.get("vision") or {}
    mismatches: list[str] = []

    llm_name = str(llm.get("name") or health.get("llm_provider_name") or "").lower()
    llm_model = str(llm.get("model") or "").strip()
    emb_name = str(emb.get("name") or "").lower()
    emb_model = str(emb.get("model") or "").strip()
    vis_name = str(vision.get("name") or "").lower()
    vis_model = str(vision.get("model") or "").strip()

    if llm_name and llm_name != "mistral":
        mismatches.append(f"llm_provider={llm_name} (want mistral)")
    if llm_model and llm_model != ACC_LLM_MODEL:
        mismatches.append(f"llm_model={llm_model} (want {ACC_LLM_MODEL})")
    if emb_name and emb_name != "mistral":
        mismatches.append(f"embed_provider={emb_name} (want mistral)")
    if emb_model and emb_model != ACC_EMBED_MODEL:
        mismatches.append(f"embed_model={emb_model} (want {ACC_EMBED_MODEL})")
    if vis_name and vis_name not in {"", "mistral"}:
        mismatches.append(f"vision_provider={vis_name} (want mistral)")
    if vis_model and vis_model != ACC_VISION_MODEL:
        mismatches.append(f"vision_model={vis_model} (want {ACC_VISION_MODEL})")

    qe = ((health.get("operational") or {}).get("query_engine") or {})
    mix = str(qe.get("mix_fusion") or "").lower()
<<<<<<< HEAD
    allow_rr = (os.environ.get("BENCH001_ALLOW_ROUND_ROBIN") or "").strip().lower() in {
=======
    # SPEC-086 Acc law default = round_robin. Legacy P0 rrf peers set
    # BENCH001_ALLOW_RRF_LEGACY=1 (or unset ALLOW_ROUND_ROBIN and force rrf).
    allow_rr = (os.environ.get("BENCH001_ALLOW_ROUND_ROBIN") or "1").strip().lower() in {
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        "1",
        "true",
        "yes",
        "on",
    }
<<<<<<< HEAD
    if mix and mix != "rrf":
        if allow_rr and mix in {"round_robin", "rr", "lightrag"}:
            pass  # 022 P2a labeled LightRAG-parity fusion ablation
        else:
            mismatches.append(f"mix_fusion={mix} (want rrf)")
=======
    allow_rrf_legacy = (
        os.environ.get("BENCH001_ALLOW_RRF_LEGACY") or ""
    ).strip().lower() in {"1", "true", "yes", "on"}
    if mix:
        if mix in {"round_robin", "rr", "lightrag"} and allow_rr:
            pass  # 086 Acc law / labeled RR peers
        elif mix == "rrf" and (allow_rrf_legacy or not allow_rr):
            pass  # prior P0 Acc peer
        else:
            want = "round_robin" if allow_rr else "rrf"
            mismatches.append(f"mix_fusion={mix} (want {want})")
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    return mismatches


def assert_publication_ingest(ingest_meta: dict[str, Any]) -> None:
    """Fail closed when publication Acc would still use a capped corpus."""
    if os.environ.get("BENCH001_PUBLICATION", "").strip() not in {"1", "true", "yes"}:
        return
    if ingest_meta.get("ingest_capped"):
        raise RuntimeError(
            "publication Acc forbids capped ingest "
            f"(ingest_max_chars={ingest_meta.get('ingest_max_chars')}; "
            "unset shell BENCH001_INGEST_MAX_CHARS and use "
            "BENCH001_INGEST_MAX_CHARS=0 / make bench001-full)"
        )
