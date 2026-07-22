#!/usr/bin/env bash
# SPEC-080 — Tiny-slice exact gate (contracts + optional DB smoke).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ART_DIR="$ROOT/specs/080-tiny-slice-exact/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-tiny-slice080.log"
: >"$LOG"

EDGEQUAKE_DIR="$ROOT/edgequake"
cd "$EDGEQUAKE_DIR"

echo "NOTE: SPEC-080 tiny-slice-exact-gate"

set +e
cargo test -p edgequake-storage --features postgres --test contract_spec080_tiny_slice_exact -- --nocapture 2>&1 | tee -a "$LOG"
RC_CONTRACT=${PIPESTATUS[0]}
cargo test -p edgequake-storage --features postgres --lib filter_column_policy -- --nocapture 2>&1 | tee -a "$LOG"
RC_LIB=${PIPESTATUS[0]}
# Unit tests for wave2 bias live in search_tuning
cargo test -p edgequake-storage --features postgres --lib test_wave2_planner_bias -- --nocapture 2>&1 | tee -a "$LOG"
RC_BIAS=${PIPESTATUS[0]}
set -e

SMOKE_NOTE="skipped (contracts cover bias skip; EQ_TINY_SLICE_SMOKE=1 for DB)"
RC_SMOKE=0
if [[ "${EQ_TINY_SLICE_SMOKE:-0}" == "1" ]]; then
  # shellcheck source=/dev/null
  source "$ROOT/scripts/eq_ephemeral_pg.sh"
  eq_ephemeral_pg_start "${EQ_PERF_PROFILES:-pg18}" "edgequake-ts080"
  eq_ephemeral_pg_migrate
  set +e
  cargo test -p edgequake-storage --features postgres --release \
    --test e2e_spec080_tiny_slice_exact_smoke -- --nocapture 2>&1 | tee -a "$LOG"
  RC_SMOKE=${PIPESTATUS[0]}
  set -e
  SMOKE_NOTE="ran exit=${RC_SMOKE}"
fi

cp "$LOG" "$ART_DIR/eq-tiny-slice-exact-cargo.log"

python3 - <<'PY' "$ART_DIR/RUN_NOTES.md" "$RC_CONTRACT" "$RC_LIB" "$RC_BIAS" "$RC_SMOKE" "$SMOKE_NOTE"
import sys
from pathlib import Path
from datetime import datetime, timezone
notes = Path(sys.argv[1])
rc_c, rc_lib, rc_bias, rc_smoke = map(int, sys.argv[2:6])
smoke = sys.argv[6]
ok = rc_c == 0 and rc_lib == 0 and rc_bias == 0 and rc_smoke == 0
lines = [
    "# SPEC-080 RUN_NOTES — Tiny-slice exact",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Env: `EDGEQUAKE_ANN_EXACT_MAX_ROWS` (default 2000)",
    "- Wave-2 planner bias skipped when workspace rows ≤ threshold",
    "- Floors unchanged; no silent flip of Wave-2 defaults",
    f"- Contract exit: {rc_c}",
    f"- Lib/filter exit: {rc_lib}",
    f"- Bias unit exit: {rc_bias}",
    f"- DB smoke: {smoke}",
    "",
    "## Gate: " + ("GREEN" if ok else "RED"),
    "",
]
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if ok else 1)
PY
