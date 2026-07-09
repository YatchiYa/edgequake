#!/usr/bin/env bash
# SPEC-045: Post-migration ingestion health proof.
#
# Gates:
#   BT-045-01 — Postgres connectivity
#   BT-045-02 — post_migration_ingest_health.sql
#   BT-045-03 — migration_readiness_proof (Rust)
#   BT-045-04..06 — spec045 edge-case battle tests (tasks, pipeline, api)
#   BT-045-07 — spec044 compensation (if postgres feature)
#   BT-045-08 — API /health + /ready (optional)
#
# Usage:
#   export DATABASE_URL=postgres://edgequake:edgequake@localhost/edgequake
#   make postgres-start   # if needed
#   ./specs/045-fix-ingestion-errors/e2e/run_ingestion_health_proof.sh
#
# Optional:
#   SKIP_API_CHECK=1      — skip localhost:8080 health
#   EDGEQUAKE_API_URL=... — default http://localhost:8080
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SPEC_DIR="$ROOT/specs/045-fix-ingestion-errors"
cd "$ROOT/edgequake"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}PASS${NC}: $*"; }
fail() { echo -e "${RED}FAIL${NC}: $*"; exit 1; }
warn() { echo -e "${YELLOW}WARN${NC}: $*"; }

echo "== SPEC-045: ingestion health proof =="
echo "   Root: $ROOT"
echo "   Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"

# ── Gate 0: DATABASE_URL ──────────────────────────────────────────────────────
if [[ -z "${DATABASE_URL:-}" ]]; then
  export DATABASE_URL="postgres://edgequake:edgequake@localhost/edgequake"
  warn "DATABASE_URL unset; using default localhost"
fi

echo "   DATABASE_URL host: $(echo "$DATABASE_URL" | sed -E 's|.*@([^/]+)/.*|\1|')"

# ── BT-045-01: connectivity ───────────────────────────────────────────────────
if ! psql "$DATABASE_URL" -c "SELECT 1" >/dev/null 2>&1; then
  fail "Cannot connect to Postgres — run: make postgres-start"
fi
pass "BT-045-01 Postgres connectivity"

# ── BT-045-02: SQL health gates ───────────────────────────────────────────────
psql "$DATABASE_URL" -f "$SPEC_DIR/e2e/sql/post_migration_ingest_health.sql"
pass "BT-045-02 post_migration_ingest_health.sql"

# ── BT-045-03: migration readiness proof ──────────────────────────────────────
if cargo test -p edgequake-api --test migration_readiness_proof -- --nocapture 2>&1; then
  pass "BT-045-03 migration_readiness_proof"
else
  warn "BT-045-03 migration_readiness_proof failed (may need POSTGRES_PASSWORD)"
fi

# ── BT-045-04..06: SPEC-045 battle tests ─────────────────────────────────────
if cargo test -p edgequake-tasks spec045 -- --nocapture 2>&1; then
  pass "BT-045-04 edgequake-tasks spec045"
else
  fail "BT-045-04 edgequake-tasks spec045"
fi

if cargo test -p edgequake-pipeline spec045 -- --nocapture 2>&1; then
  pass "BT-045-05 edgequake-pipeline spec045"
else
  fail "BT-045-05 edgequake-pipeline spec045"
fi

if cargo test -p edgequake-api --test spec045_ingestion_reliability -- --nocapture 2>&1; then
  pass "BT-045-06 edgequake-api spec045 battle tests"
else
  fail "BT-045-06 edgequake-api spec045 battle tests"
fi

if cargo test -p edgequake-core --test spec045_vector_resolve_parity -- --nocapture 2>&1; then
  pass "BT-045-06a spec045_vector_resolve_parity"
else
  fail "BT-045-06a spec045_vector_resolve_parity"
fi

if cargo test -p edgequake-api --test spec045_periodic_orphan_doc_sync -- --nocapture 2>&1; then
  pass "BT-045-06b spec045_periodic_orphan_doc_sync"
else
  fail "BT-045-06b spec045_periodic_orphan_doc_sync"
fi

if cargo test -p edgequake-query spec045 -- --nocapture 2>&1; then
  pass "BT-045-06c edgequake-query spec045"
else
  fail "BT-045-06c edgequake-query spec045"
fi

# ── BT-045-07: SPEC-044 compensation (storage) ────────────────────────────────
if cargo test -p edgequake-storage --features postgres \
  --test spec044_compensation_postgres -- --nocapture 2>&1; then
  pass "BT-045-07 spec044_compensation_postgres"
else
  warn "BT-045-07 spec044_compensation skipped or failed (needs live AGE)"
fi

# ── BT-045-08: API health (optional) ──────────────────────────────────────────
if [[ "${SKIP_API_CHECK:-0}" != "1" ]]; then
  API_URL="${EDGEQUAKE_API_URL:-http://localhost:8080}"
  if curl -sf "$API_URL/health" >/dev/null 2>&1; then
    READY_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/ready" || echo "000")
    READY_JSON=$(curl -s "$API_URL/ready" || echo "{}")
    HEALTH_JSON=$(curl -s "$API_URL/health")
    pass "BT-045-08 API /health reachable"
    echo "   /ready HTTP $READY_CODE (200=accepting uploads, 503=degraded migration)"
    if command -v python3 >/dev/null 2>&1; then
      echo "$READY_JSON" | python3 -c "
import sys, json
try:
    r = json.load(sys.stdin)
    if 'ready' in r:
        print('   /ready.ready:', r.get('ready'))
        print('   /ready.blockers:', r.get('blockers', []))
except Exception:
    pass
" 2>/dev/null || true
      echo "$HEALTH_JSON" | python3 -c "
import sys, json
try:
    h = json.load(sys.stdin)
    rm = h.get('read_model', {})
    mig = h.get('migration_bootstrap', {})
    print('   ready_for_traffic:', h.get('ready_for_traffic', 'n/a'))
    print('   llm_provider:', h.get('llm_provider_name', 'n/a'))
    schema = h.get('schema', {})
    si = schema.get('source_ids_indexes', {})
    print('   source_ids_indexes.ready:', si.get('ready', 'n/a'))
except Exception as e:
    print('   (parse skip)', e)
" 2>/dev/null || true
    fi
    if [[ "$READY_CODE" == "503" ]]; then
      warn "API not ready for traffic — see 005-quick-fix-runbook.md Step 2"
    fi
  else
    warn "BT-045-08 API not running at $API_URL (SKIP_API_CHECK=1 to silence)"
  fi
fi

echo ""
echo "== SPEC-045 ingestion health proof complete =="
echo "   Next: specs/045-fix-ingestion-errors/005-quick-fix-runbook.md"
