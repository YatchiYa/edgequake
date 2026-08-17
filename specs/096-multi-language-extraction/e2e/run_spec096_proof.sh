#!/usr/bin/env bash
# SPEC-096 curl proof: workspace extraction_language round-trip.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ARTIFACT_DIR="$(cd "$(dirname "$0")" && pwd)/artifacts"
mkdir -p "$ARTIFACT_DIR"
NOTES="$ARTIFACT_DIR/RUN_NOTES.md"

BASE_URL="${EDGEQUAKE_API_URL:-http://localhost:8080}"
API="$BASE_URL/api/v1"

pass=0
fail=0
log() { echo "$*" | tee -a "$NOTES"; }

: > "$NOTES"
log "# SPEC-096 Proof Run"
log ""
log "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
log "API: $API"
log ""

if ! curl -sf "$BASE_URL/health" >/dev/null; then
  log "FAIL: backend not healthy at $BASE_URL/health"
  exit 1
fi
log "PASS: backend healthy"
pass=$((pass + 1))

SLUG="spec096-proof-$(date +%s)"
TENANT=$(curl -sf -X POST "$API/tenants" \
  -H 'Content-Type: application/json' \
  -d "{\"name\":\"SPEC-096 Proof\",\"slug\":\"$SLUG\",\"plan\":\"pro\"}")
TENANT_ID=$(echo "$TENANT" | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
log "Tenant: $TENANT_ID"

WS=$(curl -sf -X POST "$API/tenants/$TENANT_ID/workspaces" \
  -H 'Content-Type: application/json' \
  -d "{\"name\":\"Chinese KG\",\"slug\":\"zh-$SLUG\",\"extraction_language\":\"Chinese\"}")
LANG=$(echo "$WS" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("extraction_language"))')
WS_ID=$(echo "$WS" | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')

if [[ "$LANG" == "Chinese" ]]; then
  log "PASS: create returns extraction_language=Chinese"
  pass=$((pass + 1))
else
  log "FAIL: create expected Chinese, got $LANG"
  fail=$((fail + 1))
fi

GET=$(curl -sf -H "X-Tenant-ID: $TENANT_ID" "$API/workspaces/$WS_ID")
GET_LANG=$(echo "$GET" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("extraction_language"))')
if [[ "$GET_LANG" == "Chinese" ]]; then
  log "PASS: GET returns extraction_language=Chinese"
  pass=$((pass + 1))
else
  log "FAIL: GET expected Chinese, got $GET_LANG"
  fail=$((fail + 1))
fi

BAD=$(curl -s -o /tmp/spec096_bad.json -w '%{http_code}' -X POST "$API/tenants/$TENANT_ID/workspaces" \
  -H 'Content-Type: application/json' \
  -d "{\"name\":\"Bad\",\"slug\":\"bad-$SLUG\",\"extraction_language\":\"Klingon\"}")
if [[ "$BAD" == "400" ]]; then
  log "PASS: unsupported language rejected with HTTP 400"
  pass=$((pass + 1))
else
  log "FAIL: expected HTTP 400 for Klingon, got $BAD"
  fail=$((fail + 1))
fi

log ""
log "## Summary"
log "PASS=$pass FAIL=$fail"
if [[ "$fail" -gt 0 ]]; then
  exit 1
fi
log "ALL GREEN"
