#!/usr/bin/env bash
# 032 B3b — workspace-scoped AGE node identity + markdown + gleaning (NO FAQ induce).
# Fresh workspace → audit density → A1 query-only (concurrency≤4).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

unset BENCH001_EQ_WORKSPACE_ID || true
export BENCH001_EQ_WORKSPACE_NAME="${BENCH001_EQ_WORKSPACE_NAME:-bench001-b3b-ws-scoped}"
export BENCH001_CHUNK_STRATEGY="${BENCH001_CHUNK_STRATEGY:-markdown}"
export BENCH001_ENABLE_GLEANING="${BENCH001_ENABLE_GLEANING:-1}"
export BENCH001_MAX_GLEANING="${BENCH001_MAX_GLEANING:-1}"
# First principles: no flaky FAQ structure induce on Acc.
export EDGEQUAKE_STRUCTURE_INDUCE=0
unset BENCH001_STRUCTURE_INDUCE || true
export BENCH001_PUBLICATION=1
export BENCH001_FULL_ACC=1
export BENCH001_INGEST_MAX_CHARS=0
export EDGEQUAKE_ADAPTIVE_CHUNKING=0
export EDGEQUAKE_CHUNK_SIZE=1200
export EDGEQUAKE_CHUNK_OVERLAP=100
export BENCH001_ACC_QUERY_CONCURRENCY="${BENCH001_ACC_QUERY_CONCURRENCY:-4}"
export BENCH001_ACC_EVAL_CONCURRENCY="${BENCH001_ACC_EVAL_CONCURRENCY:-8}"
export PYTHONPATH="tools/bench001:${PYTHONPATH:-}"
export PYTHONUNBUFFERED=1
export LLM_API_KEY="${LLM_API_KEY:-${MISTRAL_API_KEY:-}}"

ACC_PORT="${BENCH001_ACC_PORT:-8090}"
echo "==> 032 B3b force-ingest (ws-scoped graph ids + markdown + glean) → A1"
echo "==> STRUCTURE_INDUCE=$EDGEQUAKE_STRUCTURE_INDUCE (must be off)"

# Disk gate: AGE relationship merge spills to pgsql_tmp; ENOSPC triggers saga wipe.
avail_gi="$(df -g /System/Volumes/Data 2>/dev/null | awk 'NR==2{print $4}')"
avail_gi="${avail_gi:-0}"
if [[ "${avail_gi}" -lt 15 ]]; then
  echo "ERROR: host free space ${avail_gi}Gi < 15Gi — refuse Acc force-ingest (ENOSPC risk)" >&2
  exit 2
fi
echo "==> host free ≈ ${avail_gi}Gi (ok)"

python3 tools/bench001/scripts/start_acc_backend.py --port "$ACC_PORT"
set -a
[[ -f /tmp/edgequake-dev-ports.env ]] && . /tmp/edgequake-dev-ports.env
[[ -f "$ROOT/.edgequake-dev-ports.env" ]] && . "$ROOT/.edgequake-dev-ports.env"
set +a
export EDGEQUAKE_API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:${ACC_PORT}}"

set +e
python3 -m bench001.cli smoke --api "$EDGEQUAKE_API_URL" --force-ingest \
  --llm-provider mistral --llm-model "${BENCH001_ACC_LLM_MODEL:-mistral-small-latest}" \
  --vision-provider mistral --vision-model "${BENCH001_ACC_LLM_MODEL:-mistral-small-latest}" \
  --embedding-provider mistral --embedding-model mistral-embed --embedding-dim 1024 \
  --judge-provider mistral --judge-model "${BENCH001_ACC_JUDGE_MODEL:-mistral-small-latest}" \
  --judge-embedding-model mistral-embed \
  --answer-style gold \
  --profile-id B3b_ws_scoped_graph_md_glean_v1 \
  --query-concurrency "${BENCH001_ACC_QUERY_CONCURRENCY:-4}" \
  --eval-concurrency "${BENCH001_ACC_EVAL_CONCURRENCY:-8}"
smoke_rc=$?
set -e

# Prefer archive eq_workspace from this force-ingest (warm may stay on B2 if invalid).
NEW_WS="$(
  PYTHONPATH="tools/bench001:${PYTHONPATH:-}" python3 - <<'PY'
import json
from pathlib import Path
from bench001.paths import ARTIFACTS_DIR
p = ARTIFACTS_DIR / "smoke" / "eq_workspace.json"
if p.is_file():
    d = json.loads(p.read_text())
    print((d.get("workspace_id") or "").strip())
PY
)"
if [[ -n "${NEW_WS}" ]]; then
  export BENCH001_EQ_WORKSPACE_ID="$NEW_WS"
elif [[ -z "${BENCH001_EQ_WORKSPACE_ID:-}" ]]; then
  BENCH001_EQ_WORKSPACE_ID="$(cd tools/bench001 && PYTHONPATH=. python3 -m bench001.cli resolve-warm-workspace)"
  export BENCH001_EQ_WORKSPACE_ID
fi
echo "==> audit workspace after B3b ingest: $BENCH001_EQ_WORKSPACE_ID (smoke_rc=$smoke_rc)"
python3 tools/bench001/scripts/audit_eq_lr_ingest.py | tee /tmp/bench001-b3b-audit.json
age_over="$(python3 - <<'PY'
import json,re,sys
from pathlib import Path
# audit prints identity_parity JSON near end; also read latest SUMMARY
root=Path('specs/001-benchmark/e2e/artifacts/ingest-audit')
dirs=sorted(root.glob('*'), key=lambda p: p.name, reverse=True)
for d in dirs:
    jp=d/'audit_report.json'
    if jp.is_file():
        ip=json.loads(jp.read_text()).get('identity_parity') or {}
        print(ip.get('age_over_vectors') if ip.get('age_over_vectors') is not None else '')
        break
PY
)"
echo "==> identity age_over_vectors=${age_over:-missing}"
python3 - <<PY
age = "${age_over}".strip()
if not age:
    raise SystemExit("B3b gate fail: missing age_over_vectors (empty ingest?)")
v = float(age)
if not (0.90 <= v <= 1.10):
    raise SystemExit(f"B3b gate fail: age_over_vectors={v} not in [0.90, 1.10]")
print(f"B3b identity gate OK age_over_vectors={v}")
PY

echo "==> A1 query-only on B3b workspace (concurrency≤4)"
./tools/bench001/scripts/run_p_ladder_acc.sh a1
