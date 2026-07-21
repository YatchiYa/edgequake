#!/usr/bin/env bash
# 022 P0–P5 + 024 Q0–Q4 labeled Acc ladder (warm query-only against full-corpus workspace).
#
# Usage:
#   export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>   # or rely on warm_workspace.json
#   cargo build --release --bin edgequake
#   ./tools/bench001/scripts/run_p_ladder_acc.sh p0|…|t0|t0b|t1|…|a0|a1|a1l2|…
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

STEP="${1:-}"
if [[ -z "$STEP" ]]; then
  echo "usage: $0 p0|…|a1fp|c1a|c1b|c1d|c1e|c1cold|a1fplr|…|a4" >&2
  exit 2
fi

if [[ -z "${BENCH001_EQ_WORKSPACE_ID:-}" ]]; then
  BENCH001_EQ_WORKSPACE_ID="$(cd tools/bench001 && PYTHONPATH=. python3 -m bench001.cli resolve-warm-workspace)"
  export BENCH001_EQ_WORKSPACE_ID
fi

# Acc headline base (022 P0): BM25 · PATH off · popular fallback off · arms on · RRF
export EDGEQUAKE_MIX_RELEVANCY_PRUNE=0
export EDGEQUAKE_RERANKER=bm25
export EDGEQUAKE_PATH_PRUNE=0
export EDGEQUAKE_PATH_PRUNE_FRACTION=0
export EDGEQUAKE_RERANK_PROTECT_FIRST=0
export EDGEQUAKE_ENTITY_RANK=degree
export EDGEQUAKE_RELATION_SELECT=default
export EDGEQUAKE_RELATED_CHUNK_NUMBER=5
export EDGEQUAKE_MIX_LOCAL_WEIGHT=1
export EDGEQUAKE_MIX_GLOBAL_WEIGHT=1
export EDGEQUAKE_MIX_NAIVE_WEIGHT=1
export EDGEQUAKE_MIX_FUSION=rrf
export EDGEQUAKE_MIX_ARM_GATE=false
export EDGEQUAKE_CONTEXT_FORMAT=flat
export EDGEQUAKE_PASSAGE_PACK=0
export EDGEQUAKE_GRAPH_WALK_COMPRESS=0
export EDGEQUAKE_POPULAR_NODE_FALLBACK=0
export EDGEQUAKE_KEYWORD_LEXICAL_BOOST=0
export EDGEQUAKE_CONTENT_HEADINGS=0
export EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=0
export EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=0
export EDGEQUAKE_L2_SOURCES_UNION=0
export EDGEQUAKE_L2_SOURCES_MIX_TOP_K=30
unset EDGEQUAKE_FACT_RERANKER || true
unset EDGEQUAKE_FACT_CE_SKIP || true
export EDGEQUAKE_FACT_PROTECT_BM25=0
export EDGEQUAKE_FACT_CE_SKIP=0
export EDGEQUAKE_KEYWORD_MODE=llm
unset EDGEQUAKE_KEYWORD_LLM_PROVIDER || true
unset EDGEQUAKE_KEYWORD_LLM_MODEL || true
export EDGEQUAKE_KEYWORD_LLM_PROVIDER=
export EDGEQUAKE_KEYWORD_LLM_MODEL=
export EDGEQUAKE_COVERAGE_PROTECT_FIRST=0
export EDGEQUAKE_TOPIC_ENTITY_ADMIT=0
export EDGEQUAKE_TOPIC_CE_PROTECT=0
export EDGEQUAKE_TOPIC_TRUNC_PROTECT=0
export EDGEQUAKE_TOPIC_TRUNC_PROTECT_MAX=4
export EDGEQUAKE_TOPIC_MATERIALIZE=0
export EDGEQUAKE_TOPIC_MATERIALIZE_MAX=4
export EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT=0
unset EDGEQUAKE_TOPIC_MATERIALIZE_TYPES || true
export EDGEQUAKE_INTENT_RERANK=0
export EDGEQUAKE_L2_BM25_UNION=0
export EDGEQUAKE_L2_BM25_MIX_TOP_K=30
export EDGEQUAKE_L2_BM25_MODE=union
export EDGEQUAKE_INTENT_FACTUAL_BIAS=0
export EDGEQUAKE_ANSWER_PROMPT=default
unset EDGEQUAKE_ANSWER_SPECIFIC_TYPES || true
export EDGEQUAKE_QUERY_ARM_CONCURRENCY="${EDGEQUAKE_QUERY_ARM_CONCURRENCY:-16}"
# (ANSWER_PROMPT reset each step; a1fpspec/a1fpscx set specific)
export BENCH001_QUERY_ONLY=1
export BENCH001_EQ_RERANK_TOP_K=30
# 063: Acc Acc/default keeps LR LLM cache warm; latency cold peer sets 0.
export BENCH001_LR_ENABLE_LLM_CACHE="${BENCH001_LR_ENABLE_LLM_CACHE:-1}"

