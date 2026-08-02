"""Publishable dual-SUT fairness pins (SPEC-001 / July 2026 practice).

First principles for a publishable EQ↔LR comparison:
  1. Shared corpus, questions, generator family, embed family, judge.
  2. Matched retrieval *budget* (top-k / max_results) — not merely mode names.
  3. Layered metrics: generation (L0/L1) + retrieval (L2 Evidence Recall /
     Context Relevancy) so Acc cannot be misread as retrieval quality.
  4. Explicit answer-style control applied equally (concise) — Acc is F1-heavy.
  5. No phantom reranker: disable LR enable_rerank when no model is configured.
  6. Headline modes stay EQ mix ↔ LR mix (name-fair); paper hybrid/topk is an
     ablation (P0_paper), not mixed into the headline claim.

Profile id bumps to ``P0_mistral_mix_v2`` when these pins are active.
"""

from __future__ import annotations

import os
from typing import Any

# Paper H.2 LightRAG retrieval_topk=30 — use as the matched budget for both SUTs.
PUBLISH_RETRIEVE_TOPK = 30
PUBLISH_LR_TOP_K = 30
PUBLISH_LR_CHUNK_TOP_K = 30
PUBLISH_EQ_MAX_RESULTS = 30
PUBLISH_PROFILE_ID = "P0_mistral_mix_v2"

# Empty-context / empty-answer ceilings for publishable smoke (stricter than plumbing).
PUBLISH_EMPTY_ANSWER_MAX = 0.05
PUBLISH_EMPTY_CONTEXT_MAX = 0.05

# LR-like Mix: always run local+global+naive (EQ production default gates Factual→naive-only).
# Must be set on the EdgeQuake *server* process (`EDGEQUAKE_MIX_ARM_GATE=false`).
LRLIKE_ARMS_PROFILE_SUFFIX = "lrlike_arms_v2"

# LightRAG-matched ingest chunk pin (fair Acc). Server must set
# EDGEQUAKE_ADAPTIVE_CHUNKING=0 + EDGEQUAKE_CHUNK_SIZE=1200 for force-ingest.
FAIR_CHUNK_TOKEN_SIZE = 1200
FAIR_CHUNK_OVERLAP_TOKEN_SIZE = 100


def publish_fairness_enabled() -> bool:
    """Default on; set BENCH001_PUBLISH_FAIRNESS=0 to revert legacy topk=5 pins."""
    raw = (os.environ.get("BENCH001_PUBLISH_FAIRNESS") or "1").strip().lower()
    return raw not in {"0", "false", "no", "off"}


def _path_prune_fraction_pin() -> float:
    """Effective path-prune fraction for scorecard pins (022 P0: Acc default 0).

    ``EDGEQUAKE_PATH_PRUNE=0|false|off`` forces 0 even if FRACTION is set.
    """
    prune = (os.environ.get("EDGEQUAKE_PATH_PRUNE") or "").strip().lower()
    if prune in {"0", "false", "off", "no"}:
        return 0.0
    raw = (os.environ.get("EDGEQUAKE_PATH_PRUNE_FRACTION") or "0").strip()
    try:
        return max(0.0, min(0.9, float(raw)))
    except ValueError:
        return 0.0


def mix_arm_gate_enabled() -> bool:
    """Mirror EQ ``parse_mix_arm_gate``: default on; false/0/force_all → off."""
    raw = (os.environ.get("EDGEQUAKE_MIX_ARM_GATE") or "").strip().lower()
    if not raw:
        # Harness default for publishable dual-SUT: LR-like always-on 3 arms.
        if publish_fairness_enabled():
            return False
        return True
    return raw not in {"0", "false", "off", "no", "force_all", "all"}


def retrieve_topk() -> int:
    if publish_fairness_enabled():
        return int(os.environ.get("BENCH001_RETRIEVE_TOPK", PUBLISH_RETRIEVE_TOPK))
    return int(os.environ.get("BENCH001_RETRIEVE_TOPK", "5"))


