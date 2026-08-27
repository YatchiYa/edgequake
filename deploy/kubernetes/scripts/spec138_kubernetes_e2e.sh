#!/usr/bin/env bash
# SPEC-138 — full Kubernetes E2E proof (kind + Langfuse trace delivery).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
K8S_SCRIPTS="${ROOT}/deploy/kubernetes/scripts"
MEAS="${ROOT}/specs/138-kubernetes/measurements"
# shellcheck source=/dev/null
source "${ROOT}/scripts/langfuse_e2e_common.sh"
# shellcheck source=/dev/null
source "${K8S_SCRIPTS}/k8s_context.sh"

mkdir -p "${MEAS}"

PK="${LANGFUSE_PUBLIC_KEY:-pk-lf-edgequake-k8s}"
SK="${LANGFUSE_SECRET_KEY:-sk-lf-edgequake-k8s-dev}"
PROJECT_ID="${LANGFUSE_PROJECT_ID:-edgequake-k8s}"
API_PORT="${SPEC138_API_PORT:-18080}"
WEB_PORT="${SPEC138_WEB_PORT:-13000}"
LF_PORT="${SPEC138_LANGFUSE_PORT:-13310}"
BACKEND_URL="http://127.0.0.1:${API_PORT}"
FRONTEND_URL="http://127.0.0.1:${WEB_PORT}"
LF_BASE="http://127.0.0.1:${LF_PORT}"

cleanup_pf() {
  local pids
  pids=$(jobs -p 2>/dev/null || true)
  if [ -n "${pids}" ]; then
    kill ${pids} 2>/dev/null || true
  fi
}
trap cleanup_pf EXIT

log_gate() {
  local id="$1" file="$2"
  shift 2
  echo "=== ${id} ===" | tee "${MEAS}/${file}"
  "$@" 2>&1 | tee -a "${MEAS}/${file}"
  echo "✓ ${id}" | tee -a "${MEAS}/${file}"
}

echo "SPEC-138 Kubernetes proof — $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee "${MEAS}/SUMMARY.md"

log_gate "E2E-138-01" "e2e138-prereqs.txt" bash "${K8S_SCRIPTS}/k8s_prereqs.sh"
log_gate "E2E-138-02" "e2e138-kind.txt" bash "${K8S_SCRIPTS}/k8s_kind_up.sh"
log_gate "E2E-138-03" "e2e138-helm-install.txt" bash "${K8S_SCRIPTS}/k8s_install_stack.sh"
log_gate "E2E-138-03b" "e2e138-helm-install.txt" bash "${K8S_SCRIPTS}/k8s_wait_ready.sh"

# Port-forwards for localhost tests
k port-forward -n edgequake svc/edgequake-api "${API_PORT}:8080" >/tmp/pf-api.log 2>&1 &
k port-forward -n edgequake svc/edgequake-web "${WEB_PORT}:3000" >/tmp/pf-web.log 2>&1 &
k port-forward -n langfuse svc/langfuse-web "${LF_PORT}:3000" >/tmp/pf-lf.log 2>&1 &
sleep 3

{
  echo "=== E2E-138-04 postgres ==="
  k exec -n edgequake statefulset/edgequake-postgres -- \
    pg_isready -U edgequake -d edgequake
  echo "✓ E2E-138-04"
} 2>&1 | tee "${MEAS}/e2e138-postgres.txt"

{
  echo "=== E2E-138-05 api ready ==="
  curl -sf "${BACKEND_URL}/ready"
  echo "✓ E2E-138-05"
} 2>&1 | tee "${MEAS}/e2e138-api-ready.txt"

{
  echo "=== E2E-138-06 web ==="
  curl -sf "${FRONTEND_URL}/" -o /dev/null
  echo "✓ E2E-138-06"
} 2>&1 | tee "${MEAS}/e2e138-web.txt"

{
  echo "=== E2E-138-07 langfuse smoke ==="
  langfuse_smoke "${LF_BASE}" "${PK}" "${SK}" "${PROJECT_ID}"
  echo "✓ E2E-138-07"
} 2>&1 | tee "${MEAS}/e2e138-langfuse-smoke.txt"

{
  echo "=== E2E-138-08 settings dto ==="
  langfuse_verify_settings_dto "${BACKEND_URL}" "${LF_BASE}" "${PROJECT_ID}" "0"
  echo "✓ E2E-138-08"
} 2>&1 | tee "${MEAS}/e2e138-settings-dto.txt"

SESSION_ID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
{
  echo "=== E2E-138-09 trace delivery session=${SESSION_ID} ==="
  edgequake_query_with_session "${BACKEND_URL}" "${SESSION_ID}"
  langfuse_poll_session_observations "${LF_BASE}" "${PK}" "${SK}" "${SESSION_ID}" 15
  echo "✓ E2E-138-09"
} 2>&1 | tee "${MEAS}/e2e138-trace-delivery.txt"

{
  echo "=== E2E-138-10/11 playwright ==="
  cd "${ROOT}/edgequake_webui"
  export E2E_LIVE_STACK=1
  export PLAYWRIGHT_BASE_URL="${FRONTEND_URL}"
  export EQ_BACKEND_URL="${BACKEND_URL}"
  export LANGFUSE_PUBLIC_KEY="${PK}"
  export LANGFUSE_SECRET_KEY="${SK}"
  export LANGFUSE_BASE_URL="${LF_BASE}"
  export LANGFUSE_PROJECT_ID="${PROJECT_ID}"
  pnpm exec playwright test \
    e2e/spec124-langfuse-settings.spec.ts \
    e2e/spec124-langfuse-sessions.spec.ts \
    --project=chromium --reporter=line
  echo "✓ E2E-138-10/11"
} 2>&1 | tee "${MEAS}/e2e138-playwright.txt"

{
  echo "=== E2E-138-15 helm test ==="
  helm test edgequake-stack --kube-context "${KUBECTL_CONTEXT}" -n edgequake --timeout 5m
  echo "✓ E2E-138-15"
} 2>&1 | tee "${MEAS}/e2e138-helm-test.txt"

{
  echo "=== E2E-138-14 pod restart ==="
  k delete pod -n edgequake -l app.kubernetes.io/component=api --wait=true
  k wait --for=condition=Ready pod -l app.kubernetes.io/component=api -n edgequake --timeout=300s
  sleep 5
  SESSION2="$(python3 -c 'import uuid; print(uuid.uuid4())')"
  edgequake_query_with_session "${BACKEND_URL}" "${SESSION2}"
  langfuse_poll_session_observations "${LF_BASE}" "${PK}" "${SK}" "${SESSION2}" 15
  echo "✓ E2E-138-14"
} 2>&1 | tee -a "${MEAS}/e2e138-trace-delivery.txt"

echo "| Gate | Status |" >> "${MEAS}/SUMMARY.md"
echo "|------|--------|" >> "${MEAS}/SUMMARY.md"
for f in e2e138-*.txt; do
  echo "| ${f%.txt} | pass |" >> "${MEAS}/SUMMARY.md"
done

echo "✓ spec138-kubernetes-proof complete — artifacts in ${MEAS}"
