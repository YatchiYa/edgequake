#!/usr/bin/env bash
# 049 B6 — rel source-chunk union inherit + markdown + gleaning (NO FAQ).
# Fresh workspace → audit zero-chunk ≤1% → a1fp query-only (concurrency≤4).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

unset BENCH001_EQ_WORKSPACE_ID || true
export BENCH001_EQ_WORKSPACE_NAME="${BENCH001_EQ_WORKSPACE_NAME:-bench001-b6-rel-chunk-union}"
export BENCH001_CHUNK_STRATEGY="${BENCH001_CHUNK_STRATEGY:-markdown}"
export BENCH001_ENABLE_GLEANING="${BENCH001_ENABLE_GLEANING:-1}"
export BENCH001_MAX_GLEANING="${BENCH001_MAX_GLEANING:-1}"
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
echo "==> 049 B6 force-ingest (rel source-chunk union + md + glean) → a1fp"
echo "==> STRUCTURE_INDUCE=$EDGEQUAKE_STRUCTURE_INDUCE (must be off)"

avail_gi="$(df -g /System/Volumes/Data 2>/dev/null | awk 'NR==2{print $4}')"
avail_gi="${avail_gi:-0}"
if [[ "${avail_gi}" -lt 15 ]]; then
  echo "ERROR: host free space ${avail_gi}Gi < 15Gi — refuse Acc force-ingest (ENOSPC risk)" >&2
  exit 2
fi
echo "==> host free ≈ ${avail_gi}Gi (ok)"

# Rebuild Acc binary so rel source-chunk union is live.
echo "==> rebuild Acc backend (edgequake) for B6 code"
(
  cd edgequake
  cargo build --release --bin edgequake
)

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
  --profile-id B6_rel_dedup_source_chunk_union_v1 \
  --query-concurrency "${BENCH001_ACC_QUERY_CONCURRENCY:-4}" \
  --eval-concurrency "${BENCH001_ACC_EVAL_CONCURRENCY:-8}"
smoke_rc=$?
set -e

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
echo "==> audit workspace after B6 ingest: $BENCH001_EQ_WORKSPACE_ID (smoke_rc=$smoke_rc)"
python3 tools/bench001/scripts/audit_eq_lr_ingest.py | tee /tmp/bench001-b6-audit.json

python3 - <<'PY'
import json, os, re
from pathlib import Path
root = Path("specs/001-benchmark/e2e/artifacts/ingest-audit")
ws = (os.environ.get("BENCH001_EQ_WORKSPACE_ID") or "").strip()
# Prefer UTC timestamp dirs (028 B1 layout); skip topic-fidelity/summarize audits.
dirs = sorted(
    [d for d in root.iterdir() if d.is_dir() and re.fullmatch(r"\d{8}T\d{6}Z", d.name)],
    key=lambda p: p.name,
    reverse=True,
)
report = None
for d in dirs:
    jp = d / "audit_report.json"
    if not jp.is_file():
        continue
    cand = json.loads(jp.read_text())
    if ws and cand.get("workspace_id") != ws:
        continue
    if "linkage_density" not in cand:
        continue
    report = cand
    print(f"using audit {d.name}")
    break
if not report:
    raise SystemExit("B6 gate fail: missing B1 audit_report.json for workspace")
ip = report.get("identity_parity") or {}
age = ip.get("age_over_vectors")
stub = report.get("stub_provenance") or {}
rate = stub.get("eq_zero_chunk_rate")
print(
    f"identity age_over_vectors={age} stub_zero_rate={rate} "
    f"stubs={stub.get('eq_unknown_empty_stubs')}"
)
if age is None:
    raise SystemExit("B6 gate fail: missing age_over_vectors")
v = float(age)
if not (0.90 <= v <= 1.20):
    # Slightly wider than B3b: stubs may remain vectorless until entity extract fills them.
    raise SystemExit(f"B6 gate fail: age_over_vectors={v} not in [0.90, 1.20]")
if rate is None:
    raise SystemExit("B6 gate fail: missing stub_provenance.eq_zero_chunk_rate")
if float(rate) > 0.01:
    raise SystemExit(f"B6 gate fail: eq_zero_chunk_rate={rate} > 0.01")
rl = report.get("relation_linkage") or {}
ge2 = rl.get("eq_edges_ge2_rate")
print(
    f"B6 provenance gate OK zero_rate={rate} age_over_vectors={v} "
    f"eq_edges_ge2_rate={ge2} mean_chunks/edge={rl.get('eq_mean_chunks_per_edge')}"
)
if ge2 is None:
    raise SystemExit("B6 gate fail: missing relation_linkage.eq_edges_ge2_rate")
# Structural: some edges must carry multi-chunk lineage after union dedupe.
if float(ge2) < 0.05:
    raise SystemExit(f"B6 gate fail: eq_edges_ge2_rate={ge2} < 0.05 (union not live?)")
PY

echo "==> a1fp query-only on B6 workspace (Acc Fact peer pack)"
./tools/bench001/scripts/run_p_ladder_acc.sh a1fp