def adaptive_chunking_enabled() -> bool:
    """Mirror EQ server: default on; Acc sets EDGEQUAKE_ADAPTIVE_CHUNKING=0."""
    raw = (os.environ.get("EDGEQUAKE_ADAPTIVE_CHUNKING") or "").strip().lower()
    if not raw:
        # Under publish fairness, Acc expects fixed 1200 unless explicitly adaptive.
        if publish_fairness_enabled():
            return False
        return True
    return raw not in {"0", "false", "off", "no"}


def chunk_token_size() -> int:
    return int(
        os.environ.get("EDGEQUAKE_CHUNK_SIZE")
        or os.environ.get("BENCH001_CHUNK_SIZE")
        or FAIR_CHUNK_TOKEN_SIZE
    )


def chunk_overlap_token_size() -> int:
    return int(
        os.environ.get("EDGEQUAKE_CHUNK_OVERLAP")
        or os.environ.get("BENCH001_CHUNK_OVERLAP")
        or FAIR_CHUNK_OVERLAP_TOKEN_SIZE
    )


def lr_query_param_overrides() -> dict[str, Any]:
    """Explicit LightRAG QueryParam knobs for fair dual-SUT comparison."""
    k = retrieve_topk()
    return {
        "top_k": int(os.environ.get("BENCH001_LR_TOP_K", k)),
        "chunk_top_k": int(os.environ.get("BENCH001_LR_CHUNK_TOP_K", k)),
        # No rerank model is configured in the harness — leave disabled so LR
        # does not pretend to rerank (paper RAG used bge-reranker; we don't).
        "enable_rerank": False,
        "include_references": True,
    }


def eq_enable_rerank() -> bool:
    """Whether EQ query requests post-fuse rerank.

    SPEC-086 Acc law default OFF (E2-occ). Set BENCH001_EQ_ENABLE_RERANK=1 for
    labeled CE / prior-P0 peers.
    """
    raw = (os.environ.get("BENCH001_EQ_ENABLE_RERANK") or "0").strip().lower()
    return raw not in {"0", "false", "off", "no"}


def eq_query_overrides() -> dict[str, Any]:
    """EdgeQuake query payload knobs matching the LR retrieval budget."""
    k = retrieve_topk()
    return {
        "max_results": int(os.environ.get("BENCH001_EQ_MAX_RESULTS", k)),
        "rerank_top_k": int(os.environ.get("BENCH001_EQ_RERANK_TOP_K", k)),
        "enable_rerank": eq_enable_rerank(),
        "include_references": True,
        "content_granularity": "agent",
    }


