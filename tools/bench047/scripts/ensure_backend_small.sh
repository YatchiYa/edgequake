#!/usr/bin/env bash
# SPEC-047: keep EdgeQuake on Mistral Small + mistral-embed (fail-closed if wrong).
# Law: edgequake-llm MistralProvider::from_env() reads MISTRAL_MODEL (default medium-3-5).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$REPO_ROOT"

API_URL="${EDGEQUAKE_API_URL:-http://127.0.0.1:8090}"
PID_FILE="${EDGEQUAKE_BACKEND_PID_FILE:-/tmp/edgequake-backend.pid}"
LOG_FILE="${EDGEQUAKE_BACKEND_LOG:-/tmp/edgequake-backend.log}"
START_SH="${EDGEQUAKE_START_SH:-/tmp/edgequake-start.sh}"
WATCHDOG_PID_FILE="/tmp/edgequake-bench047-watchdog.pid"
WATCHDOG_LOG="/tmp/edgequake-bench047-watchdog.log"
START_LOCK="/tmp/edgequake-bench047-start.lock"

export MISTRAL_MODEL="${MISTRAL_MODEL:-mistral-small-latest}"
export EDGEQUAKE_LLM_PROVIDER="${EDGEQUAKE_LLM_PROVIDER:-mistral}"
export EDGEQUAKE_LLM_MODEL="${EDGEQUAKE_LLM_MODEL:-mistral-small-latest}"
export EDGEQUAKE_VISION_PROVIDER="${EDGEQUAKE_VISION_PROVIDER:-mistral}"
export EDGEQUAKE_VISION_MODEL="${EDGEQUAKE_VISION_MODEL:-mistral-small-latest}"
export EDGEQUAKE_EMBEDDING_PROVIDER="${EDGEQUAKE_EMBEDDING_PROVIDER:-mistral}"
export MISTRAL_EMBEDDING_MODEL="${MISTRAL_EMBEDDING_MODEL:-mistral-embed}"
export VLM_PROCESS_ENABLE="${VLM_PROCESS_ENABLE:-true}"
export EDGEQUAKE_AUTH_ENABLED="${EDGEQUAKE_AUTH_ENABLED:-false}"

die() { echo "ERROR: $*" >&2; exit 1; }

require_keys() {
  [ -n "${MISTRAL_API_KEY:-}" ] || die "MISTRAL_API_KEY unset"
}

health_json() {
  curl -fsS -m 3 "${API_URL}/health" 2>/dev/null || true
}

assert_small() {
  local h="$1"
  python3 - "$h" <<'PY'
import json,sys
raw=sys.argv[1].strip()
if not raw:
    raise SystemExit(2)
h=json.loads(raw)
llm=(h.get("providers") or {}).get("llm") or {}
emb=(h.get("providers") or {}).get("embedding") or {}
ok = (
    h.get("status") in {"healthy","degraded"}
    and llm.get("name")=="mistral"
    and "small" in str(llm.get("model","")).lower()
    and emb.get("name")=="mistral"
    and int(emb.get("dimension") or 0)==1024
)
print(json.dumps({"ok": ok, "llm": llm, "emb": emb, "status": h.get("status")}))
raise SystemExit(0 if ok else 3)
PY
}

# ≥10 parallel document ingestions per workspace require admission:
# WORKER_THREADS ⊃ MAX_TASKS_PER_TENANT ⊃ PDF_VISION_JOBS
export BENCH047_WORKER_THREADS="${BENCH047_WORKER_THREADS:-24}"
export BENCH047_MAX_TASKS_PER_TENANT="${BENCH047_MAX_TASKS_PER_TENANT:-16}"
export BENCH047_PDF_VISION_JOBS="${BENCH047_PDF_VISION_JOBS:-12}"
export BENCH047_PDF_CONCURRENCY="${BENCH047_PDF_CONCURRENCY:-4}"
export BENCH047_MM_IMAGE_CONCURRENCY="${BENCH047_MM_IMAGE_CONCURRENCY:-8}"

