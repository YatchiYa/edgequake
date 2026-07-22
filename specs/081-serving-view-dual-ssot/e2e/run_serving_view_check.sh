#!/usr/bin/env bash
# SPEC-081 — Serving view dual-SSOT check (contract + migrate presence).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ART_DIR="$ROOT/specs/081-serving-view-dual-ssot/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-serving081.log"
: >"$LOG"

EDGEQUAKE_DIR="$ROOT/edgequake"
cd "$EDGEQUAKE_DIR"

echo "NOTE: SPEC-081 serving-view-check"

set +e
cargo test -p edgequake-storage --test contract_spec081_serving_view -- --nocapture 2>&1 | tee -a "$LOG"
RC_CONTRACT=${PIPESTATUS[0]}
set -e

DB_NOTE="skipped (set EQ_SERVING_VIEW_SMOKE=1 for ephemeral migrate+probe)"
RC_DB=0
if [[ "${EQ_SERVING_VIEW_SMOKE:-0}" == "1" ]]; then
  # shellcheck source=/dev/null
  source "$ROOT/scripts/eq_ephemeral_pg.sh"
  eq_ephemeral_pg_start "${EQ_PERF_PROFILES:-pg18}" "edgequake-sv081"
  eq_ephemeral_pg_migrate
  set +e
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -c \
    "SELECT proname FROM pg_proc WHERE proname IN ('eq_serving_chunk_presence','eq_serving_vector_presence') ORDER BY 1;" \
    2>&1 | tee -a "$LOG"
  RC_DB=${PIPESTATUS[0]}
  set -e
  DB_NOTE="probed exit=${RC_DB}"
fi

cp "$LOG" "$ART_DIR/eq-serving-view-cargo.log"

python3 - <<'PY' "$ART_DIR/RUN_NOTES.md" "$RC_CONTRACT" "$RC_DB" "$DB_NOTE"
import sys
from pathlib import Path
from datetime import datetime, timezone
notes = Path(sys.argv[1])
rc_c, rc_db = int(sys.argv[2]), int(sys.argv[3])
db = sys.argv[4]
ok = rc_c == 0 and rc_db == 0
lines = [
    "# SPEC-081 RUN_NOTES — Serving view dual-SSOT",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Functions: `eq_serving_chunk_presence(uuid)`, `eq_serving_vector_presence(uuid, regclass)`",
    "- Serving view ≠ RAG ANN SSOT; ingest/ANN paths unchanged",
    "- Silent store unify: forbidden",
    f"- Contract exit: {rc_c}",
    f"- DB probe: {db}",
    "",
    "## Gate: " + ("GREEN" if ok else "RED"),
    "",
    "## Phase-4 note",
    "",
    "Broader dual-SSOT narrowing only if retract surfaces decrease without recall loss.",
]
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if ok else 1)
PY
