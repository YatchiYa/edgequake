#!/usr/bin/env bash
# SPEC-047 / 032 / 034: Acc #5 — W3-arith-v2 (MUST compute + worked example).
# Query-only against Acc #4 workspace (no re-ingest). Gen-only causal change.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$REPO_ROOT"
[ -f .env ] && set -a && . ./.env && set +a
export PATH="${HOME}/.venv/bin:${PATH}"
export MISTRAL_API_KEY
export EDGEQUAKE_API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:8090}"
export BENCH047_WORKER_THREADS="${BENCH047_WORKER_THREADS:-6}"
export BENCH047_MAX_TASKS_PER_TENANT="${BENCH047_MAX_TASKS_PER_TENANT:-2}"
export BENCH047_PDF_VISION_JOBS="${BENCH047_PDF_VISION_JOBS:-2}"
export PYTHONUNBUFFERED=1
export PYTHONPATH="${REPO_ROOT}/tools/bench047:${PYTHONPATH:-}"
export MISTRAL_MODEL=mistral-small-latest
export EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_VISION_MODEL=mistral-small-latest
export VLM_PROCESS_ENABLE=true

ENSURE="$REPO_ROOT/tools/bench047/scripts/ensure_backend_small.sh"
TAG="chart8-026-w3-arith-v2-$(date +%Y%m%d-%H%M)"
ART="$REPO_ROOT/specs/047-rag-evaluation/e2e/artifacts"
LOG_DIR="$ART/smoke/logs"
mkdir -p "$LOG_DIR"
LOG="$LOG_DIR/w3arithv2-$TAG.log"
BASE_FIGAS="$ART/smoke-chart8-026-fig-as-chart-20260715-1707"
BASE_ACC4="$ART/smoke-chart8-026-w3-arith-20260715-2012"

echo "=== cargo build (W3-arith-v2 MUST + example) ==="
(cd "$REPO_ROOT/edgequake" && cargo build -p edgequake >"$LOG_DIR/w3arithv2-build-$TAG.log" 2>&1)
tail -20 "$LOG_DIR/w3arithv2-build-$TAG.log"

echo "=== restart Small with Acc #5 binary ==="
"$ENSURE" restart-parallel
curl -sf "$EDGEQUAKE_API_URL/health" | python3 -c "
import sys,json
d=json.load(sys.stdin)
print('build', d['build_info']['build_number'], d['build_info']['build_timestamp'])
assert d['status'] in ('healthy','degraded')
"

echo "=== seed smoke/ from Acc #4 workspace (query-only) ==="
rm -rf "$ART/smoke"
mkdir -p "$ART/smoke/logs"
cp -a "$BASE_ACC4/meta.json" "$BASE_ACC4/ingest.jsonl" "$ART/smoke/"
# Keep prior predictions out so query-only rewrites them
cp -a "$BASE_ACC4/." "$ART/smoke/" 2>/dev/null || true
rm -f "$ART/smoke/predictions.jsonl" "$ART/smoke/scorecard.json" "$ART/smoke/SUMMARY.md" "$ART/smoke/fidelity.json" "$ART/smoke/FIDELITY.md"

echo "=== Acc #5 query-only TAG=$TAG workspace=$(python3 -c "import json;print(json.load(open('$ART/smoke/meta.json'))['workspace_id'])") ==="
python3 -m bench047.cli smoke \
  --api "$EDGEQUAKE_API_URL" \
  --profile P0_mm_ite \
  --query-only \
  --document-scope \
  --workers 1 \
  --i-accept-cost \
  2>&1 | tee "$LOG"

echo "=== fidelity + snapshot ==="
python3 -m bench047.cli fidelity smoke --api "$EDGEQUAKE_API_URL" || true
mkdir -p "$ART/smoke-$TAG"
cp -a "$ART/smoke/." "$ART/smoke-$TAG/"
python3 -m bench047.cli report "$ART/smoke-$TAG" \
  --compare "$BASE_FIGAS" \
  >"$ART/smoke-$TAG/COMPARE_vs_figas_acc2.md" 2>&1 || true
python3 -m bench047.cli report "$ART/smoke-$TAG" \
  --compare "$BASE_ACC4" \
  >"$ART/smoke-$TAG/COMPARE_vs_acc4.md" 2>&1 || true
echo "DONE_OK artifact=$ART/smoke-$TAG"
