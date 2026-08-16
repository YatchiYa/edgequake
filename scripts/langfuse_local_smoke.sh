#!/usr/bin/env bash
# SPEC-124: smoke local Langfuse Docker (no EdgeQuake required).
set -euo pipefail

PORT="${LANGFUSE_PORT:-3310}"
BASE="${LANGFUSE_UI_URL:-http://localhost:${PORT}}"
PK="${LANGFUSE_LOCAL_PK:-pk-lf-edgequake-local}"
SK="${LANGFUSE_LOCAL_SK:-sk-lf-edgequake-local-dev}"
EXPECT_ID="${LANGFUSE_LOCAL_PROJECT_ID:-edgequake-local}"

echo "→ Langfuse smoke ${BASE}"

code="$(curl -sS -o /tmp/eq-langfuse-health.json -w '%{http_code}' "${BASE}/api/public/health" || true)"
if [ "${code}" != "200" ]; then
  echo "✗ health HTTP ${code} — run: make langfuse-up" >&2
  exit 1
fi

code="$(curl -sS -o /tmp/eq-langfuse-ready.json -w '%{http_code}' "${BASE}/api/public/ready" || true)"
if [ "${code}" != "200" ]; then
  echo "✗ ready HTTP ${code}" >&2
  exit 1
fi

code="$(curl -sS -u "${PK}:${SK}" -o /tmp/eq-langfuse-projects.json -w '%{http_code}' \
  "${BASE}/api/public/projects" || true)"
if [ "${code}" != "200" ]; then
  echo "✗ GET /api/public/projects HTTP ${code}" >&2
  cat /tmp/eq-langfuse-projects.json 2>/dev/null || true
  exit 1
fi

python3 - "${EXPECT_ID}" <<'PY'
import json, sys
expect = sys.argv[1]
with open("/tmp/eq-langfuse-projects.json", encoding="utf-8") as f:
    body = json.load(f)
rows = body.get("data") or []
ids = [r.get("id") for r in rows if isinstance(r, dict)]
if expect not in ids:
    raise SystemExit(f"expected project id {expect!r} in {ids!r}")
print(f"✓ projects API id={expect}")
PY

echo "✓ Langfuse local smoke passed (${BASE})"
