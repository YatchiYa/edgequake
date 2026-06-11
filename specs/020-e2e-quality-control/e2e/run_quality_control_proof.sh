#!/usr/bin/env bash
# SPEC-020 — Full quality-control Playwright proof (live stack + mock workspace).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
WEBUI="$ROOT/edgequake_webui"
PROOF_DIR="$(cd "$(dirname "$0")" && pwd)"
SCREENSHOTS="$PROOF_DIR/screenshots"
BACKEND_PORT="${BACKEND_PORT:-8081}"
FRONTEND_PORT="${FRONTEND_PORT:-3001}"
LOG="$PROOF_DIR/001-test-run.log"

resolve_auth_flag() {
  if [[ "${SPEC020_AUTH_PROOF:-0}" == "1" ]]; then
    echo "true"
  else
    echo "${DEV_AUTH_ENABLED:-false}"
  fi
}

backend_auth_enabled() {
  local url="$1"
  local code
  code="$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 "${url}/api/v1/documents" 2>/dev/null || echo 000)"
  [[ "$code" == "401" ]]
}

ensure_auth_stack() {
  local url="$1"
  local auth_flag="$2"
  local port="${url##*:}"
  if [[ "$auth_flag" != "true" ]]; then
    return 0
  fi
  if backend_auth_enabled "$url"; then
    echo "✓ Backend auth enabled at $url"
    return 0
  fi
  echo "→ Restarting stack with DEV_AUTH_ENABLED=true (SPEC020_AUTH_PROOF)"
  local pid
  pid="$(lsof -nP -iTCP:"${port}" -sTCP:LISTEN -t 2>/dev/null | head -1 || true)"
  if [[ -n "$pid" ]]; then
    kill "$pid" 2>/dev/null || true
    sleep 2
  fi
  (cd "$ROOT" && DEV_AUTH_ENABLED=true EDGEQUAKE_LLM_PROVIDER=ollama \
    make backend-bg BACKEND_PORT="$port" --no-print-directory)
  for _ in $(seq 1 45); do
    backend_healthy "http://127.0.0.1:${port}" && backend_auth_enabled "http://127.0.0.1:${port}" && return 0
    sleep 2
  done
  echo "✗ Backend auth mode failed to start"
  return 1
}

AUTH_FLAG="$(resolve_auth_flag)"

backend_healthy() {
  local body
  body="$(curl -sf --max-time 3 "$1/health" 2>/dev/null || true)"
  echo "$body" | grep -qE '"status"[[:space:]]*:[[:space:]]*"(healthy|degraded)"' \
    && echo "$body" | grep -q '"storage_mode"' \
    && echo "$body" | grep -q '"kv_storage"[[:space:]]*:[[:space:]]*true'
}

backend_ready() {
  curl -sf --max-time 3 "$1/ready" >/dev/null 2>&1
}

restart_backend_for_migration_cache() {
  local url="$1"
  local port="${url##*:}"
  echo "→ Restarting backend on :${port} (refresh migration-038 bootstrap cache)"
  local pid
  pid="$(lsof -nP -iTCP:"${port}" -sTCP:LISTEN -t 2>/dev/null | head -1 || true)"
  if [[ -n "$pid" ]]; then
    kill "$pid" 2>/dev/null || true
    sleep 2
  fi
  (cd "$ROOT" && DEV_AUTH_ENABLED="$AUTH_FLAG" EDGEQUAKE_LLM_PROVIDER=ollama \
    make backend-bg BACKEND_PORT="$port" --no-print-directory)
  for _ in $(seq 1 45); do
    backend_healthy "http://127.0.0.1:${port}" && backend_ready "http://127.0.0.1:${port}" && return 0
    sleep 2
  done
  echo "✗ Backend did not become ready after migration repair restart"
  return 1
}