ensure_start_sh() {
  [ -x "$START_SH" ] || die "missing $START_SH — run: make backend-bg DEV_AUTH_ENABLED=false (then stop the proc)"
  # LLM stays Small (Acc chain). Vision pin respects EDGEQUAKE_VISION_MODEL
  # so stronger-vision ablations (025 / mistral-medium-3-5) are not clobbered.
  VISION_PIN="${EDGEQUAKE_VISION_MODEL:-mistral-small-latest}"
  perl -i -pe 's/export MISTRAL_MODEL=.*/export MISTRAL_MODEL="mistral-small-latest"/' "$START_SH" || true
  if ! grep -q 'MISTRAL_MODEL=' "$START_SH"; then
    perl -i -pe 's|^exec |export MISTRAL_MODEL="mistral-small-latest"\nexec |' "$START_SH"
  fi
  perl -i -pe 's/export EDGEQUAKE_LLM_MODEL=.*/export EDGEQUAKE_LLM_MODEL="mistral-small-latest"/' "$START_SH"
  if grep -q 'EDGEQUAKE_VISION_MODEL=' "$START_SH"; then
    perl -i -pe "s/export EDGEQUAKE_VISION_MODEL=.*/export EDGEQUAKE_VISION_MODEL=\"${VISION_PIN}\"/" "$START_SH"
  else
    perl -i -pe "s|^exec |export EDGEQUAKE_VISION_MODEL=\"${VISION_PIN}\"\nexec |" "$START_SH"
  fi
  echo "ensure_start_sh: LLM=mistral-small-latest VISION=${VISION_PIN}"
  grep -q 'VLM_PROCESS_ENABLE="true"' "$START_SH" || \
    perl -i -pe 's|^exec |export VLM_PROCESS_ENABLE="true"\nexec |' "$START_SH"
  # Parallel ingest admission (≥10 docs / workspace)
  for pair in \
    "WORKER_THREADS=${BENCH047_WORKER_THREADS}" \
    "MAX_TASKS_PER_TENANT=${BENCH047_MAX_TASKS_PER_TENANT}" \
    "EDGEQUAKE_PDF_VISION_JOBS=${BENCH047_PDF_VISION_JOBS}" \
    "EDGEQUAKE_PDF_CONCURRENCY=${BENCH047_PDF_CONCURRENCY}" \
    "EDGEQUAKE_MM_IMAGE_CONCURRENCY=${BENCH047_MM_IMAGE_CONCURRENCY}"
  do
    key="${pair%%=*}"
    val="${pair#*=}"
    if grep -q "export ${key}=" "$START_SH"; then
      perl -i -pe "s/export ${key}=.*/export ${key}=\"${val}\"/" "$START_SH"
    else
      perl -i -pe "s|^exec |export ${key}=\"${val}\"\nexec |" "$START_SH"
    fi
  done
}

port_free() {
  local port
  port="$(python3 - <<'PY'
from pathlib import Path
import re
text=Path("/tmp/edgequake-start.sh").read_text() if Path("/tmp/edgequake-start.sh").exists() else ""
m=re.search(r'PORT="(\d+)"', text)
print(m.group(1) if m else "8090")
PY
)"
  if lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
    lsof -nP -iTCP:"$port" -sTCP:LISTEN -t | xargs kill -9 2>/dev/null || true
    sleep 1
  fi
}

start_once() {
  ensure_start_sh
  port_free
  : >> "$LOG_FILE"
  python3 - "$START_SH" "$LOG_FILE" "$PID_FILE" <<'PY'
import os, sys, time
start_sh, log_file, pid_file = sys.argv[1:4]
if os.fork() > 0:
    time.sleep(0.3)
    sys.exit(0)
os.setsid()
if os.fork() > 0:
    sys.exit(0)
log = open(log_file, "a", buffering=1)
os.dup2(log.fileno(), 1)
os.dup2(log.fileno(), 2)
with open(pid_file, "w") as f:
    f.write(str(os.getpid()) + "\n")
os.execv("/bin/bash", ["bash", start_sh])
PY
  sleep 2
  local real
  real="$(pgrep -n -f 'target/(debug|release)/edgequake' || true)"
  if [ -n "$real" ]; then
    echo "$real" > "$PID_FILE"
    echo "$real"
  else
    cat "$PID_FILE"
  fi
}

wait_healthy_small() {
  # AGE M083 / graph bootstrap can take several minutes — do not flap-kill.
  local i h
  for i in $(seq 1 180); do
    h="$(health_json)"
    if [ -n "$h" ] && assert_small "$h" >/tmp/eq-small-check.json; then
      cat /tmp/eq-small-check.json
      return 0
    fi
    if pgrep -f 'target/(debug|release)/edgequake' >/dev/null 2>&1; then
      sleep 3
      continue
    fi
    sleep 2
  done
  return 1
}

acquire_start_lock() {
  python3 - "$START_LOCK" <<'PY' || die "could not acquire start lock"
import os, sys, time
path = sys.argv[1]
try:
    age = time.time() - os.path.getmtime(path)
    if age > 600:
        os.remove(path)
except FileNotFoundError:
    pass
deadline = time.time() + 180
while time.time() < deadline:
    try:
        fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o644)
        os.write(fd, str(os.getpid()).encode())
        os.close(fd)
        sys.exit(0)
    except FileExistsError:
        time.sleep(1)
sys.exit(1)
PY
}

