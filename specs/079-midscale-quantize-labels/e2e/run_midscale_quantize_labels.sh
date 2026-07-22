#!/usr/bin/env bash
# SPEC-079 — Mid-scale B2 (binary quantize) + A6 (Filtered-DiskANN labels) archive.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ROWS_LIST="${EQ_MIDSCALE_ROWS:-50000,100000}"
export EQ_BQ_HANG_MS="${EQ_BQ_HANG_MS:-30000}"
export EQ_FDL_HANG_MS="${EQ_FDL_HANG_MS:-30000}"
export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-4g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-2GB}"

ART_DIR="$ROOT/specs/079-midscale-quantize-labels/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-midscale079.log"
REPORT="/tmp/eq-midscale079.jsonl"
: >"$LOG"
: >"$REPORT"

echo "NOTE: SPEC-079 midscale-quantize-labels rows=$ROWS_LIST hang_bq=$EQ_BQ_HANG_MS hang_fdl=$EQ_FDL_HANG_MS"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
EDGEQUAKE_DIR="$ROOT/edgequake"

RC_ALL=0
IFS=',' read -r -a ROWS_ARR <<<"$ROWS_LIST"

cleanup_eph() {
  if [[ -n "${EQ_EPHEMERAL_PG_CONTAINER:-}" ]]; then
    docker rm -f "$EQ_EPHEMERAL_PG_CONTAINER" >/dev/null 2>&1 || true
    unset EQ_EPHEMERAL_PG_CONTAINER
  fi
}
trap cleanup_eph EXIT

run_arm() {
  local kind="$1" rows="$2" profile="$3" test_name="$4"
  local runlog="/tmp/eq-midscale079-${kind}-${rows}.log"
  echo "NOTE: SPEC-079 ${kind} rows=${rows} profile=${profile}"
  cleanup_eph
  eq_ephemeral_pg_start "$profile" "edgequake-ms079${kind}"
  eq_ephemeral_pg_migrate
  if [[ "$kind" == "bq" ]]; then
    export EQ_BQ_ROWS="$rows"
  else
    export EQ_FDL_ROWS="$rows"
  fi
  cd "$EDGEQUAKE_DIR"
  set +e
  cargo test -p edgequake-storage --features postgres --release \
    --test "$test_name" -- --nocapture 2>&1 | tee "$runlog" | tee -a "$LOG"
  local rc=${PIPESTATUS[0]}
  set -e
  if [[ "$rc" -ne 0 ]]; then RC_ALL=1; fi
  grep -E '^PERF_REPORT ' "$runlog" | sed 's/^PERF_REPORT //' >>"$REPORT" || true
  cleanup_eph
}

for rows in "${ROWS_ARR[@]}"; do
  rows="$(echo "$rows" | tr -d '[:space:]')"
  [[ -z "$rows" ]] && continue
  run_arm "bq" "$rows" "pg18" "e2e_spec077_binary_quantize_bakeoff"
  run_arm "fdl" "$rows" "pg18-vectorscale" "e2e_spec078_filtered_diskann_labels_bakeoff"
done

cp "$REPORT" "$ART_DIR/eq-midscale079.jsonl"
cp "$LOG" "$ART_DIR/eq-midscale079-cargo.log"

python3 - <<'PY' "$REPORT" "$ART_DIR/RUN_NOTES.md" "$RC_ALL"
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
recalls = [o for o in rows if "filtered_recall" in str(o.get("op", ""))]
passes = [o for o in recalls if o.get("pass") is True]
fails = [o for o in recalls if o.get("pass") is False]
decision = "Not promoted"
detail = "tip remains study-only; Wave-2 default; no silent flip"
if rc == 0 and not fails and passes:
    has_100k = any("rows=100000" in str(o.get("detail", "")) for o in passes)
    if has_100k:
        decision = "promote candidate (archive only — no SSOT floor raise / no silent flip)"
        detail = "filtered recall soft-green @ mid-scale; still requires full concurrent gate before floor change"

lines = [
    "# SPEC-079 RUN_NOTES — Mid-scale B2 + A6",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Promote metric: **filtered** recall@20 vs Wave-2",
    "- Wave-2 remains product default; B2/A6 remain opt-in study (default OFF)",
    "- Floors unchanged unless explicit full-gate promote (not this pack)",
    "- Silent flip: forbidden",
    f"- Cargo aggregate exit: {rc}",
    f"- Filtered recall reports: {len(recalls)} (pass={len(passes)} fail={len(fails)})",
    f"- **Decision: {decision}**",
    f"- Detail: {detail}",
    "",
    "## Cells",
    "",
    "| op | plan_class | pass | detail |",
    "|----|------------|------|--------|",
]
for o in rows:
    if o.get("op") in (
        "bq077_filtered_recall",
        "bq077_cell",
        "bq077_decision",
        "fdl078_filtered_recall",
        "fdl078_cell",
        "fdl078_decision",
    ):
        lines.append(
            f"| `{o.get('op')}` | `{o.get('plan_class')}` | {o.get('pass')} | `{str(o.get('detail',''))[:120]}` |"
        )
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if rc == 0 else 1)
PY