resolve_backend_url() {
  for port in "$BACKEND_PORT" 8081 8080; do
    local url="http://127.0.0.1:${port}"
    if backend_healthy "$url"; then
      echo "$url"
      return 0
    fi
  done
  echo "http://127.0.0.1:${BACKEND_PORT}"
}

mkdir -p "$SCREENSHOTS"
API_URL="${EQ_BACKEND_URL:-$(resolve_backend_url)}"

if ! backend_healthy "$API_URL"; then
  echo "→ Starting full stack (make dev-bg)"
  (cd "$ROOT" && DEV_AUTH_ENABLED="$AUTH_FLAG" \
    make dev-bg BACKEND_PORT="$BACKEND_PORT" FRONTEND_PORT="$FRONTEND_PORT" --no-print-directory)
  API_URL="$(resolve_backend_url)"
fi

for _ in $(seq 1 60); do
  API_URL="$(resolve_backend_url)"
  backend_healthy "$API_URL" && break
  sleep 2
done
backend_healthy "$API_URL" || { echo "✗ Backend failed to start"; exit 1; }
echo "Using backend: $API_URL"
ensure_auth_stack "$API_URL" "$AUTH_FLAG" || exit 1
API_URL="$(resolve_backend_url)"

frontend_proxy_healthy() {
  local ui_url="$1"
  local backend_url="$2"
  local proxied direct
  proxied="$(curl -sf --max-time 3 "${ui_url}/health" 2>/dev/null || true)"
  direct="$(curl -sf --max-time 3 "${backend_url}/health" 2>/dev/null || true)"
  [[ -n "$proxied" && -n "$direct" ]] \
    && echo "$proxied" | grep -q '"status"' \
    && echo "$direct" | grep -q '"status"' \
    && echo "$proxied" | grep -q "$(echo "$direct" | python3 -c "import json,sys; print(json.load(sys.stdin).get('version',''))" 2>/dev/null || echo __nomatch__)"
}

ensure_frontend_for_backend() {
  local backend_url="$1"
  local port="${FRONTEND_PORT:-3001}"
  local ui_url="http://localhost:${port}"

  if edgequake_ui_port >/dev/null 2>&1; then
    port="$(edgequake_ui_port)"
    ui_url="http://localhost:${port}"
  fi

  if frontend_proxy_healthy "$ui_url" "$backend_url"; then
    echo "✓ Frontend proxy OK at $ui_url → $backend_url"
    echo "$port"
    return 0
  fi

  echo "→ Restarting frontend on :${port} (EDGEQUAKE_API_URL=$backend_url)"
  local fpid
  fpid="$(lsof -nP -iTCP:"${port}" -sTCP:LISTEN -t 2>/dev/null | head -1 || true)"
  if [[ -n "$fpid" ]]; then
    kill "$fpid" 2>/dev/null || true
    sleep 2
  fi
  (cd "$ROOT" && DEV_AUTH_ENABLED="$AUTH_FLAG" \
    make frontend-bg BACKEND_PORT="${backend_url##*:}" FRONTEND_PORT="$port" --no-print-directory)
  for _ in $(seq 1 45); do
    if frontend_proxy_healthy "http://localhost:${port}" "$backend_url"; then
      echo "✓ Frontend proxy ready"
      echo "$port"
      return 0
    fi
    sleep 2
  done
  echo "⚠ Frontend proxy not verified — continuing with Playwright API proxy helpers"
  echo "$port"
  return 0
}

edgequake_ui_port() {
  for p in "$FRONTEND_PORT" 3001 3000; do
    if curl -sf --max-time 3 "http://localhost:${p}/" 2>/dev/null | grep -qi EdgeQuake; then
      echo "$p"
      return 0
    fi
  done
  return 1
}

if ! edgequake_ui_port >/dev/null 2>&1; then
  echo "→ Frontend not reachable; starting via make dev-bg"
  (cd "$ROOT" && DEV_AUTH_ENABLED="$AUTH_FLAG" \
    make dev-bg BACKEND_PORT="${API_URL##*:}" FRONTEND_PORT="$FRONTEND_PORT" --no-print-directory)
