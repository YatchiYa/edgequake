#!/usr/bin/env bash
# SPEC-138 E2E-138-01 — cluster prerequisites (cert-manager + ClickHouse.com operator + nginx).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
# shellcheck source=/dev/null
source "$(dirname "$0")/k8s_context.sh"
K8S_DIR="${ROOT}/deploy/kubernetes"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "✗ required: $1" >&2; exit 1; }
}

need kubectl
need helm

helm version --short | grep -qE 'v(3\.(1[7-9]|[2-9][0-9])|4\.)' || {
  echo "✗ Helm >= 3.17 or v4.x required (Langfuse chart)" >&2
  exit 1
}

echo "→ kubectl context: ${KUBECTL_CONTEXT}"

echo "→ Installing cert-manager (if missing)..."
if ! k get crd certificates.cert-manager.io >/dev/null 2>&1; then
  helm install cert-manager oci://quay.io/jetstack/charts/cert-manager \
    --kube-context "${KUBECTL_CONTEXT}" \
    --version v1.14.4 \
    --namespace cert-manager --create-namespace \
    --set crds.enabled=true
  k wait --for=condition=Established crd/certificates.cert-manager.io --timeout=120s
  k wait --for=condition=Established crd/issuers.cert-manager.io --timeout=120s
  k wait --for=condition=Available deployment/cert-manager -n cert-manager --timeout=300s
fi

echo "→ Installing ClickHouse.com operator (Langfuse v2 chart requirement)..."
if ! k get crd clickhouseclusters.clickhouse.com >/dev/null 2>&1; then
  helm install clickhouse-operator oci://ghcr.io/clickhouse/clickhouse-operator-helm \
    --kube-context "${KUBECTL_CONTEXT}" \
    --version 0.0.5 \
    --namespace clickhouse-operator --create-namespace
  k wait --for=condition=Established crd/clickhouseclusters.clickhouse.com --timeout=180s
  k wait --for=condition=Established crd/keeperclusters.clickhouse.com --timeout=180s
fi

echo "→ Installing nginx ingress (if missing)..."
if ! k get ns ingress-nginx >/dev/null 2>&1; then
  k apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.10.0/deploy/static/provider/kind/deploy.yaml
fi
for _ in $(seq 1 60); do
  if k wait --namespace ingress-nginx \
    --for=condition=ready pod \
    --selector=app.kubernetes.io/component=controller \
    --timeout=10s 2>/dev/null; then
    break
  fi
  sleep 5
done
if ! k get pods -n ingress-nginx -l app.kubernetes.io/component=controller 2>/dev/null | grep -q Running; then
  echo "⚠ nginx ingress controller not Running yet — continuing (may need more resources)" >&2
fi

echo "✓ k8s prerequisites ready (context=${KUBECTL_CONTEXT})"
