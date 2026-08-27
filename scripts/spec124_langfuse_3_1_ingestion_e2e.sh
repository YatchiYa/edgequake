#!/usr/bin/env bash
# SPEC-124: unfakable live E2E of the native ingestion fallback against Langfuse 3.1.1.
# Starts (or reuses) docker-compose.langfuse-3.1.yml, then runs the Rust live test.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=/dev/null
source "${ROOT}/scripts/langfuse_e2e_common.sh"

PORT="${LANGFUSE_311_PORT:-3320}"
LF_BASE="${LANGFUSE_311_UI_URL:-http://localhost:${PORT}}"
PK="${LANGFUSE_311_PK:-pk-lf-edgequake-311}"
SK="${LANGFUSE_311_SK:-sk-lf-edgequake-311-dev}"
EXPECT_ID="${LANGFUSE_311_PROJECT_ID:-edgequake-local-311}"

echo "→ Langfuse 3.1.1 ingestion E2E  UI=${LF_BASE}"

code="$(curl -sS -o /tmp/eq-lf311-health.json -w '%{http_code}' "${LF_BASE}/api/public/health" || true)"
if [ "${code}" != "200" ]; then
  echo "✗ Langfuse 3.1.1 not healthy at ${LF_BASE} (make langfuse-3.1-up)" >&2
  exit 1
fi

python3 - <<'PY'
import json
body = json.load(open("/tmp/eq-lf311-health.json", encoding="utf-8"))
ver = str(body.get("version") or "")
if not ver.startswith("3.1."):
    raise SystemExit(f"unfakable pin failed: expected Langfuse 3.1.x, got {ver!r}")
print(f"✓ Langfuse version={ver}")
PY

langfuse_smoke "${LF_BASE}" "${PK}" "${SK}" "${EXPECT_ID}"

otlp="$(curl -sS -o /tmp/eq-lf311-otlp.txt -w '%{http_code}' \
  -X POST "${LF_BASE}/api/public/otel/v1/traces" \
  -u "${PK}:${SK}" \
  -H "Content-Type: application/x-protobuf" \
  --data-binary '' || true)"
if [ "${otlp}" != "404" ]; then
  echo "✗ OTLP probe expected 404 on 3.1.1, got HTTP ${otlp}" >&2
  cat /tmp/eq-lf311-otlp.txt >&2 || true
  exit 1
fi
echo "✓ OTLP /api/public/otel/v1/traces → 404 (3.1.1 has no OTLP)"

cd "${ROOT}/edgequake"
export LANGFUSE_311_E2E=1
export LANGFUSE_311_E2E_BASE="${LF_BASE}"
export LANGFUSE_BASE_URL="${LF_BASE}"
export LANGFUSE_PUBLIC_KEY="${PK}"
export LANGFUSE_SECRET_KEY="${SK}"
cargo test -p edgequake-observability --lib live_langfuse_3_1_1_ingestion_roundtrip -- --nocapture

echo "✓ spec124-langfuse-3.1-e2e passed"
