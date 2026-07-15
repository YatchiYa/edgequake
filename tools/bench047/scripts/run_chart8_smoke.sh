#!/usr/bin/env bash
# SPEC-047 chart-8 smoke — locked Acc physics (first principles).
# Fixture: smoke_chart_doc_ids_v1.txt (8 docs / 117 Qs)
# Profile: P0_mm_ite · hybrid · document-scope · mistral-small-latest · mistral-embed
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$REPO_ROOT"

ART_ROOT="specs/047-rag-evaluation/e2e/artifacts"
SMOKE_DIR="$ART_ROOT/smoke"
RUN_TAG="${BENCH047_RUN_TAG:-chart8-$(date -u +%Y%m%d-%H%M%S)}"
SNAP_DIR="$ART_ROOT/smoke-${RUN_TAG}"
API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:8090}"
WORKERS="${BENCH047_WORKERS:-2}"
INGEST_WORKERS="${BENCH047_INGEST_WORKERS:-4}"
ENSURE="$REPO_ROOT/tools/bench047/scripts/ensure_backend_small.sh"

export EDGEQUAKE_API_URL="$API_URL"
export EDGEQUAKE_BENCH_FIXTURE=smoke_chart_doc_ids_v1.txt
export MISTRAL_MODEL=mistral-small-latest
export EDGEQUAKE_LLM_PROVIDER=mistral
export EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_VISION_PROVIDER=mistral
export EDGEQUAKE_VISION_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral
export MISTRAL_EMBEDDING_MODEL=mistral-embed
export VLM_PROCESS_ENABLE=true
export BENCH047_WORKERS="$WORKERS"
export BENCH047_INGEST_WORKERS="$INGEST_WORKERS"

die() { echo "ERROR: $*" >&2; exit 1; }

echo "=== SPEC-047 chart-8 smoke ($RUN_TAG) ==="

# 0) Fail-closed stuck PDFs (steal workers / Mistral quota → flaky ingest)
echo "=== fail-closed stuck pdf_documents.processing ==="
docker exec edgequake-postgres psql -U edgequake -d edgequake -v ON_ERROR_STOP=1 -c "
UPDATE pdf_documents
SET processing_status = 'failed',
    extraction_errors = coalesce(extraction_errors, '[]'::jsonb)
      || jsonb_build_array(jsonb_build_object(
           'at', now(),
           'reason', 'bench047_pre_smoke_fail_closed_stale_processing'
         )),
    updated_at = now()
WHERE processing_status IN ('pending', 'processing')
  AND updated_at < now() - interval '30 minutes';
SELECT processing_status, count(*) FROM pdf_documents GROUP BY 1 ORDER BY 1;
" || die "could not fail-closed stale PDFs"

# 1) Backend watchdog on Small
chmod +x "$ENSURE"
"$ENSURE" start-watchdog
"$ENSURE" status || die "backend not Small-healthy"

# 2) Doctor (fail closed)
python3 -m bench047.cli doctor --api "$API_URL" --profile P0_mm_ite || die "doctor FAIL"

# 3) Archive previous smoke/ if present
mkdir -p "$SMOKE_DIR"
if [ -f "$SMOKE_DIR/SUMMARY.md" ] || [ -f "$SMOKE_DIR/scorecard.json" ]; then
  PRE="$ART_ROOT/smoke-pre-${RUN_TAG}"
  mkdir -p "$PRE"
  cp -a "$SMOKE_DIR"/. "$PRE"/ || true
  echo "archived prior smoke → $PRE"
fi

# Fresh ledgers (--no-resume = force_reindex; avoid duplicate resume contamination)
rm -f "$SMOKE_DIR"/ingest.jsonl "$SMOKE_DIR"/predictions.jsonl \
  "$SMOKE_DIR"/scorecard.json "$SMOKE_DIR"/SUMMARY.md "$SMOKE_DIR"/meta.json \
  "$SMOKE_DIR"/fidelity.json "$SMOKE_DIR"/FIDELITY.md
mkdir -p "$SMOKE_DIR/logs"

# 4) Progress monitor (ingest.jsonl growth)
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

# 5) Run smoke (locked physics + ite multimodal)
echo "=== running chart-8 smoke query_workers=$WORKERS ingest_workers=$INGEST_WORKERS ==="
set +e
python3 -m bench047.cli smoke \
  --api "$API_URL" \
  --profile P0_mm_ite \
  --no-resume \
  --document-scope \
  --workers "$WORKERS" \
  --ingest-workers "$INGEST_WORKERS"
RC=$?
set -e

kill "$MON_PID" 2>/dev/null || true

# 6) Append July-2026 SOTA comparison (official HF LVLM board — different task)
if [ -f "$SMOKE_DIR/scorecard.json" ]; then
  python3 - <<'PY'
