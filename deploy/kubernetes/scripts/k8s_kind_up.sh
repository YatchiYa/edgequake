#!/usr/bin/env bash
# SPEC-138 E2E-138-02 — create kind cluster for proof.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
# shellcheck source=/dev/null
source "$(dirname "$0")/k8s_context.sh"
KIND_CONFIG="${ROOT}/deploy/kubernetes/kind/kind-config.yaml"

command -v kind >/dev/null 2>&1 || { echo "✗ kind required: brew install kind" >&2; exit 1; }

if kind get clusters 2>/dev/null | grep -qx "${CLUSTER_NAME}"; then
  echo "→ kind cluster ${CLUSTER_NAME} already exists"
else
  echo "→ Creating kind cluster ${CLUSTER_NAME}..."
  kind create cluster --name "${CLUSTER_NAME}" --config "${KIND_CONFIG}"
fi

# Refresh context after create
# shellcheck source=/dev/null
source "$(dirname "$0")/k8s_context.sh"

k cluster-info
echo "✓ kind cluster ready: ${CLUSTER_NAME} (context=${KUBECTL_CONTEXT})"
