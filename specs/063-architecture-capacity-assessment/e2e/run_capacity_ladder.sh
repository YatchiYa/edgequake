#!/usr/bin/env bash
# SPEC-063 — capacity ladder L1/L2/L3 on a single major (default pg18).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18}"
LADDER="$(echo "${EDGEQUAKE_CAPACITY_LADDER:-L1}" | tr '[:lower:]' '[:upper:]')"
export EDGEQUAKE_CAPACITY_LADDER="$LADDER"
export EDGEQUAKE_PERF_SCALE=large
export EDGEQUAKE_PERF_RELEASE=1

ART_DIR="$ROOT/specs/063-architecture-capacity-assessment/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-capacity-${PROFILE}-${LADDER}.log"
REPORT="/tmp/eq-capacity-${PROFILE}-${LADDER}.jsonl"

echo "NOTE: SPEC-063 capacity ladder $LADDER on $PROFILE (EDGEQUAKE_PERF_SCALE=large, --release)"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-cap"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec063_capacity_ladder_ann -- --nocapture 2>&1 | tee "$LOG"

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >>"$REPORT" || true

if grep -E 'SKIP:.*(DATABASE_URL|POSTGRES_PASSWORD)' "$LOG" >/dev/null 2>&1; then
  echo "ERROR: DATABASE soft-skip under REQUIRE_POSTGRES"
  exit 1
fi

cp "$REPORT" "$ART_DIR/eq-capacity-${PROFILE}-${LADDER}.jsonl"
cp "$LOG" "$ART_DIR/eq-capacity-${PROFILE}-${LADDER}-cargo.log"
echo "OK capacity ladder $LADDER on $PROFILE → $ART_DIR/eq-capacity-${PROFILE}-${LADDER}.jsonl"
