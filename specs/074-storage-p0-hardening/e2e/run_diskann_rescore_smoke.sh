#!/usr/bin/env bash
# SPEC-074 — DiskANN opt-in recipe smoke: list=400 + rescore=200 (not silent default).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18-vectorscale}"

export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-4g}"
export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-2GB}"
export EQ_DISKANN_SMOKE=1
export EQ_PARETO_ROWS=2000
export EQ_PARETO_SPOT_ROWS=
export EQ_PARETO_REBUILD=0
# Recipe cell only (SPEC-074): list=400 implies rescore=200 via diskann_rescore_for_list
export EQ_PARETO_SEARCH_LIST=400
export EQ_PARETO_REF_SEARCH_LIST=800

ART_DIR="$ROOT/specs/074-storage-p0-hardening/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-diskann-rescore074-${PROFILE}.log"
REPORT="/tmp/eq-diskann-rescore074-${PROFILE}.jsonl"

echo "NOTE: SPEC-074 DiskANN rescore smoke profile=$PROFILE list=$EQ_PARETO_SEARCH_LIST"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-rescore074"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"

set +e
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec072_diskann_recall_pareto -- --nocapture 2>&1 | tee "$LOG"
RC=${PIPESTATUS[0]}
set -e

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true
cp "$REPORT" "$ART_DIR/eq-diskann-rescore-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-diskann-rescore-${PROFILE}-cargo.log"

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
cells = [o for o in rows if o.get("op") == "pareto_cell"]
greens = [o for o in cells if o.get("pass")]
lines = [
    "# SPEC-074 RUN_NOTES — DiskANN query_rescore recipe",
    "",
    f"- Date: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
    "- Recipe: `diskann.query_search_list_size=400` **and** `diskann.query_rescore=200` (list/2)",
    "- Helper: `edgequake_storage::diskann_optin_recipe_statements()` / `diskann_query_tuning_statements`",
    "- Silent flip: **Forbidden** (ops/harness SET LOCAL only; not boot default)",
    "- Full-gate @150k: already green in SPEC-072 with rescore=list/2; this smoke re-applies recipe on tiny corpus",
    f"- Smoke cargo exit: {rc}",
    f"- Smoke cells: {len(cells)} full_green={len(greens)}",
    "",
    "## Official guidance",
    "",
    "pgvectorscale: tune `query_rescore` for accuracy (default 50 is too low with list=400).",
    "SPEC-072 harness already set rescore=list/2; SPEC-074 productizes the tip in SSOT + shared helper.",
    "",
    "## Do not",
    "",
    "- Run DiskANN @150k with default list=100 / rescore=50",
    "- Enable vectorscale/DiskANN silently on existing DBs",
]
notes.write_text("\n".join(lines) + "\n")
print(notes.read_text())
raise SystemExit(0 if rc == 0 else rc)
PY
