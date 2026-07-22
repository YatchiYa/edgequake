#!/usr/bin/env bash
# SPEC-068 — Recall × latency Pareto on pg18 (Wave-2 + planner bias).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18}"

export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1
export EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS="${EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS:-1000}"
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-8g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-4GB}"
export EQ_PARETO_ROWS_LIST="${EQ_PARETO_ROWS_LIST:-100000,150000,200000,250000}"
export EQ_PARETO_EF_LIST="${EQ_PARETO_EF_LIST:-80,160,240,400}"
# Rebuild arm at max N (m=32, ef_c=128) — evidence if query-ef cannot green mid-scale
export EQ_PARETO_REBUILD="${EQ_PARETO_REBUILD:-1}"

ART_DIR="$ROOT/specs/068-recall-quality-scale/e2e/artifacts"
mkdir -p "$ART_DIR"
LABEL="PARETO"
LOG="/tmp/eq-recall-pareto-${PROFILE}.log"
REPORT="/tmp/eq-recall-pareto-${PROFILE}.jsonl"

echo "NOTE: SPEC-068 recall Pareto rows=$EQ_PARETO_ROWS_LIST ef=$EQ_PARETO_EF_LIST rebuild=$EQ_PARETO_REBUILD"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-pareto"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec068_recall_pareto -- --nocapture 2>&1 | tee "$LOG"
RC=${PIPESTATUS[0]}
set -e

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true

if grep -E 'SKIP:.*(DATABASE_URL|POSTGRES_PASSWORD)' "$LOG" >/dev/null 2>&1; then
  echo "ERROR: DATABASE soft-skip under REQUIRE_POSTGRES"
  exit 1
fi

cp "$REPORT" "$ART_DIR/eq-recall-pareto-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-recall-pareto-${PROFILE}-cargo.log"

python3 - <<'PY' "$REPORT" "$ART_DIR/PARETO_SUMMARY.md"
import json, sys
from pathlib import Path
report, out = Path(sys.argv[1]), Path(sys.argv[2])
rows = []
for line in report.read_text().splitlines():
    try:
        rows.append(json.loads(line))
    except Exception:
        pass
greens = []
cells = []
for o in rows:
    if o.get("op") != "pareto_stress":
        continue
    d = o.get("detail", "")
    cells.append((d, o.get("pass"), o.get("p95_ms")))
    if o.get("pass"):
        greens.append(d)
lines = ["# SPEC-068 recall × latency Pareto summary", ""]
lines.append(f"- stress cells: {len(cells)}")
lines.append(f"- full_green cells: {len(greens)}")
lines.append("")
lines.append("| detail | pass | stress_p95_ms |")
lines.append("|--------|------|---------------|")
for d, p, p95 in cells:
    lines.append(f"| `{d[:120]}` | {p} | {p95} |")
lines.append("")
if greens:
    lines.append("## Green cells (promote candidates)")
    for g in greens:
        lines.append(f"- `{g}`")
else:
    lines.append("## No full-gate green above baseline — honest wall (do not invent floors)")
out.write_text("\n".join(lines) + "\n")
print(out.read_text())
PY

if [[ "$RC" -ne 0 ]]; then
  echo "WARN recall-pareto finished with test_rc=$RC (artifacts archived)"
  exit 1
fi
echo "OK recall-pareto -> $ART_DIR/eq-recall-pareto-${PROFILE}.jsonl"
