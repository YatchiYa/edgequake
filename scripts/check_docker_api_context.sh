#!/usr/bin/env bash
# Fail closed if the API Docker build context would break `cargo` manifest parse.
# Root cause of v0.16.0 CD flake: .dockerignore excluded benches/ while Cargo.toml
# declares [[bench]] (and [[example]]) — cargo refuses to parse without those files.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EQ="$ROOT/edgequake"
DOCKERIGNORE="$EQ/.dockerignore"
CARGO_TOML="$EQ/Cargo.toml"

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

[[ -f "$CARGO_TOML" ]] || fail "missing $CARGO_TOML"
[[ -f "$DOCKERIGNORE" ]] || fail "missing $DOCKERIGNORE"

# Paths that must exist on disk for cargo to parse the workspace root manifest.
REQUIRED_PATHS=(
  benches/chunking_bench.rs
  benches/storage_bench.rs
  benches/graph_performance.rs
  benches/graphrag_bench.rs
  examples/basic_rag.rs
  examples/streaming_query.rs
  examples/graph_exploration.rs
  examples/multi_tenant.rs
)

for rel in "${REQUIRED_PATHS[@]}"; do
  [[ -f "$EQ/$rel" ]] || fail "missing required path for Cargo.toml: $rel"
done

# .dockerignore must NOT exclude benches/ or examples/ (Dockerfile COPYs them).
if grep -E '^(benches/|examples/)$' "$DOCKERIGNORE" >/dev/null; then
  fail ".dockerignore excludes benches/ or examples/ — Docker cargo build will fail to parse Cargo.toml"
fi

# Dockerfile must COPY both directories.
DOCKERFILE="$EQ/docker/Dockerfile"
[[ -f "$DOCKERFILE" ]] || fail "missing $DOCKERFILE"
grep -q 'COPY benches/ benches/' "$DOCKERFILE" || fail "Dockerfile missing: COPY benches/ benches/"
grep -q 'COPY examples/ examples/' "$DOCKERFILE" || fail "Dockerfile missing: COPY examples/ examples/"

# Parse-only check (no compile) — same failure mode as the Docker builder stage.
(cd "$EQ" && cargo metadata --format-version 1 --no-deps >/dev/null)

echo "Docker API context OK (benches/examples present, dockerignore + Dockerfile aligned)"