# S1 CE+protect overlay (for p1b / p2b / p4 / q0 / q3 / q4 labeled packages)
_apply_s1() {
  export EDGEQUAKE_RERANKER=cross_encoder
  export EDGEQUAKE_RERANKER_PROVIDER=aliyun
  export EDGEQUAKE_RERANKER_MODEL=qwen3-rerank
  export EDGEQUAKE_RERANK_PROTECT_FIRST=12
  export EDGEQUAKE_PATH_PRUNE=0
  export EDGEQUAKE_PATH_PRUNE_FRACTION=0
}

_apply_p2b() {
  _apply_s1
  export EDGEQUAKE_ENTITY_RANK=retrieval
  export EDGEQUAKE_CONTEXT_FORMAT=path
  export EDGEQUAKE_PATH_PRUNE=1
  export EDGEQUAKE_PATH_PRUNE_FRACTION=0.4
  export EDGEQUAKE_CONTENT_HEADINGS=1
}

case "$STEP" in
  p0)
    PROFILE=P0_path_off_bm25_restore_v1
    NOTE="P0: Acc headline PATH_PRUNE=0 BM25 restore (fix T011703Z BM25+path=0.4 confound)"
    ;;
  p1a)
    export EDGEQUAKE_GRAPH_WALK_COMPRESS=1
    PROFILE=P1a_gw_compress_bm25_v1
    NOTE="P1a: GRAPH_WALK_COMPRESS=1 on BM25 Acc base"
    ;;
  p1b)
    _apply_s1
    export EDGEQUAKE_GRAPH_WALK_COMPRESS=1
    PROFILE=P1b_gw_compress_s1_v1
    NOTE="P1b: GRAPH_WALK_COMPRESS=1 on S1 CE+protect (L2)"
    ;;
  p2a)
    export EDGEQUAKE_MIX_FUSION=round_robin
    export BENCH001_ALLOW_ROUND_ROBIN=1
    PROFILE=P2a_round_robin_fusion_v1
    NOTE="P2a: MIX_FUSION=round_robin (LightRAG parity ablation)"
    ;;
  p2b)
    _apply_p2b
    PROFILE=P2b_lr_pack_s1_v1
    NOTE="P2b: ENTITY_RANK=retrieval + CONTEXT_FORMAT=path + soft path0.4 + headings on S1 only"
    ;;
  q0)
    # 024: P2b stability (identical pins to p2b)
    _apply_p2b
    PROFILE=Q0_p2b_stability_v1
    NOTE="024 Q0: P2b stability Acc (S1+retrieval+path0.4+headings)"
    ;;
  q1)
    export EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=1
    PROFILE=Q1_occurrence_sort_p0_v1
    NOTE="024 Q1: KG_CHUNK_OCCURRENCE_SORT=1 on P0 BM25 (Fact order)"
    ;;
  q2)
    export EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=1
    PROFILE=Q2_vector_lr_budget_p0_v1
    NOTE="024 Q2: KG_CHUNK_PICK_LR_BUDGET=1 on P0 BM25 (uncapped VECTOR pool)"
    ;;
  q3)
    # Single Fact winner on P2b — set BENCH001_Q3_FACT_KNOB=occurrence|lr_budget
    _apply_p2b
    case "${BENCH001_Q3_FACT_KNOB:-occurrence}" in
      lr_budget|vector_lr|q2)
        export EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=1
        PROFILE=Q3_p2b_plus_lr_budget_v1
        NOTE="024 Q3: P2b + KG_CHUNK_PICK_LR_BUDGET=1 (one confound)"
        ;;
      *)
        export EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=1
        PROFILE=Q3_p2b_plus_occurrence_v1
        NOTE="024 Q3: P2b + KG_CHUNK_OCCURRENCE_SORT=1 (one confound)"
        ;;
    esac
    ;;
  q4)
    # Final CI decision — defaults to P2b; override with BENCH001_Q4_PACKAGE=p2b|occurrence|lr_budget
    _apply_p2b
    case "${BENCH001_Q4_PACKAGE:-p2b}" in
      occurrence|q1|q3_occ)
        export EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=1
        PROFILE=Q4_acc_ci_p2b_occurrence_v1
        NOTE="024 Q4: Acc CI decision (P2b+occurrence); promote only if Beat/Parity gates"
        ;;
      lr_budget|q2|q3_lr)
        export EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=1
        PROFILE=Q4_acc_ci_p2b_lr_budget_v1
        NOTE="024 Q4: Acc CI decision (P2b+lr_budget); promote only if Beat/Parity gates"
        ;;
      *)
        PROFILE=Q4_acc_ci_p2b_v1
        NOTE="024 Q4: Acc CI decision (P2b alone); promote only if Beat/Parity gates"
        ;;
    esac
    ;;
  r0)
    # 025: widen Mix protect under CE so Fact/Summarize evidence survives admission
    _apply_p2b
    export EDGEQUAKE_RERANK_PROTECT_FIRST=20
    PROFILE=R0_p2b_protect20_v1
    NOTE="025 R0: P2b + RERANK_PROTECT_FIRST=20 (recall under CE)"
    ;;
  r1)
    _apply_p2b
    export EDGEQUAKE_MIN_RERANK_SCORE=0
    PROFILE=R1_p2b_min_rerank0_v1
    NOTE="025 R1: P2b + MIN_RERANK_SCORE=0 (no hard-drop before protect)"
    ;;
  r2)
    _apply_p2b
    export EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO=0.55
    PROFILE=R2_p2b_chunk_budget055_v1
    NOTE="025 R2: P2b + MIN_CHUNK_BUDGET_RATIO=0.55 (Summarize floor)"
    ;;
  r3)
    # Decision: set BENCH001_R3_PACKAGE=protect20|min_rerank0|chunk055|combo|p2b
    _apply_p2b
    case "${BENCH001_R3_PACKAGE:-protect20}" in
      min_rerank0|r1)
        export EDGEQUAKE_MIN_RERANK_SCORE=0
        PROFILE=R3_acc_ci_p2b_min_rerank0_v1
        NOTE="025 R3: Acc CI (P2b+min_rerank0); promote only if Beat/Parity"
        ;;
      chunk055|r2)
        export EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO=0.55
        PROFILE=R3_acc_ci_p2b_chunk055_v1
        NOTE="025 R3: Acc CI (P2b+chunk_budget0.55); promote only if Beat/Parity"
        ;;
      combo|protect20_min_rerank0)
        export EDGEQUAKE_RERANK_PROTECT_FIRST=20
        export EDGEQUAKE_MIN_RERANK_SCORE=0
        PROFILE=R3_acc_ci_p2b_protect20_min_rerank0_v1
        NOTE="025 R3: Acc CI (P2b+protect20+min_rerank0); promote only if Beat/Parity"
        ;;
      p2b|baseline)
        PROFILE=R3_acc_ci_p2b_v1
        NOTE="025 R3: Acc CI (P2b baseline); promote only if Beat/Parity"
        ;;
      *)
        export EDGEQUAKE_RERANK_PROTECT_FIRST=20
        PROFILE=R3_acc_ci_p2b_protect20_v1
        NOTE="025 R3: Acc CI (P2b+protect20); promote only if Beat/Parity"
        ;;
    esac
    ;;
  s0)
    # 026: dual-list L2 sources (Mix∪CE) under P2b — prompt stays CE-ordered
    _apply_p2b
    export EDGEQUAKE_L2_SOURCES_UNION=1
    export EDGEQUAKE_L2_SOURCES_MIX_TOP_K="${EDGEQUAKE_L2_SOURCES_MIX_TOP_K:-30}"
    PROFILE=S0_p2b_l2_sources_union_v1
    NOTE="026 S0: P2b + L2_SOURCES_UNION=1 (Mix∪CE citations; Acc from CE prompt)"
    ;;
  s1)
    # Decision: P2b + L2 union (default) or BENCH001_S1_PACKAGE=p2b|union
    _apply_p2b
    case "${BENCH001_S1_PACKAGE:-union}" in
      p2b|baseline)
        PROFILE=S1_acc_ci_p2b_v1
        NOTE="026 S1: Acc CI (P2b baseline); promote only if Beat/Parity"
        ;;
      *)
        export EDGEQUAKE_L2_SOURCES_UNION=1
        export EDGEQUAKE_L2_SOURCES_MIX_TOP_K="${EDGEQUAKE_L2_SOURCES_MIX_TOP_K:-30}"
        PROFILE=S1_acc_ci_p2b_l2_union_v1
        NOTE="026 S1: Acc CI (P2b+L2_SOURCES_UNION); promote only if Beat/Parity"
        ;;
    esac
    ;;
  t0)
    # 027 closed path: Fact→BM25 on prompt (Acc toxic — kept for reproducibility)
    _apply_p2b
    export EDGEQUAKE_FACT_RERANKER=bm25
    export EDGEQUAKE_INTENT_RERANK=1
    PROFILE=T0_p2b_fact_bm25_v1
    NOTE="027 T0: P2b + FACT_RERANKER=bm25 on prompt (Acc tax; prefer t0b)"
    ;;
  t0b)
    # 027b: L2 BM25-first ∪ CE — prompt stays CE+protect (Acc-safe)
    _apply_p2b
    export EDGEQUAKE_L2_BM25_UNION=1
    export EDGEQUAKE_L2_BM25_MODE=union
    export EDGEQUAKE_L2_BM25_MIX_TOP_K="${EDGEQUAKE_L2_BM25_MIX_TOP_K:-30}"
    PROFILE=T0b_p2b_l2_bm25_union_v1
    NOTE="027 T0b: P2b + L2_BM25_UNION=1 mode=union BM25-first (CE prompt)"
    ;;
  t0c)
    # 027c: L2 sources = BM25(Mix) only — closest to BM25 Acc Fact ER; CE prompt
    _apply_p2b
    export EDGEQUAKE_L2_BM25_UNION=1
    export EDGEQUAKE_L2_BM25_MODE=replace
    export EDGEQUAKE_L2_BM25_MIX_TOP_K="${EDGEQUAKE_L2_BM25_MIX_TOP_K:-30}"
    PROFILE=T0c_p2b_l2_bm25_replace_v1
    NOTE="027 T0c: P2b + L2_BM25_UNION mode=replace (BM25 sources; CE prompt)"
    ;;
  t0d)
    # 027d: Fact-only BM25 L2; other intents keep CE sources (ctx_rel safe)
    _apply_p2b
    export EDGEQUAKE_L2_BM25_UNION=1
    export EDGEQUAKE_L2_BM25_MODE=fact_replace
    export EDGEQUAKE_L2_BM25_MIX_TOP_K="${EDGEQUAKE_L2_BM25_MIX_TOP_K:-30}"
    PROFILE=T0d_p2b_l2_bm25_fact_replace_v1
    NOTE="027 T0d: P2b + L2_BM25_MODE=fact_replace (Fact BM25 L2; else CE)"
    ;;
  t1)
    _apply_p2b
    case "${BENCH001_T1_PACKAGE:-fact_replace}" in
      p2b|baseline)
        PROFILE=T1_acc_ci_p2b_v1
        NOTE="027 T1: Acc CI (P2b baseline); promote only if Beat/Parity"
        ;;
      fact_bm25|t0)
        export EDGEQUAKE_FACT_RERANKER=bm25
        export EDGEQUAKE_INTENT_RERANK=1
        PROFILE=T1_acc_ci_p2b_fact_bm25_v1
        NOTE="027 T1: Acc CI (P2b+Fact→BM25 prompt); promote only if Beat/Parity"
        ;;
      union|t0b)
        export EDGEQUAKE_L2_BM25_UNION=1
        export EDGEQUAKE_L2_BM25_MODE=union
        export EDGEQUAKE_L2_BM25_MIX_TOP_K="${EDGEQUAKE_L2_BM25_MIX_TOP_K:-30}"
        PROFILE=T1_acc_ci_p2b_l2_bm25_union_v1
        NOTE="027 T1: Acc CI (P2b+L2 BM25-first union); promote only if Beat/Parity"
        ;;
      replace|t0c)
        export EDGEQUAKE_L2_BM25_UNION=1
        export EDGEQUAKE_L2_BM25_MODE=replace
        export EDGEQUAKE_L2_BM25_MIX_TOP_K="${EDGEQUAKE_L2_BM25_MIX_TOP_K:-30}"
        PROFILE=T1_acc_ci_p2b_l2_bm25_replace_v1
        NOTE="027 T1: Acc CI (P2b+L2 BM25 replace); promote only if Beat/Parity"
        ;;
      *)
        export EDGEQUAKE_L2_BM25_UNION=1
        export EDGEQUAKE_L2_BM25_MODE=fact_replace
        export EDGEQUAKE_L2_BM25_MIX_TOP_K="${EDGEQUAKE_L2_BM25_MIX_TOP_K:-30}"
        PROFILE=T1_acc_ci_p2b_l2_bm25_fact_replace_v1
        NOTE="027 T1: Acc CI (P2b+L2 FactReplace); promote only if Beat/Parity"
        ;;
    esac
    ;;
  p3a)
    # Intent truncation floors are code-default; this run audits query_intent on predictions.
    PROFILE=P3a_intent_trunc_audit_v1
    NOTE="P3a: BM25 + intent chunk floor (log query_intent on predictions)"
    ;;
  p3b)
    export EDGEQUAKE_KEYWORD_LEXICAL_BOOST=1
    export EDGEQUAKE_POPULAR_NODE_FALLBACK=0
    PROFILE=P3b_keyword_lexical_boost_v1
    NOTE="P3b: KEYWORD_LEXICAL_BOOST=1 + popular-node fallback off"
    ;;
  p4)
    # Decision package: best labeled stack (S1 + gw compress + retrieval order).
    # Promote only if CI excludes 0 AND ctx_rel≥0.50 — see 022 §P4.
    _apply_s1
    export EDGEQUAKE_GRAPH_WALK_COMPRESS=1
    export EDGEQUAKE_ENTITY_RANK=retrieval
    export EDGEQUAKE_KEYWORD_LEXICAL_BOOST=1
    PROFILE=P4_acc_ci_decision_v1
    NOTE="P4: Acc CI decision package (S1+gw+retrieval+lexical); promote only if gates green"
    ;;
  p5)
    export EDGEQUAKE_QUERY_ARM_CONCURRENCY=24
    export BENCH001_ACC_QUERY_CONCURRENCY="${BENCH001_ACC_QUERY_CONCURRENCY:-8}"
    PROFILE=P5_latency_arm24_v1
    NOTE="P5: arm concurrency 24 + query timing; target EQ/LR p50 ≤1.5×"
    ;;
  a0)
    # 028 A0: P2b stability baseline for Horizon A ladder
    _apply_p2b
    PROFILE=A0_p2b_baseline_v1
    NOTE="028 A0: P2b baseline Acc (S1+retrieval+path0.4+headings)"
    ;;
  a1)
    # 028 A1: LR-like Relations→Entities→Chunks serialization under P2b
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    PROFILE=A1_p2b_rr_cer_v1
    NOTE="028 A1: P2b + CONTEXT_FORMAT=rr_cer (relation-first pack; path prune still on)"
    ;;
  a1l2)
    # 034: A1 pack + dual-list L2 (Mix∪CE citations; prompt stays CE)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_L2_SOURCES_UNION=1
    export EDGEQUAKE_L2_SOURCES_MIX_TOP_K="${EDGEQUAKE_L2_SOURCES_MIX_TOP_K:-30}"
    PROFILE=A1L2_p2b_rr_cer_l2_union_v1
    NOTE="034 a1l2: A1 + L2_SOURCES_UNION=1 (citation budget fix; Acc from CE prompt)"
    ;;
  a1lr)
    # 034: A1 pack + LightRAG VECTOR chunk budget (uncapped entity-linked pool)
    # One confound vs T090743Z — L2 union OFF (a1l2 Acc tax / Fact ER flat).
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=1
    PROFILE=A1LR_p2b_rr_cer_kg_lr_budget_v1
    NOTE="034 a1lr: A1 + KG_CHUNK_PICK_LR_BUDGET=1 (related×n_entities/2 VECTOR take)"
    ;;
  a1lrl2)
    # 034 decision: a1lr Mix pool + dual-list L2 (promote only Beat/Parity)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=1
    export EDGEQUAKE_L2_SOURCES_UNION=1
    export EDGEQUAKE_L2_SOURCES_MIX_TOP_K="${EDGEQUAKE_L2_SOURCES_MIX_TOP_K:-30}"
    PROFILE=A1LRL2_p2b_rr_cer_lr_budget_l2_union_v1
    NOTE="034 a1lrl2: A1 + LR VECTOR budget + L2_SOURCES_UNION (Parity decision)"
    ;;
  a1fp)
    # 035: A1 + Fact BM25 first-stage for CE protect (no dual-list / no LR budget)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    PROFILE=A1FP_p2b_rr_cer_fact_protect_bm25_v1
    NOTE="035 a1fp: A1 + FACT_PROTECT_BM25=1 (BM25 Mix→CE protect; no dual-list)"
    ;;
  c1a)
    # 057/058: product latency Fact CE-skip on A1 pack — NOT Acc promote
    # (Acc Fact peer stays a1fp + CE; this pack measures Fact-row rerank ↓)
    # Dual-pin FACT_RERANKER=bm25 so Acc backend override + fair_pins both see it.
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=0
    export EDGEQUAKE_FACT_CE_SKIP=1
    export EDGEQUAKE_FACT_RERANKER=bm25
    PROFILE=C1A_a1_rr_cer_fact_ce_skip_v1
    NOTE="058 c1a: A1 + FACT_CE_SKIP=1 + FACT_RERANKER=bm25 — latency peer; not Acc promote"
    ;;
  c1b)
    # 059: product BM25-all (no CE HTTP) — one confound vs Acc Fact CE peer.
    # Acc Fact peer stays a1fp. Measures full CE removal; keyword/embed split in SUMMARY.
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=0
    unset EDGEQUAKE_FACT_CE_SKIP || true
    unset EDGEQUAKE_FACT_RERANKER || true
    export EDGEQUAKE_RERANKER=bm25
    PROFILE=C1B_a1_rr_cer_bm25_all_v1
    NOTE="059 c1b: A1 + RERANKER=bm25 (all intents; no CE) — latency peer; not Acc promote"
    ;;
  c1d)
    # 060: c1b + KEYWORD_MODE=heuristic (skip keyword LLM) — one confound on c1b.
    # Acc Fact peer stays a1fp + KEYWORD_MODE=llm. Not Acc promote.
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=0
    unset EDGEQUAKE_FACT_CE_SKIP || true
    unset EDGEQUAKE_FACT_RERANKER || true
    export EDGEQUAKE_RERANKER=bm25
    export EDGEQUAKE_KEYWORD_MODE=heuristic
    PROFILE=C1D_a1_rr_cer_bm25_heuristic_kw_v1
    NOTE="060 c1d: A1 + BM25-all + KEYWORD_MODE=heuristic — latency peer; not Acc promote"
    ;;
  c1e)
    # 062: c1b + fast KEYWORD LLM (LightRAG KEYWORD role) — one confound on c1b.
    # QUERY stays mistral-small; KEYWORD = ministral-3b (same provider). Not Acc promote.
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=0
    unset EDGEQUAKE_FACT_CE_SKIP || true
    unset EDGEQUAKE_FACT_RERANKER || true
    export EDGEQUAKE_RERANKER=bm25
    export EDGEQUAKE_KEYWORD_MODE=llm
    export EDGEQUAKE_KEYWORD_LLM_PROVIDER=mistral
    export EDGEQUAKE_KEYWORD_LLM_MODEL=ministral-3b-latest
    PROFILE=C1E_a1_rr_cer_bm25_fast_keyword_v1
    NOTE="062 c1e: A1 + BM25-all + KEYWORD=ministral-3b — latency peer; not Acc promote"
    ;;
  c1cold)
    # 063: c1b + LR enable_llm_cache=False — fair cold dual-SUT latency.
    # Acc archives with warm LR keywords+query cache are NOT fair for EQ/LR p50.
    # Acc Fact peer unchanged. Same models (mistral-small) both sides.
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=0
    unset EDGEQUAKE_FACT_CE_SKIP || true
    unset EDGEQUAKE_FACT_RERANKER || true
    export EDGEQUAKE_RERANKER=bm25
    export EDGEQUAKE_KEYWORD_MODE=llm
    unset EDGEQUAKE_KEYWORD_LLM_PROVIDER || true
    unset EDGEQUAKE_KEYWORD_LLM_MODEL || true
    export EDGEQUAKE_KEYWORD_LLM_PROVIDER=
    export EDGEQUAKE_KEYWORD_LLM_MODEL=
    export BENCH001_LR_ENABLE_LLM_CACHE=0
    PROFILE=C1COLD_a1_rr_cer_bm25_lr_nocache_v1
    NOTE="063 c1cold: C1b + LR LLM cache OFF — fair cold latency; not Acc promote"
    ;;
  a1fprw)
    # 051: a1fp + LightRAG relation select (rank+weight); one confound; no TOPIC_*
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_RELATION_SELECT=lightrag
    PROFILE=A1FPRW_p2b_rr_cer_fact_protect_rel_select_lr_v1
    NOTE="051 a1fprw: A1 + FACT_PROTECT_BM25 + RELATION_SELECT=lightrag (no dual-list)"
    ;;
  a1fplr)
    # 035 decision: a1fp + LR VECTOR budget (Parity without dual-list Acc tax)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=1
    PROFILE=A1FPLR_p2b_rr_cer_fact_protect_lr_budget_v1
    NOTE="035 a1fplr: A1 + FACT_PROTECT_BM25 + LR VECTOR budget (no dual-list)"
    ;;
  a1fpm0)
    # 036: a1fp + min_rerank=0 (CE coverage; no dual-list / no LR budget)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_MIN_RERANK_SCORE=0
    PROFILE=A1FPM0_p2b_rr_cer_fact_protect_min_rerank0_v1
    NOTE="036 a1fpm0: A1 + FACT_PROTECT_BM25 + MIN_RERANK_SCORE=0 (no dual-list)"
    ;;
  a1fpcov)
    # 036: a1fp + Exploratory protect=30 (CE reorder, Mix set membership)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_COVERAGE_PROTECT_FIRST=30
    PROFILE=A1FPCOV_p2b_rr_cer_fact_protect_cov30_v1
    NOTE="036 a1fpcov: A1 + FACT_PROTECT_BM25 + COVERAGE_PROTECT_FIRST=30 (no dual-list)"
    ;;
  a1fpsel)
    # 038: a1fp + Exploratory topic-entity admission (SELECT; no densify/dual-list/LR-budget)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_TOPIC_ENTITY_ADMIT=1
    PROFILE=A1FPSEL_p2b_rr_cer_fact_protect_topic_admit_v1
    NOTE="038 a1fpsel: A1 + FACT_PROTECT_BM25 + TOPIC_ENTITY_ADMIT=1 (Exploratory SELECT)"
    ;;
  a1fpce)
    # 039: a1fp + topic admit + CE/fuse protect for topic_admit_chunk_ids
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_TOPIC_ENTITY_ADMIT=1
    export EDGEQUAKE_TOPIC_CE_PROTECT=1
    PROFILE=A1FPCE_p2b_rr_cer_fact_protect_topic_ce_v1
    NOTE="039 a1fpce: A1 + FACT_PROTECT_BM25 + TOPIC_ENTITY_ADMIT + TOPIC_CE_PROTECT (Exploratory)"
    ;;
  a1fptrunc)
    # 040: a1fpce + trunc/pack prefer for topic_admit_chunk_ids (capped)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_TOPIC_ENTITY_ADMIT=1
    export EDGEQUAKE_TOPIC_CE_PROTECT=1
    export EDGEQUAKE_TOPIC_TRUNC_PROTECT=1
    export EDGEQUAKE_TOPIC_TRUNC_PROTECT_MAX=4
    PROFILE=A1FPTRUNC_p2b_rr_cer_fact_protect_topic_trunc_v1
    NOTE="040 a1fptrunc: A1 + admit + CE protect + TOPIC_TRUNC_PROTECT (Exploratory pack)"
    ;;
  a1fpmat)
    # 042: a1fp + admit + KV materialize topic CONTENT into Mix (CE_GAP)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_TOPIC_ENTITY_ADMIT=1
    export EDGEQUAKE_TOPIC_MATERIALIZE=1
    export EDGEQUAKE_TOPIC_MATERIALIZE_MAX=4
    PROFILE=A1FPMAT_p2b_rr_cer_fact_protect_topic_mat_v1
    NOTE="042 a1fpmat: A1 + TOPIC_ENTITY_ADMIT + TOPIC_MATERIALIZE (KV into Mix before CE)"
    ;;
  a1fpcmat)
    # 045: a1fpmat + CONTENT bigram gate (043 leftover — Sum ER without blind Acc tax)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_TOPIC_ENTITY_ADMIT=1
    export EDGEQUAKE_TOPIC_MATERIALIZE=1
    export EDGEQUAKE_TOPIC_MATERIALIZE_MAX=4
    export EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT=1
    PROFILE=A1FPCMAT_p2b_rr_cer_fact_protect_topic_mat_content_v1
    NOTE="045 a1fpcmat: A1 + admit + MATERIALIZE + CONTENT gate (phrase-hit KV only)"
    ;;
  a1fpspec)
    # 046: a1fp + ANSWER_PROMPT=specific (Complex Acc — name Context entities)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_ANSWER_PROMPT=specific
    PROFILE=A1FPSPEC_p2b_rr_cer_fact_protect_answer_specific_v1
    NOTE="046 a1fpspec: A1 + FACT_PROTECT_BM25 + ANSWER_PROMPT=specific (no TOPIC_*)"
    ;;
  a1fpscx)
    # 047: Complex-only specificity (046 Acc tax → type-scoped)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_ANSWER_PROMPT=specific
    export EDGEQUAKE_ANSWER_SPECIFIC_TYPES=complex
    PROFILE=A1FPSCX_p2b_rr_cer_fact_protect_answer_specific_complex_v1
    NOTE="047 a1fpscx: A1 + FACT_PROTECT_BM25 + ANSWER_PROMPT=specific + SPECIFIC_TYPES=complex"
    ;;
  a1fpsumx)
    # 048: Summarize-only CONTENT materialize (045 Acc tax → type-routed)
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_FACT_PROTECT_BM25=1
    export EDGEQUAKE_TOPIC_ENTITY_ADMIT=1
    export EDGEQUAKE_TOPIC_MATERIALIZE=1
    export EDGEQUAKE_TOPIC_MATERIALIZE_MAX=4
    export EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT=1
    export EDGEQUAKE_TOPIC_MATERIALIZE_TYPES=summarize
    PROFILE=A1FPSUMX_p2b_rr_cer_fact_protect_topic_mat_content_summarize_v1
    NOTE="048 a1fpsumx: A1 + admit + CONTENT mat + MATERIALIZE_TYPES=summarize"
    ;;
  a2)
    # 028 A2: Fact intent coverage (LLM bias) on A1 pack
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_INTENT_FACTUAL_BIAS=1
    PROFILE=A2_p2b_rr_cer_fact_bias_v1
    NOTE="028 A2: P2b+rr_cer + INTENT_FACTUAL_BIAS=1 (no L2 heuristic-OR)"
    ;;
  a3)
    # 028 A3: LightRAG-like answer prompt on A2 pack
    _apply_p2b
    export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
    export EDGEQUAKE_INTENT_FACTUAL_BIAS=1
    export EDGEQUAKE_ANSWER_PROMPT=lightrag
    PROFILE=A3_p2b_rr_cer_fact_bias_lr_prompt_v1
    NOTE="028 A3: P2b+rr_cer+fact_bias + ANSWER_PROMPT=lightrag"
    ;;
  a4)
    # 028 A4: Acc CI decision — promote only on Beat/Parity gates
    _apply_p2b
    case "${BENCH001_A4_PACKAGE:-a3}" in
      a0|p2b|baseline)
        PROFILE=A4_acc_ci_p2b_v1
        NOTE="028 A4: Acc CI (P2b); promote only if Beat/Parity"
        ;;
      a1|rr_cer)
        export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
        PROFILE=A4_acc_ci_a1_rr_cer_v1
        NOTE="028 A4: Acc CI (P2b+rr_cer); promote only if Beat/Parity"
        ;;
      a2|fact_bias)
        export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
        export EDGEQUAKE_INTENT_FACTUAL_BIAS=1
        PROFILE=A4_acc_ci_a2_fact_bias_v1
        NOTE="028 A4: Acc CI (P2b+rr_cer+fact_bias); promote only if Beat/Parity"
        ;;
      *)
        export EDGEQUAKE_CONTEXT_FORMAT=rr_cer
        export EDGEQUAKE_INTENT_FACTUAL_BIAS=1
        export EDGEQUAKE_ANSWER_PROMPT=lightrag
        PROFILE=A4_acc_ci_a3_package_v1
        NOTE="028 A4: Acc CI (A3 package); promote only if Beat/Parity"
        ;;
    esac
    ;;
  *)
    echo "unknown step: $STEP" >&2
    exit 2
    ;;
