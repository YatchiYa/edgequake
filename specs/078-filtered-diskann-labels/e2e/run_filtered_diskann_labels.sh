#!/usr/bin/env bash
# SPEC-078 — Filtered-DiskANN labels vs Wave-2 / post-filter bake-off.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18-vectorscale}"

export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-4g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-2GB}"
export EQ_FDL_ROWS="${EQ_FDL_ROWS:-2000}"

ART_DIR="$ROOT/specs/078-filtered-diskann-labels/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-filtered-diskann078-${PROFILE}.log"
REPORT="/tmp/eq-filtered-diskann078-${PROFILE}.jsonl"

echo "NOTE: SPEC-078 filtered-diskann-labels-bakeoff profile=$PROFILE rows=$EQ_FDL_ROWS"

EDGEQUAKE_DIR="$ROOT/edgequake"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --test contract_spec078_filtered_diskann_labels -- --nocapture 2>&1 | tee "$LOG"
RC_CONTRACT=${PIPESTATUS[0]}
set -e

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-fdl078"
eq_ephemeral_pg_migrate

: >"$REPORT"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec078_filtered_diskann_labels_bakeoff -- --nocapture 2>&1 | tee -a "$LOG"
RC=${PIPESTATUS[0]}
set -e

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true
cp "$REPORT" "$ART_DIR/eq-filtered-diskann-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-filtered-diskann-${PROFILE}-cargo.log"

python3 - <<'PY' "$REPORT" "$ART_DIR/RUN_NOTES.md" "$RC" "$RC_CONTRACT"
import json, sys
from pathlib import Path
from datetime import datetime, timezone
report, notes, rc, rc_contract = Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
rows = []
for line in report.read_text().splitlines():
    try:
        rows.append(json.loads(line))
    except Exception:
        pass
cells = [o for o in rows if o.get("op") == "fdl078_cell"]
recalls = [o for o in rows if o.get("op") == "fdl078_filtered_recall"]
lines = [
    "# SPEC-078 RUN_NOTES — Filtered-DiskANN labels bake-off",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Promote metric: **filtered** recall@20 (labels vs Wave-2 reference)",
    "- Wave-2 remains product default; Filtered-DiskANN labels is **opt-in study**",
    "- Floors unchanged (no raise from smoke)",
    "- Silent flip: forbidden (`EDGEQUAKE_FILTERED_DISKANN_LABELS` default OFF; no product labels migration)",
    f"- Contract exit: {rc_contract}",
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
for o in recalls:
    lines.append(
        f"| `{o.get('plan_class')}` | {o.get('pass')} | `{str(o.get('detail',''))[:140]}` |"
    )
lines += [
    "",
    "## Helpers",
    "",
    "- `WorkspaceLabelMap` — dense workspace→`smallint` (fail closed at 32767)",
    "- `build_diskann_labels_index_sql` — `USING diskann (embedding …, labels)`",
    "- `build_filtered_diskann_label_select_sql` — `labels && ARRAY[$n]::smallint[]`",
    "- `build_postfilter_diskann_select_sql` — TEXT workspace honesty baseline",
    "- Env: `EDGEQUAKE_FILTERED_DISKANN_LABELS` (default off)",
    "",
    "## Decision",
    "",
    "Do **not** silent-flip product default from this smoke. Re-run at mid-scale + full gate before any promote.",
]
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if rc == 0 and rc_contract == 0 else 1)
PY
