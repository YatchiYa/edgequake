#!/usr/bin/env bash
# SPEC-069 — Dedicated WS table mid-scale ladder (claim gate, not day-2 sizing).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18}"

export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-8g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-4GB}"
export EQ_DEDICATED_ROWS_LIST="${EQ_DEDICATED_ROWS_LIST:-100000,125000,150000,200000}"
export EQ_DEDICATED_EF_LIST="${EQ_DEDICATED_EF_LIST:-80,240}"
export EQ_DEDICATED_CONTENTION="${EQ_DEDICATED_CONTENTION:-1}"
# Dedicated path: do not force partial HNSW
unset EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE || true

ART_DIR="$ROOT/specs/069-dedicated-midscale/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-dedicated-midscale-${PROFILE}.log"
REPORT="/tmp/eq-dedicated-midscale-${PROFILE}.jsonl"

echo "NOTE: SPEC-069 dedicated midscale rows=$EQ_DEDICATED_ROWS_LIST ef=$EQ_DEDICATED_EF_LIST"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-ded069"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec069_dedicated_ann_ladder -- --nocapture 2>&1 | tee "$LOG"
RC=${PIPESTATUS[0]}
set -e

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true

if grep -E 'SKIP:.*(DATABASE_URL|POSTGRES_PASSWORD)' "$LOG" >/dev/null 2>&1; then
  echo "ERROR: DATABASE soft-skip under REQUIRE_POSTGRES"
  exit 1
fi

cp "$REPORT" "$ART_DIR/eq-dedicated-midscale-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-dedicated-midscale-${PROFILE}-cargo.log"

python3 - <<'PY' "$REPORT" "$ART_DIR/DEDICATED_SUMMARY.md"
import json, sys, re
from pathlib import Path
report, out = Path(sys.argv[1]), Path(sys.argv[2])
rows = []
for line in report.read_text().splitlines():
    try:
        rows.append(json.loads(line))
    except Exception:
        pass
greens = []
stress = []
for o in rows:
    if o.get("op") == "dedicated_stress":
        stress.append(o)
        if o.get("pass"):
            greens.append(o.get("detail", ""))
decision = next((o for o in rows if o.get("op") == "dedicated_decision"), None)
lines = ["# SPEC-069 dedicated mid-scale summary", ""]
lines.append(f"- stress cells: {len(stress)}")
lines.append(f"- full_green (clients=16 ladder/contention): {len(greens)}")
if decision:
    lines.append(f"- decision: `{decision.get('detail')}` pass={decision.get('pass')}")
lines.append("")
lines.append("| detail | pass | p95_ms |")
lines.append("|--------|------|--------|")
for o in stress:
    d = str(o.get("detail", ""))[:140]
    lines.append(f"| `{d}` | {o.get('pass')} | {o.get('p95_ms')} |")
out.write_text("\n".join(lines) + "\n")
print(out.read_text())
PY

if [[ "$RC" -ne 0 ]]; then
  echo "WARN dedicated-midscale finished with test_rc=$RC (artifacts archived)"
  exit 1
fi
echo "OK dedicated-midscale -> $ART_DIR/eq-dedicated-midscale-${PROFILE}.jsonl"
