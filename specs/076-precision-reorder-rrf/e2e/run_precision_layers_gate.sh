#!/usr/bin/env bash
# SPEC-076 — Precision layers gate (A3 contract + A4 lexical bake-off; optional DB smoke).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ART_DIR="$ROOT/specs/076-precision-reorder-rrf/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-precision076.log"
: >"$LOG"

EDGEQUAKE_DIR="$ROOT/edgequake"
cd "$EDGEQUAKE_DIR"

echo "NOTE: SPEC-076 precision-layers-gate (contracts + optional smoke)"

set +e
cargo test -p edgequake-storage --features postgres --test contract_spec076_ann_exact_reorder -- --nocapture 2>&1 | tee -a "$LOG"
RC_A3=${PIPESTATUS[0]}
cargo test -p edgequake-query --test contract_spec076_sparse_rrf_tip -- --nocapture 2>&1 | tee -a "$LOG"
RC_A4=${PIPESTATUS[0]}
set -e

SMOKE_NOTE="skipped (no ephemeral DB in gate; run cargo test --test e2e_spec076_exact_reorder_smoke with DATABASE_URL)"
if [[ "${EQ_PRECISION_SMOKE:-0}" == "1" ]]; then
  # shellcheck source=/dev/null
  source "$ROOT/scripts/eq_ephemeral_pg.sh"
  eq_ephemeral_pg_start "${EQ_PERF_PROFILES:-pg18}" "edgequake-pr076"
  eq_ephemeral_pg_migrate
  set +e
  cargo test -p edgequake-storage --features postgres --release \
    --test e2e_spec076_exact_reorder_smoke -- --nocapture 2>&1 | tee -a "$LOG"
  RC_SMOKE=${PIPESTATUS[0]}
  set -e
  SMOKE_NOTE="ran exit=${RC_SMOKE}"
else
  RC_SMOKE=0
fi

cp "$LOG" "$ART_DIR/eq-precision-layers-cargo.log"

python3 - <<'PY' "$ART_DIR/RUN_NOTES.md" "$RC_A3" "$RC_A4" "$RC_SMOKE" "$SMOKE_NOTE"
import sys
from pathlib import Path
from datetime import datetime, timezone
notes, rc_a3, rc_a4, rc_smoke, smoke = Path(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), sys.argv[5]
ok = rc_a3 == 0 and rc_a4 == 0 and rc_smoke == 0
lines = [
    "# SPEC-076 RUN_NOTES — Precision layers",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Floors unchanged (Wave-2 100k; DiskANN opt-in 150k)",
    "- Silent flip: forbidden (exact reorder default OFF; sparse fusion default weighted)",
    f"- A3 contract exit: {rc_a3}",
    f"- A4 lexical/RRF tip contract exit: {rc_a4}",
    f"- A3 DB smoke: {smoke}",
    "",
    "## A3 — Opt-in exact reorder",
    "",
    "- Env: `EDGEQUAKE_ANN_EXACT_REORDER=0|1` (default 0)",
    "- Env: `EDGEQUAKE_ANN_REORDER_CANDIDATE_K` (default 50)",
    "- SQL: MATERIALIZED CTE → `ORDER BY distance + 0` → LIMIT top_k",
    "- Filter columns stay inside the CTE (workspace/tenant)",
    "",
    "## A4 — Sparse FTS+ANN RRF tip",
    "",
    "- Default: sparse-first weighted (`EDGEQUAKE_SPARSE_FUSION` unset)",
    "- Tip: `EDGEQUAKE_SPARSE_FUSION=rrf` recovers lexical SKU in top-3 vs ANN-only miss",
    "- `content_tsv` upsert honesty asserted (SPEC-058/M091)",
    "- Mix/RRF ≠ promoted ANN floor",
    "",
    f"## Gate: {'GREEN' if ok else 'FAIL'}",
    "",
]
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if ok else 1)
PY
