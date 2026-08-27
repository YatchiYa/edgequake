#!/usr/bin/env bash
# SPEC-138 — print Langfuse init keys from stack values (SSOT).
set -euo pipefail

PK="${LANGFUSE_PUBLIC_KEY:-pk-lf-edgequake-k8s}"
SK="${LANGFUSE_SECRET_KEY:-sk-lf-edgequake-k8s-dev}"
PROJECT_ID="${LANGFUSE_PROJECT_ID:-edgequake-k8s}"
BASE="${LANGFUSE_BASE_URL:-http://langfuse-web.langfuse.svc.cluster.local:3000}"

echo "LANGFUSE_PUBLIC_KEY=${PK}"
echo "LANGFUSE_SECRET_KEY=${SK}"
echo "LANGFUSE_PROJECT_ID=${PROJECT_ID}"
echo "LANGFUSE_BASE_URL=${BASE}"
