#!/usr/bin/env bash
# SPEC-124: unfakable live OTLP persist E2E against Langfuse 3.225.5 (current 3.x).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=/dev/null
source "${ROOT}/scripts/langfuse_e2e_common.sh"

PORT="${LANGFUSE_3225_PORT:-3340}"
LF_BASE="${LANGFUSE_3225_UI_URL:-http://localhost:${PORT}}"
PK="${LANGFUSE_3225_PK:-pk-lf-edgequake-3225}"
SK="${LANGFUSE_3225_SK:-sk-lf-edgequake-3225-dev}"
EXPECT_ID="${LANGFUSE_3225_PROJECT_ID:-edgequake-local-3225}"

echo "→ Langfuse 3.225.5 OTLP persist E2E  UI=${LF_BASE}"

code="$(curl -sS -o /tmp/eq-lf3225-health.json -w '%{http_code}' "${LF_BASE}/api/public/health" || true)"
if [ "${code}" != "200" ]; then
  echo "✗ Langfuse 3.225.5 not healthy at ${LF_BASE} (make langfuse-3.225-up)" >&2
  exit 1
fi

python3 - <<'PY'
import json
body = json.load(open("/tmp/eq-lf3225-health.json", encoding="utf-8"))
ver = str(body.get("version") or "")
parts = ver.split(".")
if len(parts) < 2 or parts[0] != "3" or parts[1] != "225":
    raise SystemExit(f"unfakable pin failed: expected Langfuse 3.225.x, got {ver!r}")
print(f"✓ Langfuse version={ver}")
PY

langfuse_smoke "${LF_BASE}" "${PK}" "${SK}" "${EXPECT_ID}"
langfuse_assert_otlp_exists "${LF_BASE}" "${PK}" "${SK}"

cd "${ROOT}/edgequake"
export LANGFUSE_OTLP_E2E=1
export LANGFUSE_OTLP_E2E_BASE="${LF_BASE}"
export LANGFUSE_OTLP_E2E_PIN=3.225
export LANGFUSE_OTLP_E2E_MIN=3.22.0
unset LANGFUSE_OTLP_E2E_PERSIST || true
unset LANGFUSE_OTLP_E2E_CLOUD || true
export LANGFUSE_BASE_URL="${LF_BASE}"
export LANGFUSE_PUBLIC_KEY="${PK}"
export LANGFUSE_SECRET_KEY="${SK}"
cargo test -p edgequake-observability --lib live_langfuse_otlp_roundtrip -- --nocapture

echo "✓ spec124-langfuse-3.225-e2e passed"
