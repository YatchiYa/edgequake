#!/usr/bin/env bash
# SPEC-124 / SPEC-138 — shared Langfuse E2E helpers (DRY).
# Source from spec124_langfuse_local_e2e.sh and spec138_kubernetes_e2e.sh.

langfuse_smoke() {
  local base="${1:?LANGFUSE base URL required}"
  local pk="${2:?public key required}"
  local sk="${3:?secret key required}"
  local expect_id="${4:?project id required}"

  echo "→ Langfuse smoke ${base}"

  local code
  code="$(curl -sS -o /tmp/eq-langfuse-health.json -w '%{http_code}' "${base}/api/public/health" || true)"
  if [ "${code}" != "200" ]; then
    echo "✗ health HTTP ${code}" >&2
    return 1
  fi

  code="$(curl -sS -o /tmp/eq-langfuse-ready.json -w '%{http_code}' "${base}/api/public/ready" || true)"
  if [ "${code}" != "200" ]; then
    echo "✗ ready HTTP ${code}" >&2
    return 1
  fi

  code="$(curl -sS -u "${pk}:${sk}" -o /tmp/eq-langfuse-projects.json -w '%{http_code}' \
    "${base}/api/public/projects" || true)"
  if [ "${code}" != "200" ]; then
    echo "✗ GET /api/public/projects HTTP ${code}" >&2
    cat /tmp/eq-langfuse-projects.json 2>/dev/null || true
    return 1
  fi

  python3 - "${expect_id}" <<'PY'
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
  echo "✓ Langfuse smoke passed (${base})"
}

langfuse_verify_settings_dto() {
  local backend_url="${1:?backend URL}"
  local lf_base="${2:?langfuse base}"
  local expect_id="${3:?project id}"
  local strict_ui="${4:-1}"

  curl -sf "${backend_url}/api/v1/settings/langfuse" > /tmp/eq-langfuse-settings.json
  python3 - "${lf_base}" "${expect_id}" "${strict_ui}" <<'PY'
import json, sys
raw = open("/tmp/eq-langfuse-settings.json", encoding="utf-8").read()
if __import__("re").search(r"sk-lf-[A-Za-z0-9]", raw):
    raise SystemExit("settings JSON leaked a Langfuse secret (sk-lf-)")
body = json.loads(raw)
lf_base = sys.argv[1].rstrip("/")
expect_id = sys.argv[2]
strict_ui = sys.argv[3] == "1"
ui = (body.get("ui_url") or body.get("base_url") or "").rstrip("/")
if strict_ui and ui != lf_base.rstrip("/"):
    raise SystemExit(f"backend ui_url={ui!r} expected {lf_base!r}")
if not strict_ui and not ui:
    raise SystemExit("ui_url empty")
if not body.get("export_active"):
    raise SystemExit("export_active=false")
pid = body.get("project_id")
if pid != expect_id:
    raise SystemExit(f"project_id={pid!r} expected {expect_id!r}")
print(f"✓ settings langfuse ui_url={ui} project_id={pid}")
PY
}

langfuse_poll_session_observations() {
  local base="${1:?langfuse base}"
  local pk="${2:?pk}"
  local sk="${3:?sk}"
  local session_id="${4:?session id}"
  local max_attempts="${5:-12}"

  local auth filter url
  auth=$(printf '%s:%s' "$pk" "$sk" | base64 | tr -d '\n')
  filter=$(python3 - "$session_id" <<'PY'
import json, sys
print(json.dumps([{"type":"string","column":"sessionId","operator":"=","value":sys.argv[1]}]))
PY
)

  local found=0 attempt
  for attempt in $(seq 1 "$max_attempts"); do
    sleep 2
    url="${base%/}/api/public/v2/observations?filter=$(python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1]))" "$filter")&limit=20"
    code=$(curl -sS -o /tmp/eq-lf-obs.json -w '%{http_code}' \
      -H "Authorization: Basic ${auth}" "$url" || true)
    if [ "$code" = "200" ]; then
      count=$(python3 - <<'PY'
import json
body=json.load(open("/tmp/eq-lf-obs.json"))
print(len(body.get("data") or []))
PY
)
      if [ "$count" -ge 1 ]; then
        found=1
        break
      fi
    fi
    legacy_code=$(curl -sS -o /dev/null -w '%{http_code}' \
      -H "Authorization: Basic ${auth}" \
      "${base%/}/api/public/sessions/${session_id}" || true)
    if [ "$legacy_code" = "200" ]; then
      found=1
      break
    fi
  done

  if [ "$found" != "1" ]; then
    echo "✗ no observations for session_id=${session_id}" >&2
    return 1
  fi
  echo "✓ Langfuse observations found for session_id=${session_id}"
}

# Unfakable: OTLP path must exist (not 404). Empty protobuf is fine — 400/401/200 all prove the route.
langfuse_assert_otlp_exists() {
  local base="${1:?LANGFUSE base URL required}"
  local pk="${2:?public key required}"
  local sk="${3:?secret key required}"
  local otlp
  otlp="$(curl -sS -o /tmp/eq-lf-otlp-probe.txt -w '%{http_code}' \
    -X POST "${base%/}/api/public/otel/v1/traces" \
    -u "${pk}:${sk}" \
    -H "Content-Type: application/x-protobuf" \
    --data-binary '' || true)"
  if [ "${otlp}" = "404" ]; then
    echo "✗ OTLP /api/public/otel/v1/traces → 404 (need Langfuse ≥ 3.22 / Cloud)" >&2
    cat /tmp/eq-lf-otlp-probe.txt >&2 || true
    return 1
  fi
  echo "✓ OTLP /api/public/otel/v1/traces → HTTP ${otlp} (not 404)"
}

langfuse_print_health_version() {
  local file="${1:?health json file}"
  python3 - "$file" <<'PY'
import json, sys
body = json.load(open(sys.argv[1], encoding="utf-8"))
ver = str(body.get("version") or "")
if not ver:
    raise SystemExit("health.version empty")
print(f"✓ Langfuse version={ver}")
PY
}

edgequake_query_with_session() {
  local backend_url="${1:?backend}"
  local session_id="${2:?session}"
  local tenant="${3:-00000000-0000-0000-0000-000000000002}"
  local workspace="${4:-00000000-0000-0000-0000-000000000003}"
  local user="${5:-00000000-0000-0000-0000-000000000001}"

  for turn in 1 2; do
    code=$(curl -sS -o /tmp/eq-query.json -w '%{http_code}' \
      -X POST "${backend_url}/api/v1/query" \
      -H "Content-Type: application/json" \
      -H "X-Tenant-ID: ${tenant}" \
      -H "X-Workspace-ID: ${workspace}" \
      -H "X-User-ID: ${user}" \
      -d "{\"query\":\"spec138 k8s turn ${turn}\",\"mode\":\"naive\",\"session_id\":\"${session_id}\"}" || true)
    if [ "$code" != "200" ]; then
      echo "✗ query HTTP ${code}: $(cat /tmp/eq-query.json 2>/dev/null)" >&2
      return 1
    fi
  done
  echo "✓ queries sent session_id=${session_id}"
}
