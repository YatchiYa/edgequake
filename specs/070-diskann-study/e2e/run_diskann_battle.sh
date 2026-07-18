#!/usr/bin/env bash
# SPEC-070 — DiskANN vs HNSW dedicated battle (claim gate, not day-2 sizing).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18-vectorscale}"

export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-8g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-4GB}"
export EQ_DISKANN_ROWS_LIST="${EQ_DISKANN_ROWS_LIST:-100000,150000,250000}"
# EQ_DISKANN_SMOKE=1 → tiny corpus for extension smoke only

ART_DIR="$ROOT/specs/070-diskann-study/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-diskann-battle-${PROFILE}.log"
REPORT="/tmp/eq-diskann-battle-${PROFILE}.jsonl"

echo "NOTE: SPEC-070 DiskANN battle profile=$PROFILE rows=$EQ_DISKANN_ROWS_LIST smoke=${EQ_DISKANN_SMOKE:-0}"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-disk070"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec070_diskann_battle -- --nocapture 2>&1 | tee "$LOG"
RC=${PIPESTATUS[0]}
set -e

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true

if grep -E 'SKIP:.*(DATABASE_URL|POSTGRES_PASSWORD)' "$LOG" >/dev/null 2>&1; then
  echo "ERROR: DATABASE soft-skip under REQUIRE_POSTGRES"
  exit 1
fi

cp "$REPORT" "$ART_DIR/eq-diskann-battle-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-diskann-battle-${PROFILE}-cargo.log"

python3 - <<'PY' "$REPORT" "$ART_DIR/DISKANN_SUMMARY.md" "$ART_DIR/RUN_NOTES.md"
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
decision = next((o for o in rows if o.get("op") == "diskann_decision"), None)
arms = [o for o in rows if o.get("op") == "diskann_arm_summary"]
stress = [o for o in rows if o.get("op") == "diskann_stress"]
promote = bool(decision and decision.get("pass"))
lines = ["# SPEC-070 DiskANN battle summary", ""]
lines.append(f"- arm summaries: {len(arms)}")
lines.append(f"- stress cells: {len(stress)}")
if decision:
    lines.append(f"- decision: `{decision.get('detail')}` pass={decision.get('pass')}")
lines.append("")
lines.append("| detail | pass | p95_ms | plan_class |")
lines.append("|--------|------|--------|------------|")
for o in arms:
    d = str(o.get("detail", ""))[:160]
    lines.append(f"| `{d}` | {o.get('pass')} | {o.get('p95_ms')} | {o.get('plan_class')} |")
summary.write_text("\n".join(lines) + "\n")
print(summary.read_text())

note_lines = [
    "# SPEC-070 RUN_NOTES — DiskANN / pgvectorscale study",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    f"- Profile: pg18-vectorscale (pgvectorscale 0.9.0 / StreamingDiskANN)",
    f"- Primary shape: dedicated `*_ws_*` + `USING diskann` vs HNSW halfvec control",
    f"- Promote SSOT: **{'YES' if promote else 'NO'}** (full gate only @150k clients=16)",
    "",
    "## Decision",
    "",
]
if decision:
    note_lines.append(f"- `{decision.get('detail')}`")
else:
    note_lines.append("- (no diskann_decision row — see cargo log)")
note_lines += [
    "",
    "## Honesty",
    "",
    "- Wave-2 shared+partial remains supported **100k** path unless promote=YES.",
    "- No silent DiskANN default. Opt-in recipe only if full-gate green.",
    "- Soft-fail product gates in harness; hang cliff hard-fails.",
    "",
    "Artifacts: `eq-diskann-battle-*.jsonl`, `DISKANN_SUMMARY.md`.",
    "",
]
notes.write_text("\n".join(note_lines))
print(notes.read_text())
PY

if [[ "$RC" -ne 0 ]]; then
  echo "WARN diskann-battle finished with test_rc=$RC (artifacts archived)"
  exit 1
fi
echo "OK diskann-battle -> $ART_DIR/eq-diskann-battle-${PROFILE}.jsonl"
