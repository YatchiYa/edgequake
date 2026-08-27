#!/usr/bin/env bash
# SPEC-138 — kubectl context helper for kind cluster.
set -euo pipefail

CLUSTER_NAME="${KIND_CLUSTER_NAME:-edgequake-spec138}"
export KIND_CLUSTER_NAME="${CLUSTER_NAME}"

if kind get clusters 2>/dev/null | grep -qx "${CLUSTER_NAME}"; then
  export KUBECTL_CONTEXT="kind-${CLUSTER_NAME}"
else
  export KUBECTL_CONTEXT="${KUBECTL_CONTEXT:-$(kubectl config current-context 2>/dev/null || true)}"
fi

k() {
  kubectl --context "${KUBECTL_CONTEXT}" "$@"
}
