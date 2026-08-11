#!/usr/bin/env bash
# SPEC-122 bulk ingest measurement harness (LAW-122-5).
# Usage:
#   BASE_URL=http://127.0.0.1:8090 ARM=A N=5 ./measure-bulk-ingest.sh
# Env:
#   BASE_URL, WORKSPACE_ID, ARM (A|B|C), N, FIXTURE_DIR, OUT_DIR, TIMEOUT_S
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BASE_URL="${BASE_URL:-http://127.0.0.1:8090}"
WORKSPACE_ID="${WORKSPACE_ID:-default}"
ARM="${ARM:-C}"
N="${N:-1}"
TIMEOUT_S="${TIMEOUT_S:-1800}"
FIXTURE_DIR="${FIXTURE_DIR:-$ROOT/zz_test_docs}"
OUT_DIR="${OUT_DIR:-$ROOT/specs/122-implementation/measurements}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RUN_DIR="$OUT_DIR/${STAMP}-arm${ARM}-n${N}"
mkdir -p "$RUN_DIR"

AUTH_HEADER=()
if [[ -n "${EDGEQUAKE_TOKEN:-}" ]]; then
  AUTH_HEADER=(-H "Authorization: Bearer $EDGEQUAKE_TOKEN")
fi
WS_HEADER=(-H "X-Workspace-Id: $WORKSPACE_ID")

json_get() {
  python3 -c "import json,sys; d=json.load(sys.stdin); print(d$1)" 2>/dev/null || true
}

metrics_snap() {
  local label="$1"
  curl -fsS "${AUTH_HEADER[@]}" "$BASE_URL/api/v1/pipeline/queue-metrics" \
    >"$RUN_DIR/metrics-${label}.json" || echo '{"error":"metrics_failed"}' >"$RUN_DIR/metrics-${label}.json"
}

health_snap() {
  curl -fsS "${AUTH_HEADER[@]}" "$BASE_URL/health" >"$RUN_DIR/health.json" || true
}

status_of_track() {
  local track_id="$1"
  curl -fsS "${AUTH_HEADER[@]}" "${WS_HEADER[@]}" \
    "$BASE_URL/api/v1/documents/track/${track_id}" 2>/dev/null || echo '{}'
}

# Build N unique small text fixtures (avoid content-hash dedup).
build_fixtures() {
  local i=1
  FIXTURES=()
  local srcs=(
    "$FIXTURE_DIR/test_injection.txt"
    "$FIXTURE_DIR/test_injection.md"
    "$FIXTURE_DIR/test-document.md"
  )
  while (( i <= N )); do
    local src="${srcs[$(( (i - 1) % ${#srcs[@]} ))]}"
    local dst="$RUN_DIR/fixture-${i}.txt"
    {
      echo "SPEC-122 ARM=${ARM} N=${N} INDEX=${i} TS=${STAMP}"
      echo "UNIQUE=$(uuidgen 2>/dev/null || python3 -c 'import uuid; print(uuid.uuid4())')"
      cat "$src"
    } >"$dst"
    FIXTURES+=("$dst")
    ((i++)) || true
  done
}

admit_one() {
  local file="$1"
  local name
  name="$(basename "$file")"
  local content
  content="$(cat "$file")"
  # Escape for JSON
  local payload
  payload="$(python3 - <<PY
import json
print(json.dumps({
  "content": open("$file").read(),
  "file_source": "$name",
  "async_processing": True,
}))
PY
)"
  curl -fsS -X POST "$BASE_URL/api/v1/documents" \
    "${AUTH_HEADER[@]}" "${WS_HEADER[@]}" \
    -H "Content-Type: application/json" \
    -d "$payload"
}

