#!/usr/bin/env bash
# Pre-release quality gates — fmt, workspace clippy, optional lib tests,
# SPEC-006 + SPEC-018 proofs, WebUI typecheck, version parity.
#
# Env knobs (CI sets these to avoid duplicate work already covered by CI.yml):
#   RELEASE_SKIP_LIB_TESTS=1          — skip workspace lib tests
#   RELEASE_SKIP_PER_CRATE_CLIPPY=1   — skip O(N) per-crate clippy (workspace is enough)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EQ="$ROOT/edgequake"
WEBUI="$ROOT/edgequake_webui"

CRATES=(
  edgequake-api
  edgequake-audit
  edgequake-auth
  edgequake-core
  edgequake-observability
  edgequake-pdf
  edgequake-pipeline
  edgequake-query
  edgequake-rate-limiter
  edgequake-storage
  edgequake-tasks
)

echo "== rustfmt =="
(cd "$EQ" && cargo fmt --all -- --check)

echo "== workspace clippy =="
(cd "$EQ" && cargo clippy --workspace --lib --locked -- -D warnings)

if [[ "${RELEASE_SKIP_PER_CRATE_CLIPPY:-}" == "1" ]]; then
  echo "== per-crate clippy =="
  echo "skipped (RELEASE_SKIP_PER_CRATE_CLIPPY=1 — workspace clippy is SSOT)"
