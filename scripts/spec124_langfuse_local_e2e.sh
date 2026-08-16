#!/usr/bin/env bash
# SPEC-124: live E2E against local Langfuse Docker + EdgeQuake backend.
# Requires: make langfuse-up, backend with LANGFUSE_BASE_URL=http://localhost:3310
#           and the Compose init keys, frontend up.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck disable=SC1091
set -a
[ -f "${ROOT}/.edgequake-dev-ports.env" ] && . "${ROOT}/.edgequake-dev-ports.env"
set +a

PORT="${LANGFUSE_PORT:-3310}"
LF_BASE="${LANGFUSE_UI_URL:-http://localhost:${PORT}}"
BACKEND_URL="${BACKEND_URL:-http://localhost:${BACKEND_PORT:-8090}}"
FRONTEND_URL="${FRONTEND_URL:-http://localhost:${FRONTEND_PORT:-3010}}"
PK="${LANGFUSE_LOCAL_PK:-pk-lf-edgequake-local}"
SK="${LANGFUSE_LOCAL_SK:-sk-lf-edgequake-local-dev}"
EXPECT_ID="${LANGFUSE_LOCAL_PROJECT_ID:-edgequake-local}"

echo "→ Langfuse E2E  UI=${LF_BASE}  API=${BACKEND_URL}  Web=${FRONTEND_URL}"

chmod +x "${ROOT}/scripts/langfuse_local_smoke.sh"
LANGFUSE_PORT="${PORT}" LANGFUSE_UI_URL="${LF_BASE}" \
  LANGFUSE_LOCAL_PK="${PK}" LANGFUSE_LOCAL_SK="${SK}" \
  LANGFUSE_LOCAL_PROJECT_ID="${EXPECT_ID}" \
  "${ROOT}/scripts/langfuse_local_smoke.sh"

if ! curl -sf "${BACKEND_URL}/health" >/dev/null; then
  echo "✗ EdgeQuake backend not healthy at ${BACKEND_URL} (make backend-bg / make dev-bg)" >&2
  exit 1
fi

if ! curl -sf "${FRONTEND_URL}/" 2>/dev/null | grep -qi EdgeQuake; then
  echo "✗ Frontend not EdgeQuake at ${FRONTEND_URL}" >&2
  exit 1
fi

curl -sf "${BACKEND_URL}/api/v1/settings/langfuse" > /tmp/eq-langfuse-settings.json
python3 - "${LF_BASE}" "${EXPECT_ID}" <<'PY'
import json, sys
with open("/tmp/eq-langfuse-settings.json", encoding="utf-8") as f:
    body = json.load(f)
lf_base = sys.argv[1].rstrip("/")
expect_id = sys.argv[2]
ui = (body.get("ui_url") or body.get("base_url") or "").rstrip("/")
if ui != lf_base:
    raise SystemExit(
        f"backend ui_url={ui!r} is not local Langfuse {lf_base!r}. "
        "Set LANGFUSE_BASE_URL + init keys in .env and restart the backend."
    )
if not body.get("export_active"):
    raise SystemExit("export_active=false — keys missing or otel feature off")
pid = body.get("project_id")
if pid != expect_id:
    raise SystemExit(f"project_id={pid!r} expected {expect_id!r}")
purl = (body.get("project_ui_url") or "").rstrip("/")
want = f"{lf_base}/project/{expect_id}"
if purl != want:
    raise SystemExit(f"project_ui_url={purl!r} expected {want!r}")
print(f"✓ settings langfuse ui_url={ui} project_id={pid}")
PY

cd "${ROOT}/edgequake_webui"
export E2E_LIVE_STACK=1
export PLAYWRIGHT_BASE_URL="${FRONTEND_URL}"
export EQ_BACKEND_URL="${BACKEND_URL}"
export LANGFUSE_PUBLIC_KEY="${PK}"
export LANGFUSE_SECRET_KEY="${SK}"
export LANGFUSE_BASE_URL="${LF_BASE}"
export LANGFUSE_PROJECT_ID="${EXPECT_ID}"
pnpm exec playwright test \
  e2e/spec124-langfuse-settings.spec.ts \
  e2e/spec124-langfuse-sessions.spec.ts \
  --project=chromium --reporter=line

echo "✓ spec124-langfuse-e2e passed"
