#!/usr/bin/env bash
# SPEC-071 — Wave-2 greenfield turnkey env (opt-in; no silent DB migrate).
#
# Usage:
#   eval "$(./scripts/wave2_greenfield_env.sh)"
#   source ./scripts/wave2_greenfield_env.sh   # when sourced, exports into current shell
#   ./scripts/wave2_greenfield_env.sh          # print export lines
#
# See docs/product-limits.md "Turnkey greenfield".
set -euo pipefail

print_exports() {
  cat <<'EOF'
# SPEC-071 Wave-2 greenfield turnkey — supported 100k filtered ANN shape
export EDGEQUAKE_VECTOR_STORAGE=halfvec
export EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1
# Concurrent headroom tip @100k (SPEC-068) — not the silent product clamp default
export EDGEQUAKE_HNSW_EF_SEARCH=240
# Optional:
# export EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS=1000
EOF
}

if [[ "${1:-}" == "--print" ]] || [[ ! -t 1 && "${WAVE2_ENV_FORCE_PRINT:-}" == "1" ]]; then
  print_exports
  exit 0
fi

# If sourced: export into caller shell
if [[ "${BASH_SOURCE[0]:-}" != "${0:-}" ]] || [[ -n "${ZSH_EVAL_CONTEXT:-}" && "$ZSH_EVAL_CONTEXT" == *:file* ]]; then
  # shellcheck disable=SC1091
  eval "$(print_exports | grep -v '^#')"
  echo "NOTE: Wave-2 greenfield env exported (halfvec + partial HNSW + ef_search=240)" >&2
  return 0 2>/dev/null || exit 0
fi

print_exports
