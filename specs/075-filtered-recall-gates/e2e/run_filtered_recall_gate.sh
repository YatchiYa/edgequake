#!/usr/bin/env bash
# SPEC-075 — Filtered recall@20 claim gate (Wave-2 smoke) + iterative_scan-only compare.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18}"

export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1
export EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS="${EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS:-100}"
export EDGEQUAKE_HNSW_ITERATIVE_SCAN="${EDGEQUAKE_HNSW_ITERATIVE_SCAN:-relaxed_order}"
export EDGEQUAKE_HNSW_MAX_SCAN_TUPLES="${EDGEQUAKE_HNSW_MAX_SCAN_TUPLES:-20000}"
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-4g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-2GB}"
export EQ_FILTERED_RECALL_ROWS="${EQ_FILTERED_RECALL_ROWS:-5000}"

ART_DIR="$ROOT/specs/075-filtered-recall-gates/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-filtered-recall075-${PROFILE}.log"
REPORT="/tmp/eq-filtered-recall075-${PROFILE}.jsonl"

echo "NOTE: SPEC-075 filtered-recall-gate profile=$PROFILE rows=$EQ_FILTERED_RECALL_ROWS"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-fr075"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec075_filtered_recall_gate -- --nocapture 2>&1 | tee "$LOG"
RC=${PIPESTATUS[0]}
set -e

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true

if grep -E 'SKIP:.*(DATABASE_URL|POSTGRES_PASSWORD)' "$LOG" >/dev/null 2>&1; then
  echo "ERROR: DATABASE soft-skip under REQUIRE_POSTGRES"
  exit 1
fi

cp "$REPORT" "$ART_DIR/eq-filtered-recall-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-filtered-recall-${PROFILE}-cargo.log"

python3 - <<'PY' "$REPORT" "$ART_DIR/RUN_NOTES.md" "$RC"
import json, sys
from pathlib import Path
from datetime import datetime, timezone
report, notes, rc = Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3])
rows = []
for line in report.read_text().splitlines():
    try:
        rows.append(json.loads(line))
    except Exception:
        pass
cells = [o for o in rows if o.get("op") == "fr075_cell"]
recalls = [o for o in rows if o.get("op") == "fr075_filtered_recall"]
lines = [
    "# SPEC-075 RUN_NOTES — Filtered recall gate",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Promote metric: **filtered** recall@20 (workspace filter) — never unfiltered-only",
    "- Wave-2 default unchanged; floors unchanged",
    f"- Smoke cargo exit: {rc}",
    f"- Cells: {len(cells)} · filtered_recall reports: {len(recalls)}",
    "",
    "## Cells",
    "",
    "| arm | pass | detail |",
    "|-----|------|--------|",
]
for o in cells:
    lines.append(f"| `{o.get('plan_class')}` | {o.get('pass')} | `{str(o.get('detail',''))[:140]}` |")
lines += [
    "",
    "## iterative_scan bounds",
    "",
    "- Filtered: `SET LOCAL hnsw.iterative_scan` + `max_scan_tuples` (contract_spec075)",
    "- Unfiltered: iterative_scan **off**",
    "- Env: `EDGEQUAKE_HNSW_ITERATIVE_SCAN`, `EDGEQUAKE_HNSW_MAX_SCAN_TUPLES`, `EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER`",
    "",
    "## 100k evidence",
    "",
    "See [SPEC-068 RUN_NOTES](../../../068-recall-quality-scale/e2e/artifacts/RUN_NOTES.md) — mid-scale wall; Wave-2 `highest_green_N=100000`.",
]
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if rc == 0 else rc)
PY
