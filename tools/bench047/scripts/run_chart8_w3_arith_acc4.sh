#!/usr/bin/env bash
# SPEC-047 / 026 / 032: Acc #4 — W3-arith (grounded %×N) + year-span expand.
# Acc #3 densify callouts REVERTED. Baseline compare: Acc #2 fig-as-chart.
# Honesty: Acc↑ from W3-arith = Gen composition, NOT a W1 Chart representation claim.
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
TAG="chart8-026-w3-arith-$(date +%Y%m%d-%H%M)"
ART="$REPO_ROOT/specs/047-rag-evaluation/e2e/artifacts"
LOG_DIR="$ART/smoke/logs"
mkdir -p "$LOG_DIR"
LOG="$LOG_DIR/w3arith-$TAG.log"
BASE_FIGAS="$ART/smoke-chart8-026-fig-as-chart-20260715-1707"
BASE_DENSE="$ART/smoke-chart8-026-dense-scalar-20260715-1845"

echo "=== cargo build (W3-arith + year-span; densify reverted) ==="
# Do NOT pipe through `tail` under `set -o pipefail` — early close SIGPIPEs cargo.
(cd "$REPO_ROOT/edgequake" && cargo build -p edgequake >"$LOG_DIR/w3arith-build-$TAG.log" 2>&1)
tail -30 "$LOG_DIR/w3arith-build-$TAG.log"

echo "=== restart Small with Acc #4 binary ==="
"$ENSURE" restart-parallel
curl -sf "$EDGEQUAKE_API_URL/health" | python3 -c "
import sys,json
d=json.load(sys.stdin)
print('build', d['build_info']['build_number'], d['build_info']['build_timestamp'])
assert d['status'] in ('healthy','degraded')
llm=(d.get('providers') or {}).get('llm') or {}
assert 'small' in str(llm.get('model','')).lower(), llm
"

echo "=== Acc #4 smoke TAG=$TAG ==="
rm -rf "$ART/smoke"
mkdir -p "$ART/smoke/logs"
python3 -m bench047.cli smoke \
  --api "$EDGEQUAKE_API_URL" \
  --profile P0_mm_ite \
  --no-resume \
  --document-scope \
  --workers 1 \
  --ingest-workers 1 \
  --i-accept-cost \
  2>&1 | tee "$LOG"

echo "=== fidelity (026-listmem + year-span) + snapshot ==="
python3 -m bench047.cli fidelity smoke --api "$EDGEQUAKE_API_URL" || true
mkdir -p "$ART/smoke-$TAG"
cp -a "$ART/smoke/." "$ART/smoke-$TAG/"
python3 -m bench047.cli report "$ART/smoke-$TAG" \
  --compare "$BASE_FIGAS" \
  >"$ART/smoke-$TAG/COMPARE_vs_figas_acc2.md" 2>&1 || true
if [ -d "$BASE_DENSE" ]; then
  python3 -m bench047.cli report "$ART/smoke-$TAG" \
    --compare "$BASE_DENSE" \
    >"$ART/smoke-$TAG/COMPARE_vs_dense_acc3.md" 2>&1 || true
fi
echo "DONE_OK artifact=$ART/smoke-$TAG"
