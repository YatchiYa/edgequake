#!/usr/bin/env bash
# SPEC-082 — Push-scale: A6 @150k/250k, Wave-2 filtered @150k, DiskANN full-gate @250k.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
A6_ROWS="${EQ_PUSH_A6_ROWS:-150000,250000}"
WAVE2_ROWS="${EQ_PUSH_WAVE2_ROWS:-150000}"
PUSH_DISKANN="${EQ_PUSH_DISKANN:-1}"

export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-8g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-4GB}"
export EQ_FDL_HANG_MS="${EQ_FDL_HANG_MS:-60000}"
export EQ_FILTERED_HANG_MS="${EQ_FILTERED_HANG_MS:-60000}"

ART_DIR="$ROOT/specs/082-push-scale-floors/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-push082.log"
REPORT="/tmp/eq-push082.jsonl"
: >"$LOG"
: >"$REPORT"
rm -f "$ART_DIR/PROMOTE_DISKANN_250K"

echo "NOTE: SPEC-082 push-scale-ladder a6=$A6_ROWS wave2=$WAVE2_ROWS diskann=$PUSH_DISKANN"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
EDGEQUAKE_DIR="$ROOT/edgequake"
RC_ALL=0

cleanup_eph() {
  if [[ -n "${EQ_EPHEMERAL_PG_CONTAINER:-}" ]]; then
    docker rm -f "$EQ_EPHEMERAL_PG_CONTAINER" >/dev/null 2>&1 || true
    unset EQ_EPHEMERAL_PG_CONTAINER
  fi
}
trap cleanup_eph EXIT

run_test() {
  local label="$1" profile="$2" test_name="$3"
  local runlog="/tmp/eq-push082-${label}.log"
  echo "NOTE: SPEC-082 ${label} profile=${profile}"
  cleanup_eph
  eq_ephemeral_pg_start "$profile" "edgequake-ps082${label}"
  eq_ephemeral_pg_migrate
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

# --- A6 Filtered-DiskANN labels @ higher N ---
IFS=',' read -r -a A6_ARR <<<"$A6_ROWS"
for rows in "${A6_ARR[@]}"; do
  rows="$(echo "$rows" | tr -d '[:space:]')"
  [[ -z "$rows" ]] && continue
  export EQ_FDL_ROWS="$rows"
  run_test "a6-${rows}" "pg18-vectorscale" "e2e_spec078_filtered_diskann_labels_bakeoff"
done

# --- Wave-2 filtered spot @150k (DIM=1536 product shape; soft floors) ---
if [[ -n "$WAVE2_ROWS" && "$WAVE2_ROWS" != "0" ]]; then
  export EQ_FILTERED_RECALL_ROWS="$WAVE2_ROWS"
  export EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1
  export EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS="${EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS:-1000}"
  run_test "wave2-${WAVE2_ROWS}" "pg18" "e2e_spec075_filtered_recall_gate"
fi

# --- DiskANN primary full-gate @250k (promote candidate) ---
if [[ "$PUSH_DISKANN" == "1" ]]; then
  export EQ_PARETO_ROWS=250000
  export EQ_PARETO_SPOT_ROWS=
  export EQ_PARETO_SEARCH_LIST="${EQ_PARETO_SEARCH_LIST:-400,800}"
  export EQ_PARETO_REF_SEARCH_LIST="${EQ_PARETO_REF_SEARCH_LIST:-1600}"
  export EQ_PARETO_REBUILD="${EQ_PARETO_REBUILD:-1}"
  unset EQ_DISKANN_SMOKE || true
  run_test "diskann-250k" "pg18-vectorscale" "e2e_spec072_diskann_recall_pareto"
fi

cp "$REPORT" "$ART_DIR/eq-push-scale082.jsonl"
cp "$LOG" "$ART_DIR/eq-push-scale082-cargo.log"

python3 - <<'PY' "$REPORT" "$ART_DIR/RUN_NOTES.md" "$RC_ALL" "$ART_DIR"
import json, re, sys
from pathlib import Path
from datetime import datetime, timezone

report, notes, rc, art = Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3]), Path(sys.argv[4])
rows = []
for line in report.read_text().splitlines():
    try:
        rows.append(json.loads(line))
    except Exception:
        pass

a6 = [o for o in rows if o.get("op") == "fdl078_filtered_recall" and "labels" in str(o.get("plan_class", ""))]
wave2 = [o for o in rows if o.get("op") in ("fr075_filtered_recall", "fr075_cell")]
decisions = [o for o in rows if o.get("op") == "pareto_decision"]

diskann_promote = False
highest = 150000
detail_d = "DiskANN floor unchanged (150k)"
for o in decisions:
    d = str(o.get("detail", ""))
    if o.get("pass") and "green_250k=true" in d:
        diskann_promote = True
        highest = 250000
        detail_d = "DiskANN opt-in highest_green_N→250000 (full-gate @250k)"
    m = re.search(r"highest_green_N=(\d+)", d)
    if m and diskann_promote:
        highest = max(highest, int(m.group(1)))

a6_pass = all(o.get("pass") for o in a6) if a6 else False
a6_detail = "; ".join(str(o.get("detail", ""))[:90] for o in a6) if a6 else "no A6 cells"

decision = "Not promoted (Wave-2 floor unchanged; DiskANN opt-in floor unchanged)"
if diskann_promote:
    decision = (
        "DiskANN opt-in floor raised to 250k; Wave-2 default unchanged; "
        "A6 tip not default; silent flip forbidden"
    )
    (art / "PROMOTE_DISKANN_250K").write_text("1\n")

lines = [
    "# SPEC-082 RUN_NOTES — Push-scale floors",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Promote metric: filtered recall (A6/Wave-2); DiskANN dedicated full-gate",
    "- Silent flip: forbidden",
    f"- Cargo aggregate exit: {rc}",
    f"- **Decision: {decision}**",
    f"- DiskANN: {detail_d} (highest_green_N candidate={highest})",
    f"- A6 labels soft-pass={a6_pass}: {a6_detail[:220]}",
    f"- Wave-2 @150k spot cells: {len(wave2)} (default floor stays 100k unless separate full-gate)",
    "",
    "## Cells",
    "",
    "| op | plan_class | pass | detail |",
    "|----|------------|------|--------|",
]
interesting = {
    "fdl078_filtered_recall",
    "fdl078_cell",
    "fdl078_decision",
    "fr075_filtered_recall",
    "fr075_cell",
    "fr075_decision",
    "pareto_decision",
    "pareto_spot",
    "pareto_rebuild",
    "pareto_cell",
}
for o in rows:
    op = o.get("op", "")
    if op in interesting or "filtered" in op:
        lines.append(
            f"| `{op}` | `{o.get('plan_class')}` | {o.get('pass')} | `{str(o.get('detail',''))[:110]}` |"
        )

lines += [
    "",
    "- SSOT: apply DiskANN 250k floor only when `PROMOTE_DISKANN_250K` exists after a green full-gate",
    "",
]
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if rc == 0 else 1)
PY
