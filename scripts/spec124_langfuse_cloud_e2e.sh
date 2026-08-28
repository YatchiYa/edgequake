#!/usr/bin/env bash
# SPEC-124: unfakable live OTLP E2E against current Langfuse Cloud.
# Requires LANGFUSE_PUBLIC_KEY + LANGFUSE_SECRET_KEY + a Cloud LANGFUSE_BASE_URL
# (typically from repo-root .env). Never prints key material.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=/dev/null
source "${ROOT}/scripts/langfuse_e2e_common.sh"

if [ -f "${ROOT}/.env" ]; then
  set -a
  # shellcheck disable=SC1091
  source "${ROOT}/.env"
  set +a
fi

LF_BASE="${LANGFUSE_BASE_URL:-${LANGFUSE_HOST:-}}"
LF_BASE="${LF_BASE%%/}"
PK="${LANGFUSE_PUBLIC_KEY:-}"
SK="${LANGFUSE_SECRET_KEY:-}"

if [ -z "${LF_BASE}" ] || [ -z "${PK}" ] || [ -z "${SK}" ]; then
  echo "✗ Cloud E2E requires LANGFUSE_BASE_URL + LANGFUSE_PUBLIC_KEY + LANGFUSE_SECRET_KEY" >&2
  echo "  Set them in repo-root .env (or the environment). This test does not mock Cloud." >&2
  exit 1
fi

python3 - "${LF_BASE}" <<'PY'
import sys
from urllib.parse import urlparse
base = sys.argv[1].strip().strip('"').strip("'")
host = (urlparse(base).hostname or "").lower()
allowed = {
    "cloud.langfuse.com",
    "us.cloud.langfuse.com",
    "jp.cloud.langfuse.com",
    "hipaa.cloud.langfuse.com",
}
if host not in allowed:
    raise SystemExit(f"unfakable Cloud pin failed: host {host!r} is not Langfuse Cloud")
print(f"→ Langfuse Cloud OTLP E2E  host={host}")
PY

code="$(curl -sS -o /tmp/eq-lf-cloud-health.json -w '%{http_code}' "${LF_BASE}/api/public/health" || true)"
if [ "${code}" != "200" ]; then
  echo "✗ Langfuse Cloud not healthy at host (HTTP ${code})" >&2
  exit 1
fi
langfuse_print_health_version /tmp/eq-lf-cloud-health.json

python3 - /tmp/eq-lf-cloud-health.json <<'PY'
import json, sys
body = json.load(open(sys.argv[1], encoding="utf-8"))
ver = str(body.get("version") or "")
# OTLP floor is 3.22.0; Cloud is currently 4.x.
core = ver.split("-")[0].split("+")[0]
parts = [int(p) for p in core.split(".")[:3] + ["0", "0"] ][:3]
if tuple(parts) < (3, 22, 0):
    raise SystemExit(f"Cloud version {ver!r} is below OTLP floor 3.22.0")
print(f"✓ Cloud version {ver} ≥ 3.22.0 (OTLP floor)")
PY

code="$(curl -sS -u "${PK}:${SK}" -o /tmp/eq-lf-cloud-projects.json -w '%{http_code}' \
  "${LF_BASE}/api/public/projects" || true)"
if [ "${code}" != "200" ]; then
  echo "✗ GET /api/public/projects HTTP ${code} (keys rejected or Cloud down)" >&2
  exit 1
fi
python3 - <<'PY'
import json
body = json.load(open("/tmp/eq-lf-cloud-projects.json", encoding="utf-8"))
rows = body.get("data") or []
ids = [r.get("id") for r in rows if isinstance(r, dict)]
if not ids:
    raise SystemExit("Cloud keys authenticated but project list is empty")
print(f"✓ Cloud projects API ok count={len(ids)}")
PY

langfuse_assert_otlp_exists "${LF_BASE}" "${PK}" "${SK}"

cd "${ROOT}/edgequake"
export LANGFUSE_OTLP_E2E=1
export LANGFUSE_OTLP_E2E_BASE="${LF_BASE}"
export LANGFUSE_OTLP_E2E_CLOUD=1
export LANGFUSE_OTLP_E2E_MIN=3.22.0
unset LANGFUSE_OTLP_E2E_PIN || true
export LANGFUSE_BASE_URL="${LF_BASE}"
export LANGFUSE_PUBLIC_KEY="${PK}"
export LANGFUSE_SECRET_KEY="${SK}"
cargo test -p edgequake-observability --lib live_langfuse_otlp_roundtrip -- --nocapture

echo "✓ spec124-langfuse-cloud-e2e passed"
