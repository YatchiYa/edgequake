#!/usr/bin/env bash
# 028/030 B2 — forced Acc re-ingest into a NEW workspace (markdown + gleaning),
# then A1 query-only Acc smoke. Never overwrites the warm peer silently.
#
# Usage:
#   cargo build --release --bin edgequake
#   ./tools/bench001/scripts/run_b2_reingest_acc.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

# Force a fresh EQ workspace (do not reuse warm peer).
unset BENCH001_EQ_WORKSPACE_ID || true
export BENCH001_EQ_WORKSPACE_NAME="${BENCH001_EQ_WORKSPACE_NAME:-bench001-b2-md-glean}"
export BENCH001_CHUNK_STRATEGY="${BENCH001_CHUNK_STRATEGY:-markdown}"
export BENCH001_ENABLE_GLEANING="${BENCH001_ENABLE_GLEANING:-1}"
export BENCH001_MAX_GLEANING="${BENCH001_MAX_GLEANING:-1}"
export BENCH001_PUBLICATION=1
export BENCH001_FULL_ACC=1
export BENCH001_INGEST_MAX_CHARS=0
export EDGEQUAKE_ADAPTIVE_CHUNKING=0
export EDGEQUAKE_CHUNK_SIZE=1200
export EDGEQUAKE_CHUNK_OVERLAP=100
export PYTHONPATH="tools/bench001:${PYTHONPATH:-}"
export PYTHONUNBUFFERED=1
export LLM_API_KEY="${LLM_API_KEY:-${MISTRAL_API_KEY:-}}"

ACC_PORT="${BENCH001_ACC_PORT:-8090}"
echo "==> 030 B2 force-ingest (markdown+gleaning) → new workspace → A1 query-only"
echo "==> chunk_strategy=$BENCH001_CHUNK_STRATEGY gleaning=$BENCH001_ENABLE_GLEANING/$BENCH001_MAX_GLEANING"

python3 tools/bench001/scripts/start_acc_backend.py --port "$ACC_PORT"
set -a
[[ -f /tmp/edgequake-dev-ports.env ]] && . /tmp/edgequake-dev-ports.env
[[ -f "$ROOT/.edgequake-dev-ports.env" ]] && . "$ROOT/.edgequake-dev-ports.env"
set +a
export EDGEQUAKE_API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:${ACC_PORT}}"

# Cold ingest (publication full corpus). Client creates a fresh workspace.
python3 -m bench001.cli smoke --api "$EDGEQUAKE_API_URL" --force-ingest \
  --llm-provider mistral --llm-model "${BENCH001_ACC_LLM_MODEL:-mistral-small-latest}" \
  --vision-provider mistral --vision-model "${BENCH001_ACC_LLM_MODEL:-mistral-small-latest}" \
  --embedding-provider mistral --embedding-model mistral-embed --embedding-dim 1024 \
  --judge-provider mistral --judge-model "${BENCH001_ACC_JUDGE_MODEL:-mistral-small-latest}" \
  --judge-embedding-model mistral-embed \
  --answer-style gold \
  --profile-id B2_md_glean_force_ingest_v1 \
  --query-concurrency "${BENCH001_ACC_QUERY_CONCURRENCY:-8}" \
  --eval-concurrency "${BENCH001_ACC_EVAL_CONCURRENCY:-16}"

# Persist warm pointer from this run, then A1 query-only on the new graph.
if [[ -z "${BENCH001_EQ_WORKSPACE_ID:-}" ]]; then
  BENCH001_EQ_WORKSPACE_ID="$(cd tools/bench001 && PYTHONPATH=. python3 -m bench001.cli resolve-warm-workspace)"
  export BENCH001_EQ_WORKSPACE_ID
fi
echo "==> warm pointer after B2 ingest: $BENCH001_EQ_WORKSPACE_ID"
python3 tools/bench001/scripts/audit_eq_lr_ingest.py || true

echo "==> A1 query-only on B2 workspace"
# Cap concurrency: CE+path under Acc has hit Postgres pool timeouts at 8.
export BENCH001_ACC_QUERY_CONCURRENCY="${BENCH001_ACC_QUERY_CONCURRENCY:-4}"
export BENCH001_ACC_EVAL_CONCURRENCY="${BENCH001_ACC_EVAL_CONCURRENCY:-8}"
./tools/bench001/scripts/run_p_ladder_acc.sh a1
