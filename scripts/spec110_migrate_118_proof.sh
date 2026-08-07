#!/usr/bin/env bash
# SPEC-110: prove migration 118/121 conflict-key dedup + checksum repair contracts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MEAS="${ROOT}/specs/110-migration-issue/measurements"
mkdir -p "${MEAS}"

export EDGEQUAKE_REQUIRE_POSTGRES_TESTS="${EDGEQUAKE_REQUIRE_POSTGRES_TESTS:-1}"

if [[ -z "${DATABASE_URL:-}" ]] && [[ -f /tmp/edgequake-db-url ]]; then
  export DATABASE_URL="$(tr -d '\n' </tmp/edgequake-db-url)"
fi

echo "SPEC-110 migrate 118 proof"
echo "DATABASE_URL set: $([[ -n "${DATABASE_URL:-}" ]] && echo yes || echo no)"

{
  echo "=== contract_spec110_migration_dedup ==="
  cd "${ROOT}/edgequake"
  cargo test -p edgequake-api --test contract_spec110_migration_dedup -- --nocapture
} 2>&1 | tee "${MEAS}/e2e110-source-guard.txt"

{
  echo "=== e2e_spec110_wsdoc_on_conflict ==="
  cd "${ROOT}/edgequake"
  cargo test -p edgequake-storage --features postgres --test e2e_spec110_wsdoc_on_conflict -- --nocapture
} 2>&1 | tee "${MEAS}/e2e110-patched-ok.txt"

{
  echo "=== m118/m121 checksum loud-refuse (integration contracts; avoid --lib while SPEC-109 WIP) ==="
  cd "${ROOT}/edgequake"
  # Unit tests in m118/m121 are covered by contract_spec110 + spec083 source greps.
  # Full `cargo test -p edgequake-api --lib` currently fails on unrelated SPEC-109
  # CreateWorkspaceRequest field gaps in the same working tree.
  cargo test -p edgequake-api --test contract_spec110_migration_dedup -- --nocapture
  cargo test -p edgequake-api --features postgres --test spec083_matrix_contracts \
    contract_checksum_drift_fails_loud -- --nocapture
} 2>&1 | tee "${MEAS}/e2e110-checksum-repair.txt"

{
  echo "=== checksums.lock 118/121 ==="
  grep -E '118_|121_' "${ROOT}/edgequake/migrations/checksums.lock"
  "${ROOT}/scripts/check_migration_checksums.sh"
} 2>&1 | tee "${MEAS}/e2e110-checksums-after.txt"

# Capture old-SQL failure evidence from the e2e log (E2E-110-01).
if grep -q "cannot affect row a second time\|e2e110_01_old_118" "${MEAS}/e2e110-patched-ok.txt"; then
  grep -E "e2e110_01|cannot affect row a second time|test result" "${MEAS}/e2e110-patched-ok.txt" \
    | tee "${MEAS}/e2e110-repro-0241.txt" >/dev/null || true
fi

echo "SPEC-110 proof OK — artifacts under ${MEAS}"
