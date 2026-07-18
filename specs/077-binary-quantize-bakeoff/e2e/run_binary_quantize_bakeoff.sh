#!/usr/bin/env bash
# SPEC-077 — Binary quantize + rerank vs Wave-2 filtered recall bake-off.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18}"

export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-4g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-2GB}"
export EQ_BQ_ROWS="${EQ_BQ_ROWS:-2000}"

ART_DIR="$ROOT/specs/077-binary-quantize-bakeoff/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-binary-quantize077-${PROFILE}.log"
REPORT="/tmp/eq-binary-quantize077-${PROFILE}.jsonl"

echo "NOTE: SPEC-077 binary-quantize-bakeoff profile=$PROFILE rows=$EQ_BQ_ROWS"

EDGEQUAKE_DIR="$ROOT/edgequake"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --test contract_spec077_binary_quantize -- --nocapture 2>&1 | tee "$LOG"
RC_CONTRACT=${PIPESTATUS[0]}
set -e

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-bq077"
eq_ephemeral_pg_migrate

: >"$REPORT"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec077_binary_quantize_bakeoff -- --nocapture 2>&1 | tee -a "$LOG"
RC=${PIPESTATUS[0]}
set -e

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true
cp "$REPORT" "$ART_DIR/eq-binary-quantize-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-binary-quantize-${PROFILE}-cargo.log"

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
cells = [o for o in rows if o.get("op") == "bq077_cell"]
recalls = [o for o in rows if o.get("op") == "bq077_filtered_recall"]
lines = [
    "# SPEC-077 RUN_NOTES — Binary quantize bake-off",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Promote metric: **filtered** recall@20 (binary vs Wave-2 reference)",
    "- Wave-2 remains product default; binary+rerank is **opt-in study**",
    "- Floors unchanged (no raise from smoke)",
    "- Silent flip: forbidden (`EDGEQUAKE_BINARY_QUANTIZE` default OFF)",
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
    "- `build_binary_hnsw_index_sql` — expression HNSW `bit_hamming_ops`",
    "- `build_binary_rerank_select_sql` — Hamming candidates → exact halfvec reorder",
    "- Env: `EDGEQUAKE_BINARY_QUANTIZE` (default off), `EDGEQUAKE_BINARY_CANDIDATE_K` (default 200)",
    "",
    "## Decision",
    "",
    "Do **not** silent-flip product default from this smoke. Re-run at mid-scale + full gate before any promote.",
]
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if rc == 0 and rc_contract == 0 else 1)
PY
