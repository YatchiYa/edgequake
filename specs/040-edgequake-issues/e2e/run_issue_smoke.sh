#!/usr/bin/env bash
# SPEC-040 — Smoke verification for GitHub issues #250-#253, #259, #262
set -euo pipefail

BACKEND="${BACKEND_URL:-http://localhost:8080}"
FRONTEND="${FRONTEND_URL:-http://localhost:3000}"
WS_ID="${WORKSPACE_ID:-}"

echo "=== SPEC-040 issue smoke ==="

echo "--- #250 Version ---"
API_VER=$(curl -sf "$BACKEND/health" | python3 -c "import sys,json; print(json.load(sys.stdin).get('version','?'))")
echo "API: v${API_VER}"
UI_SNIP=$(curl -sf "$FRONTEND" 2>/dev/null | grep -oE 'UI v[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "UI: (not found)")
echo "$UI_SNIP"

echo "--- #251 Models catalog (ollama count) ---"
curl -sf "$BACKEND/api/v1/models/llm" | python3 -c "
import sys,json
ms=json.load(sys.stdin).get('models',[])
print('ollama models:', sum(1 for m in ms if m.get('provider')=='ollama'))
"

if [[ -n "$WS_ID" ]]; then
  echo "--- #262 Workspace stats ---"
  curl -sf "$BACKEND/api/v1/workspaces/${WS_ID}/stats" | python3 -m json.tool | head -20
fi

echo "--- Done (see specs/040-edgequake-issues/009-cross-reference-matrix.md) ---"
