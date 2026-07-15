#!/usr/bin/env bash
# SPEC-047 chart-8 smoke — stronger vision ablation (025 / EQ-047-W1-vision).
# Same Acc physics as run_chart8_smoke.sh EXCEPT vision = mistral-medium-3-5.
# Query LLM stays mistral-small-latest (FP3: one causal change).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$REPO_ROOT"

ART_ROOT="specs/047-rag-evaluation/e2e/artifacts"
SMOKE_DIR="$ART_ROOT/smoke"
RUN_TAG="${BENCH047_RUN_TAG:-chart8-vision-medium-$(date -u +%Y%m%d-%H%M%S)}"
SNAP_DIR="$ART_ROOT/smoke-${RUN_TAG}"
API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:8090}"
WORKERS="${BENCH047_WORKERS:-2}"
INGEST_WORKERS="${BENCH047_INGEST_WORKERS:-4}"
ENSURE="$REPO_ROOT/tools/bench047/scripts/ensure_backend_small.sh"
PROFILE="P0_mm_ite_vision_medium"
VISION_MODEL="mistral-medium-3-5"

export EDGEQUAKE_API_URL="$API_URL"
export EDGEQUAKE_BENCH_FIXTURE=smoke_chart_doc_ids_v1.txt
export MISTRAL_MODEL=mistral-small-latest
export EDGEQUAKE_LLM_PROVIDER=mistral
export EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_VISION_PROVIDER=mistral
export EDGEQUAKE_VISION_MODEL="$VISION_MODEL"
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral
export MISTRAL_EMBEDDING_MODEL=mistral-embed
export VLM_PROCESS_ENABLE=true
export BENCH047_WORKERS="$WORKERS"
export BENCH047_INGEST_WORKERS="$INGEST_WORKERS"

die() { echo "ERROR: $*" >&2; exit 1; }

echo "=== SPEC-047 chart-8 stronger-vision smoke ($RUN_TAG) ==="
echo "profile=$PROFILE llm=mistral-small-latest vision=$VISION_MODEL"

# 0) Fail-closed stuck PDFs
echo "=== fail-closed stuck pdf_documents.processing ==="
docker exec edgequake-postgres psql -U edgequake -d edgequake -v ON_ERROR_STOP=1 -c "
UPDATE pdf_documents
SET processing_status = 'failed',
    extraction_errors = coalesce(extraction_errors, '[]'::jsonb)
      || jsonb_build_array(jsonb_build_object(
           'at', now(),
           'reason', 'bench047_pre_vision_medium_smoke_fail_closed_stale_processing'
         )),
    updated_at = now()
WHERE processing_status IN ('pending', 'processing')
  AND updated_at < now() - interval '30 minutes';
SELECT processing_status, count(*) FROM pdf_documents GROUP BY 1 ORDER BY 1;
" || die "could not fail-closed stale PDFs"

# 1) Backend with Small LLM + Medium vision env (ensure respects EDGEQUAKE_VISION_MODEL)
chmod +x "$ENSURE"
"$ENSURE" start-watchdog
"$ENSURE" status || die "backend not Small-LLM-healthy"

# 2) Doctor (fail closed on vision catalog + VLM)
python3 -m bench047.cli doctor --api "$API_URL" --profile "$PROFILE" || die "doctor FAIL"

# 3) Archive previous smoke/
mkdir -p "$SMOKE_DIR"
if [ -f "$SMOKE_DIR/SUMMARY.md" ] || [ -f "$SMOKE_DIR/scorecard.json" ]; then
  PRE="$ART_ROOT/smoke-pre-${RUN_TAG}"
  mkdir -p "$PRE"
  cp -a "$SMOKE_DIR"/. "$PRE"/ || true
  echo "archived prior smoke → $PRE"
fi

rm -f "$SMOKE_DIR"/ingest.jsonl "$SMOKE_DIR"/predictions.jsonl \
  "$SMOKE_DIR"/scorecard.json "$SMOKE_DIR"/SUMMARY.md "$SMOKE_DIR"/meta.json \
  "$SMOKE_DIR"/fidelity.json "$SMOKE_DIR"/FIDELITY.md
