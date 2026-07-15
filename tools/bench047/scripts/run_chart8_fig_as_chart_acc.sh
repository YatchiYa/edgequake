#!/usr/bin/env bash
# SPEC-047 / 026: Acc #2 after W1-coexist + W1-fig-as-chart rebuild.
# Run ONLY after coexist Acc finishes (do not restart mid-run).
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
export MISTRAL_MODEL=mistral-small-latest
export EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_VISION_MODEL=mistral-small-latest
export VLM_PROCESS_ENABLE=true

ENSURE="$REPO_ROOT/tools/bench047/scripts/ensure_backend_small.sh"
TAG="chart8-026-fig-as-chart-$(date +%Y%m%d-%H%M)"
ART="$REPO_ROOT/specs/047-rag-evaluation/e2e/artifacts"
LOG_DIR="$ART/smoke/logs"
mkdir -p "$LOG_DIR"
LOG="$LOG_DIR/figas-$TAG.log"

echo "=== rebuild/restart Small with fig-as-chart binary ==="
# Binary should already include promote_fig_as_chart (cargo build -p edgequake)
"$ENSURE" restart-parallel
curl -sf "$EDGEQUAKE_API_URL/health" | python3 -c "
import sys,json
d=json.load(sys.stdin)
print('build', d['build_info']['build_number'], d['build_info']['build_timestamp'])
assert d['status'] in ('healthy','degraded')
"

echo "=== Acc smoke TAG=$TAG ==="
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

echo "=== fidelity + snapshot ==="
python3 -m bench047.cli fidelity smoke --api "$EDGEQUAKE_API_URL" || true
mkdir -p "$ART/smoke-$TAG"
cp -a "$ART/smoke/." "$ART/smoke-$TAG/"
python3 -m bench047.cli report "$ART/smoke-$TAG" \
  --compare "$ART/smoke-chart8-026-coexist-20260715-1547" \
  >"$ART/smoke-$TAG/COMPARE_vs_coexist.md" 2>&1 || true
python3 -m bench047.cli report "$ART/smoke-$TAG" \
  --compare "$ART/smoke-chart8-026-crop-expand-20260715-0535" \
  >"$ART/smoke-$TAG/COMPARE_vs_crop_expand.md" 2>&1 || true
echo "DONE artifact=$ART/smoke-$TAG"