fi

UI_PORT="$(ensure_frontend_for_backend "$API_URL" | tail -1)"
UI_PORT="${UI_PORT:-$FRONTEND_PORT}"
UI_URL="http://localhost:${UI_PORT}"
echo "Using frontend: $UI_URL (dev proxy → $API_URL)"

HEALTH="$(curl -sf --max-time 15 "${API_URL}/health")"
echo "$HEALTH" | tee "$PROOF_DIR/002-health-response.json"

chmod +x "$PROOF_DIR/ensure_migration_038.sh"
MIG_REPAIR_OUT="$("$PROOF_DIR/ensure_migration_038.sh" "$HEALTH" 2>&1 | tee /dev/stderr)" || {
  echo "⚠ migration-038 auto-repair failed"
  if [[ "${SPEC020_STRICT_MIGRATION:-0}" == "1" ]]; then
    echo "✗ SPEC020_STRICT_MIGRATION=1 — aborting (migration-038 required)"
    exit 1
  fi
  echo "→ continuing (non-strict QC)"
}
if echo "$MIG_REPAIR_OUT" | grep -q '^REPAIRED=1$'; then
  restart_backend_for_migration_cache "$API_URL" || true
  API_URL="$(resolve_backend_url)"
fi

# Refresh health after migration repair (bootstrap cache may have been stale)
HEALTH="$(curl -sf --max-time 15 "${API_URL}/health")"
echo "$HEALTH" > "$PROOF_DIR/002-health-response.json"
python3 -c "
import json,sys
d=json.load(open('$PROOF_DIR/002-health-response.json'))
idx=d.get('schema',{}).get('source_ids_indexes',{})
out={
  'ready': idx.get('ready', True),
  'missingCount': len(idx.get('missing_indexes',[]) or []),
  'migrationsApplied': d.get('schema',{}).get('migrations_applied',0),
  'latestVersion': d.get('schema',{}).get('latest_version',0),
  'autoMigration': '${SPEC020_AUTO_MIGRATION:-1}',
}
json.dump(out, open('$PROOF_DIR/005-migration038-status.json','w'), indent=2)
" 2>/dev/null || true

echo "→ SPEC-020 Playwright quality control"
set +e
(cd "$WEBUI" && PLAYWRIGHT_SKIP_STACK_CHECK=1 PLAYWRIGHT_BASE_URL="$UI_URL" \
  E2E_LIVE_STACK=1 EQ_BACKEND_URL="$API_URL" EDGEQUAKE_API_URL="$API_URL" \
  E2E_BACKEND_URL="$API_URL" DEV_AUTH_ENABLED="$AUTH_FLAG" \
  SPEC020_STRICT_MIGRATION="${SPEC020_STRICT_MIGRATION:-0}" \
  SPEC020_REQUIRE_OLLAMA="${SPEC020_REQUIRE_OLLAMA:-0}" \
  SPEC020_AUTH_PROOF="${SPEC020_AUTH_PROOF:-0}" \
  NEXT_PUBLIC_AUTH_ENABLED="$AUTH_FLAG" \
  NEXT_PUBLIC_API_URL="$API_URL" \
  bunx playwright test e2e/spec020-quality-control.spec.ts \
  --project=audit --workers=1 --timeout=600000 2>&1 | tee "$LOG")
RC=${PIPESTATUS[0]}
set -e

echo ""
echo "Screenshots:"
ls -la "$SCREENSHOTS"/*.png 2>/dev/null || true

chmod +x "$PROOF_DIR/generate_proof_report.sh"
"$PROOF_DIR/generate_proof_report.sh"

if [[ "$RC" -eq 0 ]]; then
  echo "✓ SPEC-020 quality-control proof passed"
else
  echo "✗ SPEC-020 quality-control proof failed (exit $RC)"
  exit "$RC"
fi