mkdir -p "$SMOKE_DIR/logs"

# 4) Progress monitor
MON_LOG="$SMOKE_DIR/logs/progress.log"
(
  echo "progress monitor start $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  while true; do
    ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    ing=$( [ -f "$SMOKE_DIR/ingest.jsonl" ] && wc -l < "$SMOKE_DIR/ingest.jsonl" || echo 0 )
    pred=$( [ -f "$SMOKE_DIR/predictions.jsonl" ] && wc -l < "$SMOKE_DIR/predictions.jsonl" || echo 0 )
    healthy=$("$ENSURE" status 2>/dev/null | head -c 200 || echo down)
    echo "$ts ingest_rows=$ing pred_rows=$pred backend=$healthy"
    sleep 60
  done
) >>"$MON_LOG" 2>&1 &
MON_PID=$!
disown "$MON_PID" 2>/dev/null || true
echo "progress monitor pid=$MON_PID → $MON_LOG"

# 5) Run smoke
echo "=== running chart-8 vision-medium smoke query_workers=$WORKERS ingest_workers=$INGEST_WORKERS ==="
set +e
python3 -m bench047.cli smoke \
  --api "$API_URL" \
  --profile "$PROFILE" \
  --no-resume \
  --document-scope \
  --workers "$WORKERS" \
  --ingest-workers "$INGEST_WORKERS"
RC=$?
set -e

kill "$MON_PID" 2>/dev/null || true

# 6) Annotate SUMMARY with ablation identity + fidelity gate reminder
if [ -f "$SMOKE_DIR/SUMMARY.md" ]; then
  python3 - <<'PY'
from pathlib import Path
smoke = Path("specs/047-rag-evaluation/e2e/artifacts/smoke")
summary = smoke / "SUMMARY.md"
block = """
## Stronger vision ablation (025)

- Profile: `P0_mm_ite_vision_medium`
- Query LLM: `mistral-small-latest` (unchanged)
- Vision Pass A/B: `mistral-medium-3-5` (only causal change)
- Gate before Acc claims: Chart `answer_in_evidence_rate_long` ≥ 0.50 via `bench047 fidelity` (full-n, gateable=true)
- Baseline: locked `P0_mm_ite` Small+Small chart-8 Acc ~0.415 / Chart a_in_e ~0.40

"""
text = summary.read_text()
marker = "## Stronger vision ablation (025)"
if marker in text:
    text = text.split(marker)[0].rstrip() + "\n"
cite = "## Citation"
if cite in text:
    text = text.replace(cite, block + cite)
else:
    text = text.rstrip() + "\n" + block
summary.write_text(text)
print("annotated SUMMARY with 025 ablation block")
PY
fi

# 7) Snapshot
mkdir -p "$SNAP_DIR"
cp -a "$SMOKE_DIR"/. "$SNAP_DIR"/
echo "snapshot → $SNAP_DIR"

# 8) Gates
if [ -f "$SMOKE_DIR/SUMMARY.md" ]; then
  head -60 "$SMOKE_DIR/SUMMARY.md"
fi
if [ -f "$SMOKE_DIR/scorecard.json" ]; then
  python3 - <<PY
import json
from pathlib import Path
sc=json.loads(Path("$SMOKE_DIR/scorecard.json").read_text())
m=sc.get("metrics") or {}
ops=sc.get("ops") or {}
pins=sc.get("pins") or {}
print("GATES:",
      "valid=", sc.get("valid"),
      "acc=", round(float(m.get("accuracy") or 0), 4),
      "f1=", round(float(m.get("f1") or 0), 4),
      "vision_pin=", pins.get("vision_model") or pins.get("profile"),
      "n_docs=", m.get("n_docs"),
      "ingest_coverage=", ops.get("ingest_coverage"),
)
ok = bool(sc.get("valid")) and float(ops.get("ingest_coverage") or 0) >= 0.9 and int(m.get("n_docs") or 0) >= 8
raise SystemExit(0 if ok and $RC == 0 else 2)
PY
else
  die "no scorecard written (rc=$RC)"
fi
