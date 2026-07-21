#!/usr/bin/env bash
# 021 F1–F4 labeled Acc ladder (warm query-only against full-corpus workspace).
#
# Usage:
#   export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>
#   export DASHSCOPE_API_KEY=...   # S1 CE (qwen3-rerank / DashScope intl)
#   cargo build --release --bin edgequake
#   ./tools/bench001/scripts/run_f_ladder_acc.sh f1a|f2a|f3a|f4a
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

STEP="${1:-}"
if [[ -z "$STEP" ]]; then
  echo "usage: $0 f1a|f2a|f3a|f4a" >&2
  exit 2
fi

if [[ -z "${BENCH001_EQ_WORKSPACE_ID:-}" ]]; then
  echo "BENCH001_EQ_WORKSPACE_ID required (warm full-corpus workspace)" >&2
  exit 2
fi

# S1 CE+protect base (labeled — not Acc headline).
export EDGEQUAKE_MIX_RELEVANCY_PRUNE=0
export EDGEQUAKE_RERANKER=cross_encoder
export EDGEQUAKE_RERANKER_PROVIDER=aliyun
export EDGEQUAKE_RERANKER_MODEL=qwen3-rerank
export EDGEQUAKE_RERANK_PROTECT_FIRST=12
export EDGEQUAKE_ENTITY_RANK=degree
export EDGEQUAKE_RELATED_CHUNK_NUMBER=5
export EDGEQUAKE_MIX_LOCAL_WEIGHT=1
export EDGEQUAKE_MIX_GLOBAL_WEIGHT=1
export EDGEQUAKE_MIX_NAIVE_WEIGHT=1
export BENCH001_EQ_RERANK_TOP_K=30
export EDGEQUAKE_QUERY_ARM_CONCURRENCY="${EDGEQUAKE_QUERY_ARM_CONCURRENCY:-16}"
export BENCH001_QUERY_ONLY=1
export EDGEQUAKE_PASSAGE_PACK=0
export EDGEQUAKE_CONTEXT_FORMAT=flat

case "$STEP" in
  f1a)
    # Intent Summarize floor is code-default (Exploratory/Relational ≥0.60).
    export EDGEQUAKE_PATH_PRUNE=0
    PROFILE=F1a_s1_summarize_trunc_v1
    NOTE="F1a: S1 CE+protect + Summarize-like chunk floor≥0.60 (truncation_config_for_intent)"
    ;;
  f2a)
    # Soft path only with CE+protect: PATH_PRUNE=1 + FRACTION=0.4 (022: never BM25+path).
    export EDGEQUAKE_PATH_PRUNE=1
    export EDGEQUAKE_PATH_PRUNE_FRACTION=0.4
    export EDGEQUAKE_CONTEXT_FORMAT=path
    PROFILE=F2a_path_pack_v1
    NOTE="F2a: CONTEXT_FORMAT=path + PATH_PRUNE=1 FRACTION=0.4 + S1 CE+protect"
    ;;
  f3a)
    export EDGEQUAKE_PATH_PRUNE=0
    export EDGEQUAKE_QUERY_ARM_CONCURRENCY=24
    export BENCH001_ACC_QUERY_CONCURRENCY="${BENCH001_ACC_QUERY_CONCURRENCY:-8}"
    PROFILE=F3a_latency_stage_v1
    NOTE="F3a: stage timing export + arm concurrency 24; fair query concurrency 8"
    ;;
  f4a)
    export EDGEQUAKE_PATH_PRUNE=0
    export EDGEQUAKE_PASSAGE_PACK=1
    PROFILE=F4a_passage_pack_v1
    NOTE="F4a: labeled HippoRAG2-style PASSAGE_PACK=1 (chunks-first)"
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
set +a
export EDGEQUAKE_API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:${ACC_PORT}}"
export BENCH001_PUBLICATION=1
export BENCH001_FULL_ACC=1
export EDGEQUAKE_MIX_ARM_GATE=false
export EDGEQUAKE_ADAPTIVE_CHUNKING=0
export EDGEQUAKE_CHUNK_SIZE=1200
export EDGEQUAKE_CHUNK_OVERLAP=100
export EDGEQUAKE_MIX_FUSION=rrf
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
| Summarize evidence_recall | ≥0.95 or ≥LR−0.03 | |
| Complex ΔF1 vs LR | ≤0.03 | |
| Acc drop vs S1 | ≤0.02 | |
| ctx_rel | ≥0.50 discovery | |
| EQ/LR p50 ratio | ≤1.5 (F3) | |

**Promote?** No — labeled only until Acc CI excludes 0.
EOF
  echo "Wrote $ART/ABLATION_NOTE.md"
fi

echo "→ SUMMARY: specs/001-benchmark/e2e/artifacts/smoke/SUMMARY.md"
