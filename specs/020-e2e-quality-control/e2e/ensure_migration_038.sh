#!/usr/bin/env bash
# SPEC-020 — Auto-repair migration-038 indexes before strict QC (first principles).
#
# Bootstrap already runs size-aware apply on startup; this handles residual
# missing indexes on long-lived dev DBs so SPEC020_STRICT_MIGRATION can pass.
#
# Skip: SPEC020_AUTO_MIGRATION=0
# Large graphs: SPEC020_MIGRATION_CONCURRENT=1 → --apply --concurrent --yes
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APPLY_SCRIPT="$ROOT/edgequake/scripts/migrations/apply_038.sh"
HEALTH_JSON="${1:-}"

migration_degraded() {
  local body="$1"
  echo "$body" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    idx = d.get('schema', {}).get('source_ids_indexes', {})
    ready = idx.get('ready', True)
    missing = idx.get('missing_indexes') or []
    print('yes' if ready is False or len(missing) > 0 else 'no')
except Exception:
    print('no')
" | grep -q '^yes$'
}

resolve_database_url() {
  if [[ -n "${DATABASE_URL:-}" ]]; then
    echo "$DATABASE_URL"
    return
  fi
  if [[ -f /tmp/edgequake-db-url ]]; then
    cat /tmp/edgequake-db-url
    return
  fi
  echo "postgres://edgequake:edgequake@localhost:5433/edgequake"
}

if [[ "${SPEC020_AUTO_MIGRATION:-1}" == "0" ]]; then
  echo "→ SPEC020_AUTO_MIGRATION=0 — skipping migration-038 auto-repair"
  exit 0
fi

if [[ -z "$HEALTH_JSON" ]]; then
  echo "→ No health JSON — skipping migration check"
  exit 0
fi

if ! migration_degraded "$HEALTH_JSON"; then
  echo "✓ migration-038 indexes ready (no auto-repair needed)"
  exit 0
fi

missing="$(echo "$HEALTH_JSON" | python3 -c "
import json,sys
try:
  d=json.load(sys.stdin)
  idx=d.get('schema',{}).get('source_ids_indexes',{})
  print(len(idx.get('missing_indexes',[]) or []))
except Exception:
  print(0)
" 2>/dev/null || echo 0)"

echo "⚠ migration-038 degraded ($missing missing indexes) — auto-repair via apply_038.sh"

chmod +x "$APPLY_SCRIPT"
export DATABASE_URL="$(resolve_database_url)"

apply_and_verify() {
  local mode="$1"
  if [[ "$mode" == "concurrent" ]]; then
    "$APPLY_SCRIPT" --apply --concurrent --yes
  else
    "$APPLY_SCRIPT" --apply --yes
  fi
  "$APPLY_SCRIPT" --verify
}

if [[ "${SPEC020_MIGRATION_CONCURRENT:-0}" == "1" ]]; then
  apply_and_verify concurrent
else
  if ! apply_and_verify standard 2>&1; then
    echo "⚠ standard apply failed — retrying with CONCURRENT index build"
    apply_and_verify concurrent
  fi
fi

echo "✓ migration-038 auto-repair complete"
echo "REPAIRED=1"
