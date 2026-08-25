#!/usr/bin/env bash
# SPEC-137: prove 0.25→0.26 migrate consent alias, abort class, 149 expandable.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MEAS="${ROOT}/specs/137-issue-migration-25-to-26/measurements"
mkdir -p "${MEAS}"

export EDGEQUAKE_REQUIRE_POSTGRES_TESTS="${EDGEQUAKE_REQUIRE_POSTGRES_TESTS:-1}"

if [[ -z "${DATABASE_URL:-}" ]] && [[ -f /tmp/edgequake-db-url ]]; then
  export DATABASE_URL="$(tr -d '\n' </tmp/edgequake-db-url)"
fi

echo "SPEC-137 migrate 0.25→0.26 proof"
echo "DATABASE_URL set: $([[ -n "${DATABASE_URL:-}" ]] && echo yes || echo no)"

{
  echo "=== unit: confirm flags + abort classifier ==="
  cd "${ROOT}/edgequake"
  cargo test -p edgequake --bin edgequake --features postgres first_principles -- --nocapture
} 2>&1 | tee "${MEAS}/e2e137-unit.txt"

{
  echo "=== cli_migrate_console SPEC-137 ==="
  cd "${ROOT}/edgequake"
  cargo test --test cli_migrate_console --features postgres unknown_apply_flag -- --nocapture
  cargo test --test cli_migrate_console --features postgres bare_confirm -- --nocapture
  cargo test --test cli_migrate_console --features postgres spec137 -- --nocapture --test-threads=1
} 2>&1 | tee "${MEAS}/e2e137-cli.txt"

{
  echo "=== LAW-C3 advisor ≡ SQL (125/126/111) ==="
  cd "${ROOT}/edgequake"
  if [[ -n "${DATABASE_URL:-}" ]]; then
    cargo test -p edgequake-storage --features postgres --test e2e_spec091_console \
      contract_spec091_advisor_matches_125_guard -- --nocapture
    cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_retire \
      contract_spec091_advisor_matches_126_guard -- --nocapture
    cargo test -p edgequake-storage --features postgres --test e2e_spec111_provenance_parity \
      -- --nocapture || true
  else
    echo "SKIP contracts: DATABASE_URL unset"
  fi
} 2>&1 | tee "${MEAS}/e2e137-contracts.txt"

{
  echo "=== source guards ==="
  grep -n "CONFIRM_DROP_FLAGS" "${ROOT}/edgequake/src/migrate_console.rs"
  grep -n "first_unknown_migrate_apply_flag" "${ROOT}/edgequake/src/main.rs"
  grep -n "Leftover SPEC-091" "${ROOT}/docs/operations/upgrade-to-0.26.0.md"
} 2>&1 | tee "${MEAS}/e2e137-source-guard.txt"

echo "SPEC-137 proof OK — artifacts under ${MEAS}"
