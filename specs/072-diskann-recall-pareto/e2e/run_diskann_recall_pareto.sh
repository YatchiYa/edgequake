#!/usr/bin/env bash
# SPEC-072 — DiskANN recall×latency Pareto @150k (claim gate, not day-2 sizing).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18-vectorscale}"

export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-8g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-4GB}"
if [[ "${EQ_DISKANN_SMOKE:-0}" == "1" ]]; then
  export EQ_PARETO_ROWS=2000
  export EQ_PARETO_SPOT_ROWS=
  export EQ_PARETO_REBUILD=0
else
  export EQ_PARETO_ROWS="${EQ_PARETO_ROWS:-150000}"
  export EQ_PARETO_SPOT_ROWS="${EQ_PARETO_SPOT_ROWS:-100000,250000}"
  export EQ_PARETO_REBUILD="${EQ_PARETO_REBUILD:-1}"
fi
export EQ_PARETO_SEARCH_LIST="${EQ_PARETO_SEARCH_LIST:-100,200,400,800}"
export EQ_PARETO_REF_SEARCH_LIST="${EQ_PARETO_REF_SEARCH_LIST:-1600}"

ART_DIR="$ROOT/specs/072-diskann-recall-pareto/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-diskann-pareto-${PROFILE}.log"
REPORT="/tmp/eq-diskann-pareto-${PROFILE}.jsonl"

echo "NOTE: SPEC-072 DiskANN recall Pareto profile=$PROFILE rows=$EQ_PARETO_ROWS spot=$EQ_PARETO_SPOT_ROWS smoke=${EQ_DISKANN_SMOKE:-0}"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-pareto072"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec072_diskann_recall_pareto -- --nocapture 2>&1 | tee "$LOG"
RC=${PIPESTATUS[0]}
set -e

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true

if grep -E 'SKIP:.*(DATABASE_URL|POSTGRES_PASSWORD)' "$LOG" >/dev/null 2>&1; then
  echo "ERROR: DATABASE soft-skip under REQUIRE_POSTGRES"
  exit 1
fi

cp "$REPORT" "$ART_DIR/eq-diskann-pareto-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-diskann-pareto-${PROFILE}-cargo.log"

python3 - <<'PY' "$REPORT" "$ART_DIR/PARETO_SUMMARY.md" "$ART_DIR/RUN_NOTES.md"
import json, sys
from pathlib import Path
from datetime import datetime, timezone
report, summary, notes = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
rows = []
for line in report.read_text().splitlines():
    try:
        rows.append(json.loads(line))
    except Exception:
        pass
decision = next((o for o in rows if o.get("op") == "pareto_decision"), None)
cells = [o for o in rows if o.get("op") == "pareto_cell"]
promote = bool(decision and decision.get("pass"))
greens = [o for o in cells if o.get("pass")]
lines = ["# SPEC-072 DiskANN recall Pareto summary", ""]
lines.append(f"- cells: {len(cells)}")
lines.append(f"- full_green cells: {len(greens)}")
if decision:
    lines.append(f"- decision: `{decision.get('detail')}` pass={decision.get('pass')}")
lines.append("")
lines.append("| detail | pass | p95_ms |")
lines.append("|--------|------|--------|")
for o in cells:
    d = str(o.get("detail", ""))[:180]
    lines.append(f"| `{d}` | {o.get('pass')} | {o.get('p95_ms')} |")
summary.write_text("\n".join(lines) + "\n")
print(summary.read_text())

note_lines = [
    "# SPEC-072 RUN_NOTES — DiskANN recall Pareto @150k",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    f"- Profile: pg18-vectorscale (pgvectorscale 0.9.0)",
    f"- Primary: dedicated DiskANN @150k; query_search_list_size grid + rebuild arm",
    f"- Recall ref: high DiskANN query_search_list_size (see decision detail)",
    f"- Promote SSOT: **{'YES' if promote else 'NO'}** (full gate @150k clients=16)",
    "",
    "## Decision",
    "",
]
if decision:
    note_lines.append(f"- `{decision.get('detail')}`")
else:
    note_lines.append("- (no pareto_decision — see cargo log)")
note_lines += [
    "",
    "## Honesty",
    "",
    "- Wave-2 shared+partial remains default 100k unless promote=YES.",
    "- No silent DiskANN default.",
    "- Soft-fail product gates; hang cliff hard-fails.",
    "",
    "Artifacts: `eq-diskann-pareto-*.jsonl`, `PARETO_SUMMARY.md`.",
    "",
]
notes.write_text("\n".join(note_lines))
print(notes.read_text())
PY

if [[ "$RC" -ne 0 ]]; then
  echo "WARN diskann-recall-pareto finished with test_rc=$RC (artifacts archived)"
  exit 1
fi
echo "OK diskann-recall-pareto -> $ART_DIR/eq-diskann-pareto-${PROFILE}.jsonl"
