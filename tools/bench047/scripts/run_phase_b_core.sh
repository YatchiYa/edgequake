#!/usr/bin/env bash
# SPEC-047 Phase B (Stage B CORE): ~40 docs with checkpoint assessment every 5 docs.
# Stack: BEST_SCORE_STACK / Acc #5 W3-arith-v2 (P0_mm_ite + document-scope + Small).
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
export BENCH047_INGEST_WORKERS="${BENCH047_INGEST_WORKERS:-1}"
export PYTHONUNBUFFERED=1
export PYTHONPATH="${REPO_ROOT}/tools/bench047:${PYTHONPATH:-}"
export MISTRAL_MODEL=mistral-small-latest
export EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_VISION_MODEL=mistral-small-latest
export VLM_PROCESS_ENABLE=true
export BENCH047_PROFILE=P0_mm_ite

ENSURE="$REPO_ROOT/tools/bench047/scripts/ensure_backend_small.sh"
ART="$REPO_ROOT/specs/047-rag-evaluation/e2e/artifacts"
CORE_DIR="$ART/core"
CHECKPOINT_ROOT="$ART/core-checkpoints"
TAG="core-phase-b-$(date +%Y%m%d-%H%M)"
LOG_DIR="$ART/smoke/logs"
mkdir -p "$LOG_DIR" "$CHECKPOINT_ROOT"
LOG="$LOG_DIR/phase-b-$TAG.log"

echo "=== freeze core fixture (if needed) ==="
if grep -q PLACEHOLDER "$REPO_ROOT/specs/047-rag-evaluation/fixtures/core_doc_ids_v1.txt" 2>/dev/null; then
  python3 -m bench047.cli download-qa
  python3 -m bench047.cli freeze-core -n 40
fi
N_DOCS=$(grep -c '\.pdf$' "$REPO_ROOT/specs/047-rag-evaluation/fixtures/core_doc_ids_v1.txt" || true)
echo "core fixture n_docs=$N_DOCS"

echo "=== cargo build (best-score stack) ==="
(cd "$REPO_ROOT/edgequake" && cargo build -p edgequake >"$LOG_DIR/phase-b-build-$TAG.log" 2>&1)
tail -15 "$LOG_DIR/phase-b-build-$TAG.log"

echo "=== restart Small backend ==="
"$ENSURE" restart-parallel
python3 -m bench047.cli doctor --api "$EDGEQUAKE_API_URL" --profile P0_mm_ite

echo "=== download core PDFs ==="
python3 -m bench047.cli download-pdfs --fixture core_doc_ids_v1.txt

# Fresh core run unless BENCH047_RESUME=1
if [ "${BENCH047_RESUME:-0}" != "1" ]; then
  rm -rf "$CORE_DIR"
  mkdir -p "$CORE_DIR/logs"
fi

_assess_checkpoint() {
  local n="$1"
  local cp="$CHECKPOINT_ROOT/at_${n}_docs"
  mkdir -p "$cp"
  if [ -f "$CORE_DIR/scorecard.json" ]; then
    cp -a "$CORE_DIR/scorecard.json" "$CORE_DIR/SUMMARY.md" "$cp/" 2>/dev/null || true
    python3 -m bench047.cli fidelity core --api "$EDGEQUAKE_API_URL" \
      >"$cp/FIDELITY.md" 2>&1 || true
    [ -f "$CORE_DIR/fidelity.json" ] && cp "$CORE_DIR/fidelity.json" "$cp/" || true
    # Shell-expand n into the heredoc (quoted <<'PY' would break $CORE_DIR/$cp).
    python3 <<PY
import json
from pathlib import Path
from datetime import datetime, timezone
n = int("$n")
sc = json.loads(Path("$CORE_DIR/scorecard.json").read_text())
m = sc["metrics"]
lines = [
    f"# Phase B checkpoint — {n} docs",
    "",
    f"**Time:** {datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}",
    "**Stack:** P0_mm_ite · W3-arith-v2 · protocol 026-listmem",
    "",
    f"- Acc: **{m['accuracy']:.4f}**",
    f"- F1: **{m['f1']:.4f}**",
    f"- valid: {sc.get('valid')}",
    f"- ingest_coverage: {(sc.get('ops') or {}).get('ingest_coverage')}",
    "",
    "See SUMMARY.md / FIDELITY.md in this folder.",
]
Path("$cp/ASSESSMENT.md").write_text("\n".join(lines) + "\n")
print(f"checkpoint at_{n}_docs Acc={m['accuracy']:.4f} F1={m['f1']:.4f}")
PY
  else
    echo "WARN: no scorecard yet at n=$n"
  fi
}

echo "=== Phase B CORE: assess every 5 docs (tag=$TAG) ===" | tee "$LOG"
RESUME_FLAG="--no-resume"
START_N="${BENCH047_START_N:-5}"
for N in 5 10 15 20 25 30 35 40; do
  [ "$N" -gt "$N_DOCS" ] && break
  [ "$N" -lt "$START_N" ] && continue
  # Skip re-run if checkpoint assessment already present (resume mid-ladder).
  if [ "${BENCH047_RESUME:-0}" = "1" ] && [ -f "$CHECKPOINT_ROOT/at_${N}_docs/ASSESSMENT.md" ]; then
    echo "--- skip max-docs=$N (checkpoint exists) ---" | tee -a "$LOG"
    RESUME_FLAG="--resume"
    continue
  fi
  echo "--- batch max-docs=$N resume=$RESUME_FLAG ---" | tee -a "$LOG"
  python3 -m bench047.cli core \
    --api "$EDGEQUAKE_API_URL" \
    --profile P0_mm_ite \
    --document-scope \
    --max-docs "$N" \
    $RESUME_FLAG \
    --workers 1 \
    --ingest-workers 1 \
    --i-accept-cost \
    2>&1 | tee -a "$LOG"
  _assess_checkpoint "$N"
  RESUME_FLAG="--resume"
done

# Final snapshot
SNAP="$ART/core-$TAG"
mkdir -p "$SNAP"
cp -a "$CORE_DIR/." "$SNAP/"
_assess_checkpoint "$N_DOCS"
cp -a "$CHECKPOINT_ROOT/at_${N_DOCS}_docs/." "$SNAP/checkpoint-final/" 2>/dev/null || mkdir -p "$SNAP/checkpoint-final"

echo "DONE_OK artifact=$SNAP checkpoints=$CHECKPOINT_ROOT" | tee -a "$LOG"
