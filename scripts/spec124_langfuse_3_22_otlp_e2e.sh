#!/usr/bin/env bash
# SPEC-124: unfakable live OTLP E2E against Langfuse 3.22.0 (first OTLP release).
# Starts (or reuses) docker-compose.langfuse-3.22.yml, then runs the Rust live test.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=/dev/null
source "${ROOT}/scripts/langfuse_e2e_common.sh"

PORT="${LANGFUSE_322_PORT:-3330}"
LF_BASE="${LANGFUSE_322_UI_URL:-http://localhost:${PORT}}"
PK="${LANGFUSE_322_PK:-pk-lf-edgequake-322}"
SK="${LANGFUSE_322_SK:-sk-lf-edgequake-322-dev}"
EXPECT_ID="${LANGFUSE_322_PROJECT_ID:-edgequake-local-322}"

echo "→ Langfuse 3.22.0 OTLP E2E  UI=${LF_BASE}"

code="$(curl -sS -o /tmp/eq-lf322-health.json -w '%{http_code}' "${LF_BASE}/api/public/health" || true)"
if [ "${code}" != "200" ]; then
  echo "✗ Langfuse 3.22.0 not healthy at ${LF_BASE} (make langfuse-3.22-up)" >&2
  exit 1
fi

python3 - <<'PY'
import json
body = json.load(open("/tmp/eq-lf322-health.json", encoding="utf-8"))
ver = str(body.get("version") or "")
parts = ver.split(".")
if len(parts) < 2 or parts[0] != "3" or parts[1] != "22":
    raise SystemExit(f"unfakable pin failed: expected Langfuse 3.22.x, got {ver!r}")
print(f"✓ Langfuse version={ver}")
PY

langfuse_smoke "${LF_BASE}" "${PK}" "${SK}" "${EXPECT_ID}"
langfuse_assert_otlp_exists "${LF_BASE}" "${PK}" "${SK}"

cd "${ROOT}/edgequake"
export LANGFUSE_OTLP_E2E=1
export LANGFUSE_OTLP_E2E_BASE="${LF_BASE}"
export LANGFUSE_OTLP_E2E_PIN=3.22
export LANGFUSE_OTLP_E2E_MIN=3.22.0
# 3.22.0 added the OTLP route (not 404 + auto-probe=Otlp) but its first-release
# protobuf parser raises "Invalid time value" on current OTEL timestamps.
# Persist is proven on 3.225.5 / Cloud — not this tag.
export LANGFUSE_OTLP_E2E_PERSIST=0
unset LANGFUSE_OTLP_E2E_CLOUD || true
export LANGFUSE_BASE_URL="${LF_BASE}"
export LANGFUSE_PUBLIC_KEY="${PK}"
export LANGFUSE_SECRET_KEY="${SK}"
cargo test -p edgequake-observability --lib live_langfuse_otlp_roundtrip -- --nocapture

echo "✓ spec124-langfuse-3.22-e2e passed (route+probe; persist is 3.225.5 / Cloud)"