def publish_pin_fields() -> dict[str, Any]:
    from .ingest_cap import ingest_max_chars

    gate_on = mix_arm_gate_enabled()
    adaptive = adaptive_chunking_enabled()
    return {
        "publish_fairness": publish_fairness_enabled(),
        "retrieve_topk": retrieve_topk(),
        "lr_top_k": lr_query_param_overrides()["top_k"],
        "lr_chunk_top_k": lr_query_param_overrides()["chunk_top_k"],
        "lr_enable_rerank": False,
        # 063: LightRAG enable_llm_cache (default True = warm Acc latency unfair).
        "lr_enable_llm_cache": (
            os.environ.get("BENCH001_LR_ENABLE_LLM_CACHE") or "1"
        )
        .strip()
        .lower()
        not in {"0", "false", "off", "no"},
        "eq_max_results": eq_query_overrides()["max_results"],
        "eq_rerank_top_k": eq_query_overrides()["rerank_top_k"],
        "eq_enable_rerank": eq_query_overrides()["enable_rerank"],
        "graph_walk": (os.environ.get("EDGEQUAKE_GRAPH_WALK") or "ppr").strip().lower()
        or "ppr",
        "kg_chunk_pick": (
            os.environ.get("EDGEQUAKE_KG_CHUNK_PICK")
            or os.environ.get("KG_CHUNK_PICK_METHOD")
            or "vector"
        )
        .strip()
        .lower()
        or "vector",
        "l2_retrieval_required": True,
        "mix_arm_gate": gate_on,
        "eq_mix_arm_gate_env": os.environ.get("EDGEQUAKE_MIX_ARM_GATE", ""),
        "mix_fusion": (os.environ.get("EDGEQUAKE_MIX_FUSION") or "rrf").strip().lower()
        or "rrf",
        "rr_order": (
            (os.environ.get("EDGEQUAKE_RR_ORDER") or "local_first").strip().lower()
            or "local_first"
        ),
        "related_chunk_number": int(
            os.environ.get("EDGEQUAKE_RELATED_CHUNK_NUMBER")
            or os.environ.get("RELATED_CHUNK_NUMBER")
            or "5"
        ),
        "kg_chunk_occurrence_sort": (
            os.environ.get("EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        # Default on (empty/unset → true). Dense-only Mix arms pin 0/false/off (077 E1).
        "bm25_retrieval": (
            (os.environ.get("EDGEQUAKE_BM25_RETRIEVAL") or "1").strip().lower()
            not in {"0", "false", "off", "no"}
        ),
        # Default per_arm. LR-like truncate→VECTOR → post_truncate (078 R3).
        "kg_chunk_pick_timing": (
            (os.environ.get("EDGEQUAKE_KG_CHUNK_PICK_TIMING") or "per_arm")
            .strip()
            .lower()
            or "per_arm"
        ),
        "kg_chunk_pick_lr_budget": (
            os.environ.get("EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "mix_relevancy_prune": (
            os.environ.get("EDGEQUAKE_MIX_RELEVANCY_PRUNE") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "mix_relevancy_keep": int(
            os.environ.get("EDGEQUAKE_MIX_RELEVANCY_KEEP") or "12"
        ),
        "mix_relevancy_score": (
            os.environ.get("EDGEQUAKE_MIX_RELEVANCY_SCORE") or "rrf"
        )
        .strip()
        .lower()
        or "rrf",
        "mix_graph_soft_prune": (
            os.environ.get("EDGEQUAKE_MIX_GRAPH_SOFT_PRUNE") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "eq_reranker": (
            os.environ.get("EDGEQUAKE_RERANKER") or "bm25"
        )
        .strip()
        .lower()
        or "bm25",
        "eq_reranker_provider": (
            os.environ.get("EDGEQUAKE_RERANKER_PROVIDER") or ""
        )
        .strip()
        .lower(),
        "path_prune_fraction": _path_prune_fraction_pin(),
        "path_prune_orphan_entities": (
            os.environ.get("EDGEQUAKE_PATH_PRUNE_ORPHAN_ENTITIES") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "rerank_protect_first": int(
            os.environ.get("EDGEQUAKE_RERANK_PROTECT_FIRST") or "0"
        ),
        "min_rerank_score": float(
            os.environ.get("EDGEQUAKE_MIN_RERANK_SCORE") or "0.1"
        ),
        "entity_rank": (
            os.environ.get("EDGEQUAKE_ENTITY_RANK") or "degree"
        )
        .strip()
        .lower()
        or "degree",
        # 051: LightRAG incident-edge (rank, weight) select — default off.
        "relation_select": (
            os.environ.get("EDGEQUAKE_RELATION_SELECT") or "default"
        )
        .strip()
        .lower()
        or "default",
        "mix_local_weight": float(
            os.environ.get("EDGEQUAKE_MIX_LOCAL_WEIGHT") or "1"
        ),
        "mix_global_weight": float(
            os.environ.get("EDGEQUAKE_MIX_GLOBAL_WEIGHT") or "1"
        ),
        "mix_naive_weight": float(
            os.environ.get("EDGEQUAKE_MIX_NAIVE_WEIGHT") or "1"
        ),
        "context_format": (
            os.environ.get("EDGEQUAKE_CONTEXT_FORMAT") or "flat"
        )
        .strip()
        .lower()
        or "flat",
        "passage_pack": (
            os.environ.get("EDGEQUAKE_PASSAGE_PACK") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "graph_walk_compress": (
            os.environ.get("EDGEQUAKE_GRAPH_WALK_COMPRESS") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "popular_node_fallback": (
            os.environ.get("EDGEQUAKE_POPULAR_NODE_FALLBACK") or "1"
        )
        .strip()
        .lower()
        not in {"0", "false", "off", "no"},
        "keyword_lexical_boost": (
            os.environ.get("EDGEQUAKE_KEYWORD_LEXICAL_BOOST") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "content_headings": (
            os.environ.get("EDGEQUAKE_CONTENT_HEADINGS") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "l2_sources_union": (
            os.environ.get("EDGEQUAKE_L2_SOURCES_UNION") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "l2_sources_mix_top_k": int(
            os.environ.get("EDGEQUAKE_L2_SOURCES_MIX_TOP_K") or "30"
        ),
        "fact_reranker": (
            os.environ.get("EDGEQUAKE_FACT_RERANKER") or ""
        )
        .strip()
        .lower()
        or None,
        "fact_ce_skip": (
            os.environ.get("EDGEQUAKE_FACT_CE_SKIP") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "keyword_mode": (
            os.environ.get("EDGEQUAKE_KEYWORD_MODE") or "llm"
        )
        .strip()
        .lower()
        or "llm",
        "keyword_llm_provider": (
            (os.environ.get("EDGEQUAKE_KEYWORD_LLM_PROVIDER") or "").strip() or None
        ),
        "keyword_llm_model": (
            (os.environ.get("EDGEQUAKE_KEYWORD_LLM_MODEL") or "").strip() or None
        ),
        "extract_llm_provider": (
            (os.environ.get("EDGEQUAKE_EXTRACT_LLM_PROVIDER") or "").strip() or None
        ),
        "extract_llm_model": (
            (os.environ.get("EDGEQUAKE_EXTRACT_LLM_MODEL") or "").strip() or None
        ),
        "fact_protect_bm25": (
            os.environ.get("EDGEQUAKE_FACT_PROTECT_BM25") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "coverage_protect_first": int(
            os.environ.get("EDGEQUAKE_COVERAGE_PROTECT_FIRST") or "0"
        ),
        "topic_entity_admit": (
            os.environ.get("EDGEQUAKE_TOPIC_ENTITY_ADMIT") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "topic_ce_protect": (
            os.environ.get("EDGEQUAKE_TOPIC_CE_PROTECT") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "topic_trunc_protect": (
            os.environ.get("EDGEQUAKE_TOPIC_TRUNC_PROTECT") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "topic_trunc_protect_max": int(
            os.environ.get("EDGEQUAKE_TOPIC_TRUNC_PROTECT_MAX") or "4"
        ),
        "topic_materialize": (
            os.environ.get("EDGEQUAKE_TOPIC_MATERIALIZE") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "topic_materialize_max": int(
            os.environ.get("EDGEQUAKE_TOPIC_MATERIALIZE_MAX") or "4"
        ),
        "topic_materialize_content": (
            os.environ.get("EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "topic_materialize_types": (
            os.environ.get("EDGEQUAKE_TOPIC_MATERIALIZE_TYPES") or ""
        )
        .strip()
        .lower(),
        "intent_rerank": (
            os.environ.get("EDGEQUAKE_INTENT_RERANK") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        # Mirror Rust product defaults (SPEC-086 E2-occ): on unless explicitly off.
        "l2_bm25_union": (
            os.environ.get("EDGEQUAKE_L2_BM25_UNION") or "1"
        )
        .strip()
        .lower()
        not in {"0", "false", "no", "off"},
        "l2_bm25_mix_top_k": int(
            os.environ.get("EDGEQUAKE_L2_BM25_MIX_TOP_K") or "30"
        ),
        "l2_bm25_mode": (
            os.environ.get("EDGEQUAKE_L2_BM25_MODE") or "fact_replace"
        )
        .strip()
        .lower()
        or "fact_replace",
        "mix_intent_weights": (
            os.environ.get("EDGEQUAKE_MIX_INTENT_WEIGHTS") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "intent_factual_bias": (
            os.environ.get("EDGEQUAKE_INTENT_FACTUAL_BIAS") or ""
        )
        .strip()
        .lower()
        in {"1", "true", "yes", "on"},
        "answer_prompt": (
            os.environ.get("EDGEQUAKE_ANSWER_PROMPT") or "default"
        )
        .strip()
        .lower()
        or "default",
        "answer_specific_types": (
            os.environ.get("EDGEQUAKE_ANSWER_SPECIFIC_TYPES") or ""
        )
        .strip()
        .lower(),
        # 031 B3a: FAQ→markdown heading induction at ingest (Acc labeled).
        "structure_induce": (
            os.environ.get("EDGEQUAKE_STRUCTURE_INDUCE")
            or os.environ.get("BENCH001_STRUCTURE_INDUCE")
            or "off"
        )
        .strip()
        .lower()
        or "off",
        "min_chunk_budget_ratio": float(
            os.environ.get("EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO") or "0.40"
        ),
        "query_arm_concurrency": int(
            os.environ.get("EDGEQUAKE_QUERY_ARM_CONCURRENCY") or "16"
        ),
        "adaptive_chunking": adaptive,
        "chunk_token_size": chunk_token_size(),
        "chunk_overlap_token_size": chunk_overlap_token_size(),
        "ingest_max_chars": ingest_max_chars(),
        "eq_query_concurrency": int(
            os.environ.get("BENCH001_QUERY_CONCURRENCY")
            or os.environ.get("BENCH001_ACC_QUERY_CONCURRENCY")
            or "0"
        )
        or None,
        "lr_query_concurrency_effective": int(
            os.environ.get("BENCH001_LR_QUERY_CONCURRENCY") or "2"
        ),
        "fairness_note": (
            "Matched top-k=30 budgets; LR rerank off (no model); "
            "L2 Evidence Recall + Context Relevancy required for valid smoke+; "
            "EQ Mix arm gate off (LR-like always-on local+global+naive) unless "
            "EDGEQUAKE_MIX_ARM_GATE=true on the server; "
            "Mix fusion default round_robin (SPEC-086 E2-occ; rrf is labeled ablation); "
            "Acc PATH_PRUNE=0 (022 P0; soft path only with CE+protect); "
            "Phase-1 EDGEQUAKE_MIX_RELEVANCY_PRUNE Acc default off; "
            "fair Acc ingest: adaptive_chunking off + chunk_token_size=1200 "
            "(LightRAG CHUNK_SIZE parity) unless explicitly ablated; "
            "smoke-fast Acc may set BENCH001_INGEST_MAX_CHARS for fast force-ingest "
            "(full corpus = 0)"
        ),
    }


def resolve_publish_profile_id(base: str) -> str:
    if not publish_fairness_enabled():
        return base
    if base == "P0_mistral_mix":
        base = PUBLISH_PROFILE_ID
    # Tag LR-like always-on arms when Mix gate is disabled (server must match).
    if not mix_arm_gate_enabled() and LRLIKE_ARMS_PROFILE_SUFFIX not in base:
        if base.endswith("_v2"):
            return f"{base[:-3]}_{LRLIKE_ARMS_PROFILE_SUFFIX}"
        return f"{base}_{LRLIKE_ARMS_PROFILE_SUFFIX}"
    return base