release_start_lock() { rm -f "$START_LOCK"; }

ensure_now() {
  require_keys
  local h
  h="$(health_json)"
  if [ -n "$h" ] && assert_small "$h" >/tmp/eq-small-check.json; then
    echo "backend already Small-ok: $(cat /tmp/eq-small-check.json)"
    return 0
  fi
  if pgrep -f 'target/(debug|release)/edgequake' >/dev/null 2>&1; then
    echo "backend booting — waiting for Small health…"
    wait_healthy_small && return 0
  fi
  acquire_start_lock
  h="$(health_json)"
  if [ -n "$h" ] && assert_small "$h" >/tmp/eq-small-check.json; then
    echo "backend already Small-ok (after lock): $(cat /tmp/eq-small-check.json)"
    release_start_lock
    return 0
  fi
  if [ -f "$PID_FILE" ]; then kill -9 "$(cat "$PID_FILE")" 2>/dev/null || true; fi
  pkill -9 -f 'target/debug/edgequake' 2>/dev/null || true
  pkill -9 -f 'target/release/edgequake' 2>/dev/null || true
  sleep 1
  echo "starting Small backend…"
  set +e
  start_once
  wait_healthy_small
  local rc=$?
  set -e
  release_start_lock
  if [ "$rc" -ne 0 ]; then
    echo "failed to reach mistral-small-latest health" >&2
    tail -40 "$LOG_FILE" >&2 || true
    return 1
  fi
  return 0
}

watchdog_loop() {
  require_keys
  ensure_now
  echo "watchdog running pid=$$ api=$API_URL"
  while true; do
    h="$(health_json)"
    if [ -z "$h" ] || ! assert_small "$h" >/tmp/eq-small-check.json; then
      # Never restart while a boot is in progress
      if pgrep -f 'target/(debug|release)/edgequake' >/dev/null 2>&1; then
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) BOOTING wait"
      else
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) RESTART health_bad"
        ensure_now || true
      fi
    else
      echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) OK $(cat /tmp/eq-small-check.json)"
    fi
    sleep 30
  done
}

start_watchdog() {
  if [ -f "$WATCHDOG_PID_FILE" ] && kill -0 "$(cat "$WATCHDOG_PID_FILE")" 2>/dev/null; then
    echo "watchdog already running pid=$(cat "$WATCHDOG_PID_FILE")"
    ensure_now
    return 0
  fi
  python3 - "$0" "$WATCHDOG_LOG" "$WATCHDOG_PID_FILE" <<'PY'
import os, sys, time
script, log_file, pid_file = sys.argv[1:4]
if os.fork() > 0:
    time.sleep(0.3)
    sys.exit(0)
os.setsid()
if os.fork() > 0:
    sys.exit(0)
with open(pid_file, "w") as f:
    f.write(str(os.getpid()) + "\n")
log = open(log_file, "a", buffering=1)
os.dup2(log.fileno(), 1)
os.dup2(log.fileno(), 2)
os.execv("/bin/bash", ["bash", script, "watchdog-loop"])
PY
  sleep 2
  ensure_now
}

force_restart_parallel() {
  # Always restart so new WORKER_THREADS / PDF_VISION_JOBS take effect.
  require_keys
  acquire_start_lock
  ensure_start_sh
  echo "admission pins: WORKER_THREADS=$BENCH047_WORKER_THREADS MAX_TASKS=$BENCH047_MAX_TASKS_PER_TENANT PDF_VISION_JOBS=$BENCH047_PDF_VISION_JOBS"
  if [ -f "$PID_FILE" ]; then kill -9 "$(cat "$PID_FILE")" 2>/dev/null || true; fi
  pkill -9 -f 'target/debug/edgequake' 2>/dev/null || true
  pkill -9 -f 'target/release/edgequake' 2>/dev/null || true
  sleep 1
  echo "starting Small backend (parallel admission)…"
  set +e
  start_once
  wait_healthy_small
  local rc=$?
  set -e
  release_start_lock
  if [ "$rc" -ne 0 ]; then
    echo "failed to reach mistral-small-latest health after parallel restart" >&2
    tail -40 "$LOG_FILE" >&2 || true
    return 1
  fi
  return 0
}

case "${1:-ensure}" in
  ensure) ensure_now ;;
  restart-parallel) force_restart_parallel ;;
  start-watchdog) start_watchdog ;;
  watchdog-loop) watchdog_loop ;;
  status)
    h="$(health_json)"
    if [ -n "$h" ]; then assert_small "$h"; else echo '{"ok":false,"error":"down"}'; exit 1; fi
    ;;
  *) die "usage: $0 {ensure|restart-parallel|start-watchdog|status}" ;;
esac
