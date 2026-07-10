#!/usr/bin/env bash
# SPEC-046 EQ-046-16 — write AccReport JSON for CI artifact upload.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT_DIR="${SPEC046_ACC_OUT:-$ROOT/edgequake/target/spec046-acc}"
mkdir -p "$OUT_DIR"
cd "$ROOT/edgequake"
cargo test -p edgequake-query --test e2e_spec046_science_p4 --test e2e_spec046_ops_p3_acc -- --test-threads=4
# Artifact path produced by e2e_science_p4_acc_full_pass_and_json_artifact
REPORT="$OUT_DIR/acc_report.json"
if [[ ! -f "$REPORT" ]]; then
  echo "FAIL: expected ACC report at $REPORT" >&2
  exit 1
fi
echo "ACC report ready: $REPORT"
wc -c "$REPORT"
