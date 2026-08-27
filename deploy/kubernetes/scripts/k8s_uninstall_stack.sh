#!/usr/bin/env bash
# SPEC-138 — uninstall stack (keeps kind cluster).
set -euo pipefail

# shellcheck source=/dev/null
source "$(dirname "$0")/k8s_context.sh"

helm uninstall edgequake-stack --kube-context "${KUBECTL_CONTEXT}" -n edgequake 2>/dev/null || true
helm uninstall langfuse --kube-context "${KUBECTL_CONTEXT}" -n langfuse 2>/dev/null || true
k delete namespace edgequake --ignore-not-found --timeout=120s || true
k delete namespace langfuse --ignore-not-found --timeout=120s || true
echo "✓ stack uninstalled"
