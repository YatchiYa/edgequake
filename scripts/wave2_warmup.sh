#!/usr/bin/env bash
# SPEC-071 — Warm Wave-2 partial HNSW via admin API (or print instructions).
#
# Usage:
#   ./scripts/wave2_warmup.sh <workspace_uuid> [workspace_uuid...]
#   EDGEQUAKE_API=http://localhost:8080 ./scripts/wave2_warmup.sh <uuid>
#
# Requires backend with admin auth (or EDGEQUAKE_DEV_MODE / open local API).
set -euo pipefail

API="${EDGEQUAKE_API:-http://localhost:8080}"
if [[ "$#" -lt 1 ]]; then
  echo "Usage: $0 <workspace_uuid> [more...]" >&2
  echo "Warmup creates workspace partial HNSW when EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1." >&2
  exit 1
fi

ids=$(printf '%s\n' "$@" | jq -R . | jq -s .)
body=$(jq -n --argjson ids "$ids" '{workspace_ids: $ids}')

echo "POST $API/api/v1/admin/ann/warmup"
curl -sS -X POST "$API/api/v1/admin/ann/warmup" \
  -H "Content-Type: application/json" \
  ${EDGEQUAKE_API_KEY:+-H "Authorization: Bearer $EDGEQUAKE_API_KEY"} \
  -d "$body" | jq .
echo ""
echo "NOTE: /ready checks catalog ANN when Wave-2 is on; first filtered query also warms."
