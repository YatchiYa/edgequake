#!/usr/bin/env bash
# SPEC-138 — wait for core pods ready.
set -euo pipefail

# shellcheck source=/dev/null
source "$(dirname "$0")/k8s_context.sh"

echo "→ Waiting for EdgeQuake pods..."
k wait --for=condition=Ready pod \
  -l app.kubernetes.io/instance=edgequake-stack,app.kubernetes.io/component=api \
  -n edgequake --timeout=600s

k wait --for=condition=Ready pod \
  -l app.kubernetes.io/instance=edgequake-stack,app.kubernetes.io/component=web \
  -n edgequake --timeout=600s

k wait --for=condition=Ready pod \
  -l app.kubernetes.io/instance=edgequake-stack,app.kubernetes.io/component=postgres \
  -n edgequake --timeout=600s

echo "→ Waiting for Langfuse web..."
k wait --for=condition=Ready pod \
  -l app.kubernetes.io/name=langfuse \
  -n langfuse --timeout=900s 2>/dev/null || \
k wait --for=condition=Ready pod \
  -l app=langfuse-web \
  -n langfuse --timeout=900s

echo "✓ pods ready"
