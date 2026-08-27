#!/usr/bin/env bash
# SPEC-124: smoke local Langfuse Docker (no EdgeQuake required).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=/dev/null
source "${ROOT}/scripts/langfuse_e2e_common.sh"

PORT="${LANGFUSE_PORT:-3310}"
BASE="${LANGFUSE_UI_URL:-http://localhost:${PORT}}"
PK="${LANGFUSE_LOCAL_PK:-pk-lf-edgequake-local}"
SK="${LANGFUSE_LOCAL_SK:-sk-lf-edgequake-local-dev}"
EXPECT_ID="${LANGFUSE_LOCAL_PROJECT_ID:-edgequake-local}"

langfuse_smoke "${BASE}" "${PK}" "${SK}" "${EXPECT_ID}"
