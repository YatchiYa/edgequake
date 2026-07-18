#!/usr/bin/env bash
# SPEC-066 — Wave-2 ceiling ladder (L2 / L3 / seek) on pg18 by default.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18}"
STEP="$(echo "${EQ_CEILING_STEP:-L2}" | tr '[:lower:]' '[:upper:]')"
export EQ_CEILING_STEP="$STEP"

# Wave-2 shape (locked — no silent prod default flip elsewhere)
export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1
export EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS="${EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS:-1000}"
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1

# Residency for large corpora (SPEC-065/066 — HNSW build needs maintenance_work_mem;
# query path needs shared_buffers / OS cache residency).
export EQ_EPHEMERAL_PG_SHM="${EQ_EPHEMERAL_PG_SHM:-8g}"

case "$STEP" in
  L2)
    export EDGEQUAKE_CEILING_ROWS="${EDGEQUAKE_CEILING_ROWS:-500000}"
    export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
    export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-8GB}"
    ;;
  L3)
    export EDGEQUAKE_CEILING_ROWS="${EDGEQUAKE_CEILING_ROWS:-1000000}"
    export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-8GB}"
    export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-16GB}"
    ;;
  SEEK)
    export EDGEQUAKE_CEILING_ROWS="${EDGEQUAKE_CEILING_ROWS:-250000}"
    export EQ_CEILING_STEP=SEEK
    export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
    export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-4GB}"
    ;;
  G1|GRAPH)
    # Graph-only soak (no ANN ladder)
    export EQ_CEILING_INCLUDE_GRAPH=1
    export EQ_CEILING_GRAPH_ONLY=1
    export EQ_CEILING_STEP=G1
    export EDGEQUAKE_CEILING_ROWS="${EDGEQUAKE_CEILING_ROWS:-0}"
    export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-2GB}"
    export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-2GB}"
    ;;
  *)
    # Custom numeric step label
    if [[ "$STEP" =~ ^[0-9]+$ ]]; then
      export EDGEQUAKE_CEILING_ROWS="$STEP"
      export EQ_CEILING_STEP="N${STEP}"
    fi
    export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
    export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-4GB}"
    ;;
esac

export EQ_EPHEMERAL_PG_SHARED_BUFFERS="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-4GB}"
export EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-4GB}"

ART_DIR="$ROOT/specs/066-ceiling-proof/e2e/artifacts"
mkdir -p "$ART_DIR"
LABEL="${EQ_CEILING_STEP}-${EDGEQUAKE_CEILING_ROWS}"
LOG="/tmp/eq-ceiling-${PROFILE}-${LABEL}.log"
REPORT="/tmp/eq-ceiling-${PROFILE}-${LABEL}.jsonl"

echo "NOTE: SPEC-066 ceiling Wave-2 step=$EQ_CEILING_STEP rows=$EDGEQUAKE_CEILING_ROWS profile=$PROFILE"
echo "NOTE: shared_buffers=$EQ_EPHEMERAL_PG_SHARED_BUFFERS shm=$EQ_EPHEMERAL_PG_SHM"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-ceil"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"

ANN_RC=0
GRAPH_RC=0
if [[ "${EQ_CEILING_GRAPH_ONLY:-0}" != "1" ]]; then
  set +e
  cargo test -p edgequake-storage --features postgres --release \
    --test e2e_spec066_ceiling_ladder_ann -- --nocapture 2>&1 | tee "$LOG"
  ANN_RC=${PIPESTATUS[0]}
  set -e
fi

if [[ "${EQ_CEILING_INCLUDE_GRAPH:-0}" == "1" ]]; then
  set +e
  cargo test -p edgequake-storage --features postgres --release \
    --test e2e_spec066_graph_g1 -- --nocapture 2>&1 | tee -a "$LOG"
  GRAPH_RC=${PIPESTATUS[0]}
  set -e
fi

# Always materialize JSONL from the log (cliff archive must not depend on exit code).
grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >"$REPORT" || true

if grep -E 'SKIP:.*(DATABASE_URL|POSTGRES_PASSWORD)' "$LOG" >/dev/null 2>&1; then
  echo "ERROR: DATABASE soft-skip under REQUIRE_POSTGRES"
  exit 1
fi

cp "$REPORT" "$ART_DIR/eq-ceiling-${PROFILE}-${LABEL}.jsonl"
cp "$LOG" "$ART_DIR/eq-ceiling-${PROFILE}-${LABEL}-cargo.log"

python3 "$ROOT/specs/066-ceiling-proof/e2e/summarize_ceiling.py" \
  "$REPORT" "$ART_DIR/CEILING_SUMMARY.md" "$PROFILE" "$LABEL"

if [[ "$ANN_RC" -ne 0 || "$GRAPH_RC" -ne 0 ]]; then
  echo "WARN ceiling step $LABEL finished with test_rc ann=$ANN_RC graph=$GRAPH_RC (artifacts archived)"
  # Hang-cliff / panic => fail make; soft SLO cliffs still exit 0 from the test.
  exit 1
fi

echo "OK ceiling step $LABEL on $PROFILE -> $ART_DIR/eq-ceiling-${PROFILE}-${LABEL}.jsonl"
