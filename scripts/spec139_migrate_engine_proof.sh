#!/usr/bin/env bash
# SPEC-139: prove mid-cutover engine (iw2 21000, W3 coverage-sum, remainder).
# Fail-closed: DATABASE_URL required; SKIP/ignored is not a pass.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MEAS="${ROOT}/specs/139-issue-migration/measurements"
MIG="${ROOT}/edgequake/migrations"
mkdir -p "${MEAS}"

if [[ -z "${DATABASE_URL:-}" ]] && [[ -f /tmp/edgequake-db-url ]]; then
  export DATABASE_URL="$(tr -d '\n' </tmp/edgequake-db-url)"
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "FAIL: DATABASE_URL unset (and /tmp/edgequake-db-url missing) — unfakable e2e cannot run"
  exit 1
fi

export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1

echo "SPEC-139 migrate engine proof"
echo "DATABASE_URL set: yes"
echo "EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1"

fail_if_skip() {
  local file="$1"
  if grep -E 'SKIP:|skipped' "${file}" | grep -v 'filtered out' >/dev/null; then
    echo "FAIL: skip/ignored in ${file}"
    grep -E 'SKIP:|skipped' "${file}" || true
    exit 1
  fi
}

require_line() {
  local file="$1"
  local pat="$2"
  if ! grep -E "${pat}" "${file}" >/dev/null; then
    echo "FAIL: missing unfakable fact '${pat}' in ${file}"
    exit 1
  fi
}

{
  echo "=== U-139-DEDUP (no DB) ==="
  cd "${ROOT}/edgequake"
  cargo test -p edgequake-storage --lib contract_spec139_dedupe -- --nocapture
} 2>&1 | tee "${MEAS}/e2e139-dedupe.txt"
require_line "${MEAS}/e2e139-dedupe.txt" 'test result: ok\. 3 passed'

{
  echo "=== remainder step ids ==="
  cd "${ROOT}/edgequake"
  cargo test -p edgequake-storage --features postgres --lib \
    contract_spec139_remainder_step_ids_stable -- --nocapture
} 2>&1 | tee "${MEAS}/e2e139-remainder-unit.txt"
require_line "${MEAS}/e2e139-remainder-unit.txt" 'test result: ok\. 1 passed'

{
  echo "=== verify gate (coverage-first) ==="
  cd "${ROOT}/edgequake"
  cargo test -p edgequake-storage --features postgres --lib \
    contract_spec091_verify_report_gate -- --nocapture
} 2>&1 | tee "${MEAS}/e2e139-verify-gate.txt"
require_line "${MEAS}/e2e139-verify-gate.txt" 'test result: ok\. 1 passed'

{
  echo "=== E2E-139-01..08 (Postgres required) ==="
  cd "${ROOT}/edgequake"
  cargo test -p edgequake-storage --features postgres --test e2e_spec139_engine \
    -- --nocapture --test-threads=1
} 2>&1 | tee "${MEAS}/e2e139-engine.txt"
fail_if_skip "${MEAS}/e2e139-engine.txt"
require_line "${MEAS}/e2e139-engine.txt" 'test result: ok\. 8 passed; 0 failed; 0 ignored'
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE SQLSTATE=21000'
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE E2E-139-01 typed_count=1'
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE E2E-139-02 typed_count=1'
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE E2E-139-03 expected=8 actual_uncovered=0 actual_covered=8'
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE E2E-139-04 reclaimed='
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE E2E-139-05 before_artifacts=0 after_lineage=1'
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE E2E-139-06 second_job_ran=1'
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE E2E-139-07 .*doc_exists=1 doc_shells_after=0'
require_line "${MEAS}/e2e139-engine.txt" 'UNFAKABLE E2E-139-08 verify_passes=1 residue_lineage='
# Two 21000 probes (entity + relationship).
count_21000="$(grep -c 'UNFAKABLE SQLSTATE=21000' "${MEAS}/e2e139-engine.txt")"
if [[ "${count_21000}" -lt 2 ]]; then
  echo "FAIL: need ≥2 SQLSTATE=21000 observations, got ${count_21000}"
  exit 1