main() {
  echo "SPEC-122 measure ARM=$ARM N=$N BASE_URL=$BASE_URL → $RUN_DIR"
  health_snap
  metrics_snap "pre"
  build_fixtures

  declare -a TRACKS=()
  declare -a DOC_IDS=()
  local t0 t1
  t0="$(python3 -c 'import time; print(time.time())')"

  local f
  for f in "${FIXTURES[@]}"; do
    local resp
    resp="$(admit_one "$f")"
    echo "$resp" >>"$RUN_DIR/admits.jsonl"
    local tid did
    tid="$(echo "$resp" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("track_id") or d.get("task_id") or "")')"
    did="$(echo "$resp" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("document_id") or "")')"
    TRACKS+=("$tid")
    DOC_IDS+=("$did")
    echo "admitted $did track=$tid"
  done

  t1="$(python3 -c 'import time; print(time.time())')"
  local admit_s
  admit_s="$(python3 -c "print(round($t1 - $t0, 3))")"

  metrics_snap "post-admit"

  local first_s="" all_s=""
  local deadline
  deadline="$(python3 -c "import time; print(time.time() + $TIMEOUT_S)")"
  local max_proc=0
  declare -A DONE=()

  while true; do
    local now completed=0 processing=0
    now="$(python3 -c 'import time; print(time.time())')"
    if python3 -c "import sys; sys.exit(0 if $now > $deadline else 1)"; then
      echo "TIMEOUT after ${TIMEOUT_S}s" | tee "$RUN_DIR/timeout.txt"
      break
    fi

    local i=0
    for tid in "${TRACKS[@]}"; do
      if [[ -n "${DONE[$tid]:-}" ]]; then
        ((completed++)) || true
        ((i++)) || true
        continue
      fi
      local st_json status
      st_json="$(status_of_track "$tid")"
      echo "$st_json" >>"$RUN_DIR/status-polls.jsonl"
      status="$(echo "$st_json" | python3 -c 'import json,sys
try:
 d=json.load(sys.stdin)
 print((d.get("status") or d.get("display_status") or d.get("state") or "").lower())
except Exception:
 print("")')"
      case "$status" in
        completed|processed|success|done|failed|error|cancelled)
          DONE[$tid]="$status"
          ((completed++)) || true
          if [[ -z "$first_s" ]]; then
            first_s="$(python3 -c "print(round($now - $t0, 3))")"
          fi
          ;;
        processing|running|converting|pending|queued|"")
          if [[ "$status" == "processing" || "$status" == "running" || "$status" == "converting" ]]; then
            ((processing++)) || true
          fi
          ;;
      esac
      ((i++)) || true
    done

    if (( processing > max_proc )); then max_proc=$processing; fi

    metrics_snap "mid"
    if (( completed >= N )); then
      all_s="$(python3 -c "print(round($now - $t0, 3))")"
      break
    fi
    sleep 2
  done

  metrics_snap "final"

  local docs_per_min="null"
  if [[ -n "$all_s" && "$all_s" != "0" ]]; then
    docs_per_min="$(python3 -c "print(round(($N / float('$all_s')) * 60.0, 3))")"
  fi

  python3 - <<PY >"$RUN_DIR/summary.json"
import json
summary = {
  "arm": "$ARM",
  "n": $N,
  "base_url": "$BASE_URL",
  "workspace_id": "$WORKSPACE_ID",
  "stamp": "$STAMP",
  "admit_s": float("$admit_s"),
  "t_first_complete_s": None if "$first_s" == "" else float("$first_s"),
  "t_all_complete_s": None if "$all_s" == "" else float("$all_s"),
  "docs_per_min": None if "$docs_per_min" == "null" else float("$docs_per_min"),
  "max_concurrent_processing_observed": $max_proc,
  "tracks": $(python3 -c 'import json; print(json.dumps('"$(printf '%s\n' "${TRACKS[@]}" | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))')"'))'),
  "document_ids": $(python3 -c 'import json; print(json.dumps('"$(printf '%s\n' "${DOC_IDS[@]}" | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))')"'))'),
  "final_statuses": {k: v for k, v in $(python3 - <<'INNER'
print("{}")
INNER
)},
}
print(json.dumps(summary, indent=2))
PY

  # Rewrite summary with DONE map reliably
  python3 - <<PY
import json, pathlib
p = pathlib.Path("$RUN_DIR/summary.json")
s = json.loads(p.read_text())
done = {}
PY

  # Simpler final summary writer
  {
    echo "{"
    echo "  \"arm\": \"$ARM\","
    echo "  \"n\": $N,"
    echo "  \"base_url\": \"$BASE_URL\","
    echo "  \"admit_s\": $admit_s,"
    echo "  \"t_first_complete_s\": ${first_s:-null},"
    echo "  \"t_all_complete_s\": ${all_s:-null},"
    echo "  \"docs_per_min\": ${docs_per_min},"
    echo "  \"max_concurrent_processing_observed\": $max_proc,"
    echo "  \"run_dir\": \"$RUN_DIR\""
    echo "}"
  } | tee "$RUN_DIR/summary.json"

  echo "DONE summary → $RUN_DIR/summary.json"
}

main "$@"
