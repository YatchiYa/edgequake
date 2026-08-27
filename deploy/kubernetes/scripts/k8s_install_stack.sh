#!/usr/bin/env bash
# SPEC-138 E2E-138-03 — install Langfuse + EdgeQuake stack on cluster.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
# shellcheck source=/dev/null
source "$(dirname "$0")/k8s_context.sh"
HELM_DIR="${ROOT}/deploy/kubernetes/helm"
EDGEQUAKE_VERSION="${EDGEQUAKE_VERSION:-0.26.1}"
LANGFUSE_CHART_VERSION="${LANGFUSE_CHART_VERSION:-2.0.0}"

echo "→ Helm repo add langfuse..."
helm repo add langfuse https://langfuse.github.io/langfuse-k8s 2>/dev/null || true
helm repo update langfuse

echo "→ Namespace langfuse (must exist from langfuse helm install)..."
k get namespace langfuse >/dev/null 2>&1 || {
  echo "✗ namespace langfuse missing — install Langfuse first" >&2
  exit 1
}

echo "→ Install Langfuse (namespace: langfuse)..."
helm upgrade --install langfuse langfuse/langfuse \
  --kube-context "${KUBECTL_CONTEXT}" \
  --version "${LANGFUSE_CHART_VERSION}" \
  --namespace langfuse \
  --create-namespace \
  -f "${HELM_DIR}/langfuse-values-kind.yaml" \
  --wait --timeout 25m

echo "→ Build edgequake-stack dependencies..."
helm dependency build "${HELM_DIR}/edgequake-stack"

echo "→ Install EdgeQuake stack..."
helm upgrade --install edgequake-stack "${HELM_DIR}/edgequake-stack" \
  --kube-context "${KUBECTL_CONTEXT}" \
  --namespace edgequake \
  --create-namespace \
  -f "${HELM_DIR}/edgequake-stack/values-kind.yaml" \
  --set edgequake.global.edgequakeVersion="${EDGEQUAKE_VERSION}" \
  --wait --timeout 15m

echo "✓ stack installed (Langfuse + EdgeQuake)"