fi

{
  echo "=== LAW-C3 advisor ≡ SQL (125/126) + 131 abort (fail-closed) ==="
  cd "${ROOT}/edgequake"
  if ! grep -q "SPEC-091 IW2 ABORT" "${MIG}/131_spec091_fleet_vector_drop.sql"; then
    echo "FAIL: 131 abort text missing — DROP must stay fail-closed"
    exit 1
  fi
  grep -c "SPEC-091 IW2 ABORT" "${MIG}/131_spec091_fleet_vector_drop.sql"
  grep -q "ABORT" "${MIG}/125_spec091_kv_drop.sql"
  grep -q "ABORT" "${MIG}/126_spec091_vector_drop.sql"
  cargo test -p edgequake-storage --features postgres --test e2e_spec091_console \
    contract_spec091_advisor_matches_125_guard -- --nocapture
  cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_retire \
    contract_spec091_advisor_matches_126_guard -- --nocapture
  cargo test -p edgequake-storage --features postgres --test e2e_spec111_provenance_parity \
    e2e_spec111_17_abort_without_provenance -- --nocapture
} 2>&1 | tee "${MEAS}/e2e139-contracts.txt"
fail_if_skip "${MEAS}/e2e139-contracts.txt"
require_line "${MEAS}/e2e139-contracts.txt" 'test contract_spec091_advisor_matches_125_guard \.\.\. ok'
require_line "${MEAS}/e2e139-contracts.txt" 'test contract_spec091_advisor_matches_126_guard \.\.\. ok'
require_line "${MEAS}/e2e139-contracts.txt" 'test e2e_spec111_17_abort_without_provenance \.\.\. ok'

{
  echo "=== source guards ==="
  grep -n "dedupe_last_write_wins" \
    "${ROOT}/edgequake/crates/edgequake-storage/src/migration_engine/fleet_embedding_backfill.rs"
  grep -n "reclaim_verify_failed_jobs" \
    "${ROOT}/edgequake/crates/edgequake-storage/src/migration_engine/lease.rs"
  grep -n "w5-artifact-remainder" \
    "${ROOT}/edgequake/crates/edgequake-storage/src/migration_engine/family_remainder.rs"
  grep -n "wc-shell-remainder" \
    "${ROOT}/edgequake/crates/edgequake-storage/src/migration_engine/family_remainder.rs"
  grep -n "ShellRemainderJob" \
    "${ROOT}/edgequake/crates/edgequake-storage/src/migration_engine/runner.rs"
  grep -n "LEGACY_CHUNK_VECTOR_ID_RE" \
    "${ROOT}/edgequake/crates/edgequake-storage/src/migration_engine/verify.rs"
  grep -n "continuing remaining jobs" \
    "${ROOT}/edgequake/crates/edgequake-storage/src/migration_engine/runner.rs"
} 2>&1 | tee "${MEAS}/e2e139-source-guard.txt"

cat > "${MEAS}/SUMMARY.md" <<EOF
# SPEC-139 proof SUMMARY

- date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
- DATABASE_URL: set
- EDGEQUAKE_REQUIRE_POSTGRES_TESTS: 1
- VERSION pin: still 0.26.2 (0.26.3 after tag)
- e2e_spec139_engine: 8 passed; 0 failed; 0 ignored
- SQLSTATE 21000 observations: ${count_21000}
- artifacts: e2e139-dedupe.txt, e2e139-remainder-unit.txt, e2e139-verify-gate.txt, e2e139-engine.txt, e2e139-contracts.txt, e2e139-source-guard.txt
- result: OK
EOF

echo "SPEC-139 proof OK — artifacts under ${MEAS}"
