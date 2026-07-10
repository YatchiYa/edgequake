#!/usr/bin/env bash
# SPEC-018: Run all observability proof commands (non-interactive).
#
# First principles: scoped cargo targets + raised stack for deep query smoke.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
EQ="$ROOT/edgequake"
WEBUI="$ROOT/edgequake_webui"

# Deep Axum/query handler tests overflow default stacks under debug.
export RUST_MIN_STACK="${RUST_MIN_STACK:-16777216}"

cd "$EQ"

echo "== edgequake-observability =="
cargo test -p edgequake-observability --lib --locked

echo "== edgequake-tasks =="
cargo test -p edgequake-tasks --lib --locked

echo "== edgequake-api observability =="
cargo test -p edgequake-api --test observability_proof --features postgres --locked
cargo test -p edgequake-api --lib test_request_id_header_added --features postgres --locked 2>/dev/null || \
  cargo test -p edgequake-api --test integration_tests test_request_id_header_added --features postgres --locked

echo "== edgequake-audit =="
cargo test -p edgequake-audit --lib --locked

echo "== edgequake-api lib (smoke) =="
cargo test -p edgequake-api --lib --features postgres --locked -- \
  handlers::query::tests::test_query_success

echo "== edgequake-webui =="
cd "$WEBUI"
bun test src/lib/api/__tests__/observability-client.test.ts

echo "SPEC-018: all proof commands passed."
