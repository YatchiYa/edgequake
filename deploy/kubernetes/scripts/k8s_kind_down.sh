#!/usr/bin/env bash
# SPEC-138 — delete kind cluster.
set -euo pipefail

CLUSTER_NAME="${KIND_CLUSTER_NAME:-edgequake-spec138}"
command -v kind >/dev/null 2>&1 || exit 0

if kind get clusters 2>/dev/null | grep -qx "${CLUSTER_NAME}"; then
  kind delete cluster --name "${CLUSTER_NAME}"
  echo "✓ deleted kind cluster ${CLUSTER_NAME}"
else
  echo "→ no cluster ${CLUSTER_NAME}"
fi