import json
from pathlib import Path
smoke = Path("specs/047-rag-evaluation/e2e/artifacts/smoke")
sc = json.loads((smoke / "scorecard.json").read_text())
m = sc.get("metrics") or {}
ops = sc.get("ops") or {}
slices = sc.get("slices") or {}
src = slices.get("by_evidence_source") or {}
acc = float(m.get("accuracy") or 0)
f1 = float(m.get("f1") or 0)
chart = (src.get("Chart") or {}).get("accuracy")
# Official OpenIXCLab MMLongBench-Doc LVLM leaderboard (full ~1082 Q, page-screenshot task)
# Fetched 2026-07: https://huggingface.co/datasets/OpenIXCLab/mmlongbench-doc-results
sota = [
    ("TeleMM2.0 (2026-01-05) — official HF SOTA", 0.5609, 0.5590, 0.5416),
    ("GPT-4.1 (2025-04-14)", 0.4974, 0.5142, 0.4847),
    ("GPT-4o (2024-11-20, refreshed board)", 0.4625, 0.4624, 0.4315),
    ("Paper GPT-4o (NeurIPS'24 original report)", None, 0.449, None),
]
lines = [
    "",
    "## vs LVLM SOTA (July 2026 reference) — READ CAVEATS",
    "",
    "**Task identity:** this EdgeQuake run is a **RAG adaptation** on the chart-8 smoke fixture",
    f"({m.get('n_docs')} docs / {m.get('n_scored')} Qs, hybrid retrieve + Small LLM).",
    "Official MMLongBench-Doc leaderboard scores are **page-screenshot LVLMs on ~1082 questions**.",
    "Numbers are **difficulty references**, not a same-protocol ranking.",
    "",
    f"| System | Acc | F1 | Chart Acc | Protocol |",
    f"|--------|-----|----|-----------|----------|",
    f"| **EdgeQuake P0_mm_ite (this run)** | **{acc:.4f}** | **{f1:.4f}** | "
    f"**{(chart if chart is not None else float('nan')):.4f}** | RAG · 8-doc smoke · dscope · ite |",
]
for name, a, f, c in sota:
    a_s = f"{a:.4f}" if a is not None else "—"
    f_s = f"{f:.4f}" if f is not None else "—"
    c_s = f"{c:.4f}" if c is not None else "—"
    lines.append(f"| {name} | {a_s} | {f_s} | {c_s} | Full LVLM board |")
lines += [
    "",
    "Sources: [OpenIXCLab/mmlongbench-doc-results](https://huggingface.co/datasets/OpenIXCLab/mmlongbench-doc-results)",
    "(official). Aggregators may list higher single scores (e.g. Qwen / Nemotron ~57–62%) under",
    "third-party protocols — prefer the official Acc/F1 board for citation.",
    "",
    f"- ΔAcc vs TeleMM2.0 (SOTA Acc): **{acc - 0.5609:+.4f}** (not same task)",
    f"- ΔF1 vs TeleMM2.0 (SOTA F1): **{f1 - 0.5590:+.4f}** (not same task)",
    f"- ΔF1 vs paper GPT-4o (0.449): **{f1 - 0.449:+.4f}** (difficulty ref only)",
    f"- Ops: ingest_coverage={ops.get('ingest_coverage')} page_hit@5="
    f"{(ops.get('retrieval') or {}).get('page_hit@5')} empty={ops.get('answer_empty_rate')}",
    "",
]
summary = smoke / "SUMMARY.md"
if summary.exists():
    text = summary.read_text()
    marker = "## vs LVLM SOTA (July 2026 reference)"
    if marker in text:
        text = text.split(marker)[0].rstrip() + "\n"
    # Insert before Citation if present
    cite = "## Citation"
    block = "\n".join(lines)
    if cite in text:
        text = text.replace(cite, block + cite)
    else:
        text = text.rstrip() + "\n" + block
    summary.write_text(text)
    print("updated SUMMARY with July-2026 SOTA comparison")
PY
fi

# 7) Snapshot artifacts (never leave only overwritten smoke/)
mkdir -p "$SNAP_DIR"
cp -a "$SMOKE_DIR"/. "$SNAP_DIR"/
echo "snapshot → $SNAP_DIR"

# 8) Gate summary
if [ -f "$SMOKE_DIR/SUMMARY.md" ]; then
  head -50 "$SMOKE_DIR/SUMMARY.md"
fi
if [ -f "$SMOKE_DIR/scorecard.json" ]; then
  python3 - <<PY
import json
from pathlib import Path
sc=json.loads(Path("$SMOKE_DIR/scorecard.json").read_text())
m=sc.get("metrics") or {}
ops=sc.get("ops") or {}
print("GATES:",
      "valid=", sc.get("valid"),
      "acc=", round(float(m.get("accuracy") or 0), 4),
      "f1=", round(float(m.get("f1") or 0), 4),
      "n_docs=", m.get("n_docs"),
      "n_scored=", m.get("n_scored"),
      "ingest_coverage=", ops.get("ingest_coverage"),
      "invalid_reason=", sc.get("invalid_reason"),
)
ok = bool(sc.get("valid")) and float(ops.get("ingest_coverage") or 0) >= 0.9 and int(m.get("n_docs") or 0) >= 8
raise SystemExit(0 if ok and $RC == 0 else 2)
PY
else
  die "no scorecard written (rc=$RC)"
fi
