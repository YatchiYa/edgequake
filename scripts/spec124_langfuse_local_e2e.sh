#!/usr/bin/env bash
# SPEC-124: live E2E against local Langfuse Docker + EdgeQuake backend.
# Preferred: make spec124-langfuse-e2e (starts Langfuse + stack with init keys).
# Manual: make dev-bg-langfuse, then this script.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=/dev/null
source "${ROOT}/scripts/langfuse_e2e_common.sh"
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
  echo "✗ EdgeQuake backend not healthy at ${BACKEND_URL} (make dev-bg-langfuse / make spec124-langfuse-e2e)" >&2
  exit 1
fi

if ! curl -sf "${FRONTEND_URL}/" 2>/dev/null | grep -qi EdgeQuake; then
  echo "✗ Frontend not EdgeQuake at ${FRONTEND_URL}" >&2
  exit 1
fi

ready_fe=0
for _ in $(seq 1 30); do
  html="$(curl -sf "${FRONTEND_URL}/settings" 2>/dev/null || true)"
  if printf '%s' "${html}" | grep -qiE 'EdgeQuake|<main|langfuse'; then
    ready_fe=1
    break
  fi
  sleep 2
done
if [ "${ready_fe}" != "1" ]; then
  echo "✗ Frontend /settings did not become ready at ${FRONTEND_URL}" >&2
  exit 1
fi

langfuse_verify_settings_dto "${BACKEND_URL}" "${LF_BASE}" "${EXPECT_ID}"

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
