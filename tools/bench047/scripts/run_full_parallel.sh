#!/usr/bin/env bash
# SPEC-047 full dataset — 135 docs / 1091 Qs with ≥10 parallel ingest per workspace.
# Profile: P0_mm_ite · hybrid · document-scope · mistral-small-latest · mistral-embed
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$REPO_ROOT"

ART_ROOT="specs/047-rag-evaluation/e2e/artifacts"
FULL_DIR="$ART_ROOT/full"
RUN_TAG="${BENCH047_RUN_TAG:-full-par10-$(date -u +%Y%m%d-%H%M%S)}"
SNAP_DIR="$ART_ROOT/full-${RUN_TAG}"
API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:8090}"
INGEST_WORKERS="${BENCH047_INGEST_WORKERS:-10}"
QUERY_WORKERS="${BENCH047_WORKERS:-4}"
ENSURE="$REPO_ROOT/tools/bench047/scripts/ensure_backend_small.sh"
PROFILE="${BENCH047_PROFILE:-P0_mm_ite}"

export EDGEQUAKE_API_URL="$API_URL"
export MISTRAL_MODEL=mistral-small-latest
export EDGEQUAKE_LLM_PROVIDER=mistral
export EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_VISION_PROVIDER=mistral
export EDGEQUAKE_VISION_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral
export MISTRAL_EMBEDDING_MODEL=mistral-embed
export VLM_PROCESS_ENABLE=true
export BENCH047_INGEST_WORKERS="$INGEST_WORKERS"
export BENCH047_WORKERS="$QUERY_WORKERS"
# Backend admission so ≥10 workspace PDFs can be in-flight together
export BENCH047_WORKER_THREADS="${BENCH047_WORKER_THREADS:-24}"
export BENCH047_MAX_TASKS_PER_TENANT="${BENCH047_MAX_TASKS_PER_TENANT:-16}"
export BENCH047_PDF_VISION_JOBS="${BENCH047_PDF_VISION_JOBS:-12}"

die() { echo "ERROR: $*" >&2; exit 1; }

echo "=== SPEC-047 full parallel ingest ($RUN_TAG) ==="
echo "ingest_workers=$INGEST_WORKERS query_workers=$QUERY_WORKERS profile=$PROFILE"

# 0) Fail-closed stuck PDFs
echo "=== fail-closed stuck pdf_documents.processing ==="
docker exec edgequake-postgres psql -U edgequake -d edgequake -v ON_ERROR_STOP=1 -c "
UPDATE pdf_documents
SET processing_status = 'failed',
    extraction_errors = coalesce(extraction_errors, '[]'::jsonb)
      || jsonb_build_array(jsonb_build_object(
           'at', now(),
           'reason', 'bench047_pre_full_fail_closed_stale_processing'
         )),
    updated_at = now()
WHERE processing_status IN ('pending', 'processing')
  AND updated_at < now() - interval '30 minutes';
SELECT processing_status, count(*) FROM pdf_documents GROUP BY 1 ORDER BY 1;
" || die "could not fail-closed stale PDFs"

# 1) Backend with parallel admission + Small pins
chmod +x "$ENSURE"
"$ENSURE" restart-parallel || die "backend restart-parallel failed"
"$ENSURE" start-watchdog
"$ENSURE" status || die "backend not Small-healthy"

# 2) Doctor
python3 -m bench047.cli doctor --api "$API_URL" --profile "$PROFILE" || die "doctor FAIL"

# 3) Download all PDFs (full corpus)
echo "=== download all PDFs ==="
python3 -m bench047.cli download-qa
python3 -m bench047.cli download-pdfs --all

# 4) Archive prior full/ if present
mkdir -p "$FULL_DIR"
if [ -f "$FULL_DIR/SUMMARY.md" ] || [ -f "$FULL_DIR/scorecard.json" ] || [ -f "$FULL_DIR/ingest.jsonl" ]; then
  PRE="$ART_ROOT/full-pre-${RUN_TAG}"
  mkdir -p "$PRE"
  cp -a "$FULL_DIR"/. "$PRE"/ || true
  echo "archived prior full → $PRE"
fi

# Fresh ledgers unless BENCH047_RESUME=1
if [ "${BENCH047_RESUME:-0}" = "1" ]; then
  echo "resume mode: keeping existing ledgers in $FULL_DIR"
  RESUME_FLAG=("--resume")
else
  rm -f "$FULL_DIR"/ingest.jsonl "$FULL_DIR"/predictions.jsonl \
    "$FULL_DIR"/scorecard.json "$FULL_DIR"/SUMMARY.md "$FULL_DIR"/meta.json \
    "$FULL_DIR"/fidelity.json "$FULL_DIR"/FIDELITY.md
  mkdir -p "$FULL_DIR/logs"
  RESUME_FLAG=("--no-resume")
fi
mkdir -p "$FULL_DIR/logs"

# 5) Progress monitor
MON_LOG="$FULL_DIR/logs/progress.log"
(
  echo "progress monitor start $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  while true; do
    ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    ing=$( [ -f "$FULL_DIR/ingest.jsonl" ] && wc -l < "$FULL_DIR/ingest.jsonl" | tr -d ' ' || echo 0 )
    pred=$( [ -f "$FULL_DIR/predictions.jsonl" ] && wc -l < "$FULL_DIR/predictions.jsonl" | tr -d ' ' || echo 0 )
    done_c=$( [ -f "$FULL_DIR/ingest.jsonl" ] && grep -c '"status": "completed"' "$FULL_DIR/ingest.jsonl" 2>/dev/null || echo 0 )
    fail_c=$( [ -f "$FULL_DIR/ingest.jsonl" ] && grep -c '"status": "failed"' "$FULL_DIR/ingest.jsonl" 2>/dev/null || echo 0 )
    health=$(curl -fsS -m 2 "$API_URL/health" 2>/dev/null | python3 -c "import sys,json; h=json.load(sys.stdin); print(h.get('status'), (h.get('providers') or {}).get('llm',{}).get('model','?'))" 2>/dev/null || echo "down")
    echo "$ts ingest_lines=$ing completed=$done_c failed=$fail_c pred=$pred health=$health"
    sleep 60
  done
) >>"$MON_LOG" 2>&1 &
MON_PID=$!
trap 'kill $MON_PID 2>/dev/null || true' EXIT

# 6) Full run
echo "=== bench047 full (parallel ingest=$INGEST_WORKERS) ==="
set +e
python3 -m bench047.cli full \
  --api "$API_URL" \
  --profile "$PROFILE" \
  --document-scope \
  --ingest-workers "$INGEST_WORKERS" \
  --workers "$QUERY_WORKERS" \
  --i-accept-cost \
  "${RESUME_FLAG[@]}"
RC=$?
set -e

kill "$MON_PID" 2>/dev/null || true
trap - EXIT

# 7) Snapshot
mkdir -p "$SNAP_DIR"
cp -a "$FULL_DIR"/. "$SNAP_DIR"/ || true
echo "snapshot → $SNAP_DIR"
echo "SUMMARY → $FULL_DIR/SUMMARY.md"
exit "$RC"