esac

ACC_PORT="${BENCH001_ACC_PORT:-8090}"
echo "==> $NOTE"
echo "==> profile=$PROFILE workspace=$BENCH001_EQ_WORKSPACE_ID port=$ACC_PORT"

python3 tools/bench001/scripts/start_acc_backend.py --port "$ACC_PORT"

set -a
[[ -f /tmp/edgequake-dev-ports.env ]] && . /tmp/edgequake-dev-ports.env
[[ -f "$ROOT/.edgequake-dev-ports.env" ]] && . "$ROOT/.edgequake-dev-ports.env"
set +a
export EDGEQUAKE_API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:${ACC_PORT}}"
export BENCH001_PUBLICATION=1
export BENCH001_FULL_ACC=1
export EDGEQUAKE_ADAPTIVE_CHUNKING=0
export EDGEQUAKE_CHUNK_SIZE=1200
export EDGEQUAKE_CHUNK_OVERLAP=100
export PYTHONPATH="tools/bench001:${PYTHONPATH:-}"
export PYTHONUNBUFFERED=1
export LLM_API_KEY="${LLM_API_KEY:-${MISTRAL_API_KEY:-}}"

python3 -m bench001.cli smoke --api "$EDGEQUAKE_API_URL" --query-only \
  --llm-provider mistral --llm-model "${BENCH001_ACC_LLM_MODEL:-mistral-small-latest}" \
  --vision-provider mistral --vision-model "${BENCH001_ACC_LLM_MODEL:-mistral-small-latest}" \
  --embedding-provider mistral --embedding-model mistral-embed --embedding-dim 1024 \
  --judge-provider mistral --judge-model "${BENCH001_ACC_JUDGE_MODEL:-mistral-small-latest}" \
  --judge-embedding-model mistral-embed \
  --answer-style gold \
  --profile-id "$PROFILE" \
  --query-concurrency "${BENCH001_ACC_QUERY_CONCURRENCY:-8}" \
  --eval-concurrency "${BENCH001_ACC_EVAL_CONCURRENCY:-16}"

ART="$(ls -td specs/001-benchmark/e2e/artifacts/history/smoke-* 2>/dev/null | head -1 || true)"
if [[ -n "$ART" && ! -f "$ART/ABLATION_NOTE.md" ]]; then
  cat >"$ART/ABLATION_NOTE.md" <<EOF
# Ablation — $PROFILE

**Step:** $STEP  
**Pins:** $NOTE  
**Workspace:** \`${BENCH001_EQ_WORKSPACE_ID}\`

## Gates (fill from SUMMARY)

| Gate | Target | Result |
|------|--------|--------|
| path_prune_fraction pin | 0 for P0/P1a/P3/P5 | |
| Δ Acc 95% CI | includes 0 (P0) / excludes 0 EQ (P4) | |
| EQ ctx_rel | ≥0.48 (P1) / ≥0.50 (P4) | |
| Complex Acc Δ vs LR | ≤0.05 (P1/P2) | |
| Summarize evidence_recall | ≥0.95 or ≥LR−0.03 (P3) | |
| EQ/LR p50 ratio | ≤1.5× (P5) | |

## Verdict

- [ ] Gate met
- [ ] Gate missed (do not promote)
EOF
  echo "==> wrote $ART/ABLATION_NOTE.md"
fi