else
  echo "== per-crate clippy =="
  for crate in "${CRATES[@]}"; do
    echo "→ clippy -p $crate"
    FEATURES=()
    case "$crate" in
      edgequake-api|edgequake-core|edgequake-storage|edgequake-tasks)
        FEATURES=(--features postgres)
        ;;
    esac
    if ((${#FEATURES[@]})); then
      (cd "$EQ" && cargo clippy -p "$crate" --lib --locked "${FEATURES[@]}" -- -D warnings)
    else
      (cd "$EQ" && cargo clippy -p "$crate" --lib --locked -- -D warnings)
    fi
  done
fi

echo "== workspace lib tests =="
if [[ "${RELEASE_SKIP_LIB_TESTS:-}" == "1" ]]; then
  echo "skipped (RELEASE_SKIP_LIB_TESTS=1 — full suite runs on main CI)"
else
  (cd "$EQ" && cargo test --workspace --lib --locked --no-fail-fast)
fi

echo "== SPEC-006 resource-proof =="
(cd "$ROOT" && make resource-proof --no-print-directory)

echo "== SPEC-018 observability-proof =="
chmod +x "$ROOT/specs/018-observability/e2e/run_observability_proof.sh"
"$ROOT/specs/018-observability/e2e/run_observability_proof.sh"

echo "== WebUI typecheck (src only; e2e via Playwright) =="
(cd "$WEBUI" && bunx tsc --noEmit -p tsconfig.release.json)

echo "== WebUI unit tests (observability + runtime-config) =="
(cd "$WEBUI" && bun test src/lib/api/__tests__/observability-client.test.ts src/lib/__tests__/runtime-config.test.ts)

echo "== Docker API context (cargo manifest + COPY/dockerignore) =="
chmod +x "$ROOT/scripts/check_docker_api_context.sh"
"$ROOT/scripts/check_docker_api_context.sh"

echo "== WebUI next.config SizeLimit guard =="
# Next 16 SizeLimit = number | \`${number}${suffix}\`. Template expressions widen to
# `string` and fail `next build` typecheck in Docker CD — require numeric SSOT.
if grep -E 'proxyClientMaxBodySize:\s*`|DEV_PROXY_MAX_BODY' "$WEBUI/next.config.ts" >/dev/null; then
  echo "ERROR: next.config.ts must use numeric SizeLimit (DEFAULT_MAX_UPLOAD_BYTES), not a string template"
  exit 1
fi
grep -q 'proxyClientMaxBodySize: DEFAULT_MAX_UPLOAD_BYTES' "$WEBUI/next.config.ts" \
  || { echo "ERROR: next.config.ts missing proxyClientMaxBodySize: DEFAULT_MAX_UPLOAD_BYTES"; exit 1; }
echo "next.config SizeLimit guard OK"

echo "== Release version parity (VERSION vs Cargo.toml vs package.json vs README) =="
# Workspace package version lives under [workspace.package], not the root [package].
API_VER=$(
  awk '
    /^\[workspace\.package\]/ { in_ws=1; next }
    /^\[/ { in_ws=0 }
    in_ws && /^version[[:space:]]*=/ {
      if (match($0, /"[0-9]+\.[0-9]+\.[0-9]+"/)) {
        print substr($0, RSTART+1, RLENGTH-2)
        exit
      }
    }
  ' "$EQ/Cargo.toml"
)
UI_VER=$(node -p "require('$WEBUI/package.json').version")
FILE_VER=$(cat "$ROOT/VERSION" 2>/dev/null || echo "")
README_VER=$(grep -Eo 'badge/version-[0-9]+\.[0-9]+\.[0-9]+' "$ROOT/README.md" | head -1 | sed 's/badge\/version-//')
if [[ -z "$API_VER" ]]; then
  echo "ERROR: could not parse workspace.package version from edgequake/Cargo.toml"
  exit 1
fi
if [[ "$API_VER" != "$UI_VER" ]]; then
  echo "ERROR: version mismatch — edgequake/Cargo.toml=$API_VER edgequake_webui/package.json=$UI_VER"
  exit 1
fi
if [[ -n "$FILE_VER" && "$FILE_VER" != "$API_VER" ]]; then
  echo "ERROR: VERSION file=$FILE_VER does not match Cargo.toml=$API_VER"
  exit 1
fi
if [[ -n "$README_VER" && "$README_VER" != "$API_VER" ]]; then
  echo "ERROR: README badge version=$README_VER does not match Cargo.toml=$API_VER"
  exit 1
fi
echo "Release version parity OK: $API_VER"

echo "== Crate package version parity (workspace inherit or == VERSION) =="
CRATE_DRIFT=0
while IFS= read -r crate_toml; do
  pkg_block=$(awk '
    /^\[package\]/ { in_pkg=1; next }
    /^\[/ { in_pkg=0 }
    in_pkg { print }
  ' "$crate_toml")
  if echo "$pkg_block" | grep -qE '^version\.workspace[[:space:]]*=[[:space:]]*true'; then
    continue
  fi
  crate_ver=$(echo "$pkg_block" | grep -E '^version[[:space:]]*=' | head -1 | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/')
  if [[ -z "$crate_ver" ]]; then
    echo "ERROR: $crate_toml has neither version.workspace = true nor a numeric version"
    CRATE_DRIFT=1
    continue
  fi
  if [[ "$crate_ver" != "$API_VER" ]]; then
    echo "ERROR: $crate_toml version=$crate_ver does not match VERSION=$API_VER"
    CRATE_DRIFT=1
  fi
done < <(find "$EQ/crates" -mindepth 2 -maxdepth 2 -name Cargo.toml | sort)
if [[ "$CRATE_DRIFT" -ne 0 ]]; then
  exit 1
fi
echo "Crate package version parity OK"

echo "== OpenAPI snapshot version parity =="
SNAPSHOT="$WEBUI/openapi/openapi.snapshot.json"
if [[ ! -f "$SNAPSHOT" ]]; then
  echo "ERROR: missing OpenAPI snapshot at $SNAPSHOT — run make codegen-openapi-refresh"
  exit 1
fi
OPENAPI_VER=$(node -p "require('$SNAPSHOT').info.version")
if [[ "$OPENAPI_VER" != "$API_VER" ]]; then
  echo "ERROR: openapi.snapshot.json info.version=$OPENAPI_VER does not match VERSION=$API_VER"
  echo "HINT: run make codegen-openapi-refresh"
  exit 1
fi
echo "OpenAPI snapshot version parity OK: $OPENAPI_VER"

echo "✓ release gates passed"
