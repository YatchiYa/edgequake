#!/usr/bin/env bash
# SPEC-064 — filtered ANN scale battle on a single major (default pg18).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROFILE="${1:-pg18}"
export EDGEQUAKE_PERF_SCALE=large
export EDGEQUAKE_CAPACITY_LADDER="${EDGEQUAKE_CAPACITY_LADDER:-L1}"
export EDGEQUAKE_PERF_RELEASE=1
export EDGEQUAKE_BATTLE_ARMS="${EDGEQUAKE_BATTLE_ARMS:-full_default,halfvec_default,halfvec_partial_ws,guc_grid}"

ART_DIR="$ROOT/specs/064-filtered-ann-scale-battle/e2e/artifacts"
mkdir -p "$ART_DIR"
LOG="/tmp/eq-battle-${PROFILE}.log"
REPORT="/tmp/eq-battle-${PROFILE}.jsonl"

echo "NOTE: SPEC-064 ANN scale battle on $PROFILE (arms=$EDGEQUAKE_BATTLE_ARMS, --release, large/L1)"

# shellcheck source=/dev/null
source "$ROOT/scripts/eq_ephemeral_pg.sh"
eq_ephemeral_pg_start "$PROFILE" "edgequake-battle"
eq_ephemeral_pg_migrate

: >"$LOG"
: >"$REPORT"
cd "$EDGEQUAKE_DIR"
cargo test -p edgequake-storage --features postgres --release \
  --test e2e_spec064_ann_scale_battle -- --nocapture 2>&1 | tee "$LOG"

grep -E '^PERF_REPORT ' "$LOG" | sed 's/^PERF_REPORT //' >>"$REPORT" || true

if grep -E 'SKIP:.*(DATABASE_URL|POSTGRES_PASSWORD)' "$LOG" >/dev/null 2>&1; then
  echo "ERROR: DATABASE soft-skip under REQUIRE_POSTGRES"
  exit 1
fi

cp "$REPORT" "$ART_DIR/eq-battle-${PROFILE}.jsonl"
cp "$LOG" "$ART_DIR/eq-battle-${PROFILE}-cargo.log"

python3 - "$ART_DIR/eq-battle-${PROFILE}.jsonl" "$ART_DIR/WAVE0_EXPLAIN.md" "$PROFILE" <<'PY'
import json, sys
from pathlib import Path
src, dst, profile = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
explain = None
baseline = None
for line in src.read_text().splitlines():
    try:
        obj = json.loads(line)
    except Exception:
        continue
    op = obj.get("op", "")
    if op == "battle_full_default_explain":
        explain = obj.get("detail", "")
    if op == "battle_full_default_single":
        baseline = obj
text = [
    f"# WAVE0 EXPLAIN — SPEC-064 ({profile})",
    "",
    "See also locked hypothesis in prior WAVE0 / RUN_NOTES (btree filter → exact scan).",
    "",
    "## Effective GUCs (code defaults unless overridden)",
    "",
    "- `hnsw.ef_search` ≈ `clamp(4×top_k, 40, 1000)` (top_k=20 → 80)",
    "- `hnsw.iterative_scan = relaxed_order` when filtered + pgvector ≥ 0.8",
    "- `hnsw.max_scan_tuples = 20000`",
    "- storage mode for this arm: **full** (`vector`)",
    "",
]
if baseline:
    text += [
        "## Baseline single (full_default)",
        "",
        f"- p95_ms: **{baseline.get('p95_ms')}**",
        f"- pass (Q1-d): `{baseline.get('pass')}`",
        f"- detail: `{baseline.get('detail')}`",
        "",
    ]
text += ["## EXPLAIN (ANALYZE, BUFFERS)", ""]
if explain:
    text.append("```")
    text.append(explain)
    text.append("```")
else:
    text.append("_No `battle_full_default_explain` line — re-run with arm `full_default`._")
text += ["", "Artifacts: `eq-battle-pg18.jsonl`, `RUN_NOTES.md`.", ""]
dst.write_text("\n".join(text) + "\n")
print(f"OK wrote {dst}")
PY

echo "OK ANN scale battle on $PROFILE → $ART_DIR/eq-battle-${PROFILE}.jsonl"
