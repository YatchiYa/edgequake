#!/usr/bin/env bash
# 083 — Product query API latency smoke (keyword override).
# Labeled peer PRODUCT_QUERY_API_v1. Never promotes Acc publish/latest.
#
# Compares keyword_time_ms for the same query with vs without hl/ll override
# on the warm B5 workspace (query-only; context_only to isolate keyword stage).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

export BENCH001_SKIP_PUBLISH_LATEST=1
export BENCH001_PUBLISH_PEER="${BENCH001_PUBLISH_PEER:-PRODUCT_QUERY_API_v1}"
export BENCH001_EQ_WORKSPACE_ID="${BENCH001_EQ_WORKSPACE_ID:-8e990410-43b5-44f4-9f56-87bd154570ce}"

ACC_PORT="${BENCH001_ACC_PORT:-8090}"
API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:${ACC_PORT}}"
QUERY="${PRODUCT_QUERY_TEXT:-What is the TNM staging system for NSCLC?}"
HL_JSON='["staging","NSCLC","TNM"]'
LL_JSON='["non-small cell lung cancer","tumor stage"]'

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT_DIR="${PRODUCT_QUERY_OUT:-specs/001-benchmark/e2e/artifacts/history/product-query-api-${STAMP}}"
mkdir -p "$OUT_DIR"

echo "==> 083 PRODUCT_QUERY_API_v1 keyword-skip latency smoke"
echo "==> api=$API_URL workspace=$BENCH001_EQ_WORKSPACE_ID"
echo "==> Acc publish/latest skipped (BENCH001_SKIP_PUBLISH_LATEST=1)"
echo "==> peer=$BENCH001_PUBLISH_PEER out=$OUT_DIR"

if ! curl -fsS "$API_URL/health" >/dev/null 2>&1; then
  echo "==> API not healthy; starting Acc backend on :$ACC_PORT"
  python3 tools/bench001/scripts/start_acc_backend.py --port "$ACC_PORT"
  set -a
  [[ -f /tmp/edgequake-dev-ports.env ]] && . /tmp/edgequake-dev-ports.env
  [[ -f "$ROOT/.edgequake-dev-ports.env" ]] && . "$ROOT/.edgequake-dev-ports.env"
  set +a
  API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:${ACC_PORT}}"
fi

curl -fsS "$API_URL/health" | tee "$OUT_DIR/health.json" >/dev/null

# Acc / product API mounts query at /api/v1/query (bench001 client path).
QUERY_URL="${API_URL%/}/api/v1/query"
hdr=(-H "Content-Type: application/json" -H "X-Workspace-ID: ${BENCH001_EQ_WORKSPACE_ID}")
if [[ -n "${EDGEQUAKE_API_KEY:-${BENCH001_API_KEY:-}}" ]]; then
  _key="${EDGEQUAKE_API_KEY:-${BENCH001_API_KEY}}"
  hdr+=(-H "X-API-Key: ${_key}" -H "Authorization: Bearer ${_key}")
fi

post_query() {
  local label="$1"
  local body="$2"
  local file="$OUT_DIR/${label}.json"
  curl -fsS "${hdr[@]}" -d "$body" "$QUERY_URL" | tee "$file" >/dev/null
  python3 - "$file" <<'PY'
import json, sys
path = sys.argv[1]
data = json.load(open(path))
stats = data.get("stats") or {}
print(stats.get("keyword_time_ms", -1))
PY
}

CONTROL_BODY=$(python3 - <<PY
import json
print(json.dumps({
  "query": """$QUERY""",
  "mode": "mix",
  "context_only": True,
  "enable_rerank": False,
}))
PY
)

OVERRIDE_BODY=$(python3 - <<PY
import json
print(json.dumps({
  "query": """$QUERY""",
  "mode": "mix",
  "context_only": True,
  "enable_rerank": False,
  "hl_keywords": json.loads('''$HL_JSON'''),
  "ll_keywords": json.loads('''$LL_JSON'''),
  "response_type": "Multiple Paragraphs",
}))
PY
)

echo "==> control (no hl/ll)…"
CONTROL_KW_MS="$(post_query control "$CONTROL_BODY")"
echo "    keyword_time_ms=$CONTROL_KW_MS"

echo "==> override (hl/ll supplied)…"
OVERRIDE_KW_MS="$(post_query override "$OVERRIDE_BODY")"
echo "    keyword_time_ms=$OVERRIDE_KW_MS"

python3 - "$OUT_DIR" "$CONTROL_KW_MS" "$OVERRIDE_KW_MS" "$BENCH001_PUBLISH_PEER" "$BENCH001_EQ_WORKSPACE_ID" <<'PY'
import json, sys
from pathlib import Path
out, ctrl, ov, peer, ws = sys.argv[1:6]
ctrl_ms = int(float(ctrl))
ov_ms = int(float(ov))
# Override must be near-zero vs control (keyword LLM skipped). Allow 50ms floor noise.
ok = ov_ms <= max(50, ctrl_ms // 5) if ctrl_ms > 0 else ov_ms <= 50
summary = {
    "peer": peer,
    "workspace_id": ws,
    "control_keyword_time_ms": ctrl_ms,
    "override_keyword_time_ms": ov_ms,
    "keyword_skip_pass": ok,
    "acc_publish_latest": "skipped",
    "claim": "EQ query API matches LightRAG keyword-override (not Acc Beat)",
}
Path(out, "LATENCY_SUMMARY.json").write_text(json.dumps(summary, indent=2) + "\n")
Path(out, "ABLATION_NOTE.md").write_text(
    f"""# 083 PRODUCT_QUERY_API_v1 — keyword-skip latency

- Peer: `{peer}` (labeled; Acc `publish/latest` **skipped**)
- Workspace: `{ws}` (B5 warm)
- control keyword_time_ms: **{ctrl_ms}**
- override keyword_time_ms: **{ov_ms}**
- keyword_skip_pass: **{ok}**

Success claim: product query API law (hl/ll skip), **not** Acc Beat.
"""
)
peer_dir = Path("specs/001-benchmark/e2e/artifacts/publish/peers") / peer
peer_dir.mkdir(parents=True, exist_ok=True)
(peer_dir / "LATENCY_SUMMARY.json").write_text(json.dumps(summary, indent=2) + "\n")
(peer_dir / "ABLATION_NOTE.md").write_text(Path(out, "ABLATION_NOTE.md").read_text())
print(json.dumps(summary, indent=2))
sys.exit(0 if ok else 1)
PY

echo "==> done — Acc latest untouched; peer=$BENCH001_PUBLISH_PEER"
