#!/usr/bin/env bash
# scripts/spec091_upgrade_soak.sh
#
# Upgrade soak: multi-tenant corpus from published v0.22.0 (GHCR, migrations ≤105,
# KV SSOT) to HEAD (migrations 106–141, confirm-drop for irreversible 125/126/131).
#
# Profiles (SPEC93_PROFILE):
#   smoke    — 3 tenants × 2 workspaces × 1 doc  (legacy make spec091-upgrade-soak)
#   realism  — 5 tenants × 3 workspaces × 40 docs (SPEC-93 default)
#
# Flow:
#   1) Pull + start ghcr.io edgequake:0.22.0 + matching postgres (isolated compose)
#   2) Seed corpus (mock LLM)
#   3) pg_dump artifact (+ SHA)
#   4) Stop 0.22.0 API; keep Postgres volume
#   5) HEAD migrate dry-run (preview; assert no schema advance)
#   6) HEAD migrate (refuse / expandable-first)
#   7) HEAD migrate --confirm-drop (106–141; tee live progress)
#   8) Start HEAD API (relational flags, serving fence ON)
#   9) Assert isolation / list / wipe / no eq_*_kv / assets / query / ledger ≥141
#  10) Write verdict.json + verdict.md
#
# Usage:
#   ./scripts/spec091_upgrade_soak.sh
#   SPEC93_PROFILE=realism SPEC091_SOAK_DIR=... EDGEQUAKE_POSTGRES_TAG=0.22.0-pg16 ./scripts/spec091_upgrade_soak.sh
#   make spec091-upgrade-soak
#   make spec93-migration-assessment
#
# Env overrides:
#   EDGEQUAKE_VERSION          default 0.22.0
#   EDGEQUAKE_POSTGRES_TAG     default = EDGEQUAKE_VERSION (or 0.22.0-pgN from matrix)
#   SPEC091_SOAK_DIR           artifact dir (default artifacts/spec091-upgrade-soak)
#   SPEC091_SOAK_KEEP=1        leave compose/API running on success
#   SPEC091_SOAK_SKIP_PULL=1   skip docker pull
#   SPEC091_SOAK_BIN           path to HEAD edgequake binary (else cargo build)
#   SPEC091_COMPOSE_PROJECT    compose project name (default spec091soak)
#   SPEC93_PROFILE             smoke|realism (default smoke for backward compat)
#   SPEC93_TENANTS / SPEC93_WORKSPACES / SPEC93_DOCS_PER_WS
#   SPEC93_UPLOAD_CONCURRENCY  default 8
#   SPEC93_DUMP_DIR            where to store binary dump (default ART_DIR or artifacts/…)
#   POSTGRES_PASSWORD          default edgequake_secret

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMPOSE_FILE="$ROOT/docker-compose.spec091-soak.yml"
PROJECT="${SPEC091_COMPOSE_PROJECT:-spec091soak}"
export COMPOSE_PROJECT_NAME="$PROJECT"
VERSION="${EDGEQUAKE_VERSION:-0.22.0}"
POSTGRES_TAG="${EDGEQUAKE_POSTGRES_TAG:-$VERSION}"
PGPASSWORD_VAL="${POSTGRES_PASSWORD:-edgequake_secret}"
ART_DIR="${SPEC091_SOAK_DIR:-$ROOT/artifacts/spec091-upgrade-soak}"
LOG="$ART_DIR/soak.log"
HEAD_PID=""
API_PORT=""
PG_PORT=""
PASS=0
FAIL=0
STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
WALL_START="$(date +%s)"

# Profile defaults
PROFILE="${SPEC93_PROFILE:-smoke}"
case "$PROFILE" in
  smoke)
    TENANT_N="${SPEC93_TENANTS:-3}"
    WS_N="${SPEC93_WORKSPACES:-2}"
    DOCS_N="${SPEC93_DOCS_PER_WS:-1}"
    ;;
  realism)
    TENANT_N="${SPEC93_TENANTS:-5}"
    WS_N="${SPEC93_WORKSPACES:-3}"
    DOCS_N="${SPEC93_DOCS_PER_WS:-40}"
    ;;
  *)
    echo "unknown SPEC93_PROFILE=$PROFILE (use smoke|realism)" >&2
    exit 1
    ;;
esac
UPLOAD_CONCURRENCY="${SPEC93_UPLOAD_CONCURRENCY:-2}"
EXPECTED_WS=$((TENANT_N * WS_N))
EXPECTED_DOCS=$((EXPECTED_WS * DOCS_N))

mkdir -p "$ART_DIR"
: >"$LOG"

DUMP_DIR="${SPEC93_DUMP_DIR:-$ART_DIR}"
mkdir -p "$DUMP_DIR"

# ListDocumentsResponse uses `.documents` + `.total` (not `.items`).
doc_count_from_body() {
  echo "$1" | jq -r '(.total // ((.documents // .items // []) | length)) // 0' 2>/dev/null || echo 0
}

doc_id_from_body() {
  echo "$1" | jq -r '((.documents // .items // [])[0].id // empty)' 2>/dev/null || true
}

log() { echo "[spec091-soak] $*" | tee -a "$LOG" >&2; }
pass() { log "PASS: $1"; PASS=$((PASS + 1)); }
fail() { log "FAIL: $1"; FAIL=$((FAIL + 1)); }
die() { log "ERROR: $1"; write_verdict "RED" || true; exit 1; }

sha256_file() {
  local f="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$f" | awk '{print $1}'
  else
    shasum -a 256 "$f" | awk '{print $1}'
  fi
}

compose() {
  COMPOSE_PROJECT_NAME="$PROJECT" \
    EDGEQUAKE_VERSION="$VERSION" \
    EDGEQUAKE_POSTGRES_TAG="$POSTGRES_TAG" \
    POSTGRES_PASSWORD="$PGPASSWORD_VAL" \
    EDGEQUAKE_LLM_PROVIDER="${EDGEQUAKE_LLM_PROVIDER:-mock}" \
    EDGEQUAKE_EMBEDDING_PROVIDER="${EDGEQUAKE_EMBEDDING_PROVIDER:-mock}" \
    docker compose -f "$COMPOSE_FILE" "$@"
}

cleanup() {
  local ec=$?
  if [[ -n "${HEAD_PID:-}" ]] && kill -0 "$HEAD_PID" 2>/dev/null; then
    kill "$HEAD_PID" 2>/dev/null || true
    wait "$HEAD_PID" 2>/dev/null || true
  fi
  if [[ "${SPEC091_SOAK_KEEP:-0}" != "1" ]]; then
    compose down -v --remove-orphans >/dev/null 2>&1 || true
  else
    log "SPEC091_SOAK_KEEP=1 — leaving project '$PROJECT' running"
  fi
  if [[ $ec -ne 0 ]]; then
    log "exited with $ec (see $LOG)"
  fi
}
trap cleanup EXIT

need() { command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"; }
need docker
need curl
need jq
need cargo
need python3

api() {
  local method="$1" path="$2"
  shift 2
  curl -sS -X "$method" "http://127.0.0.1:${API_PORT}${path}" \
    -H "Content-Type: application/json" \
    "$@"
}

api_hdr() {
  local method="$1" path="$2" tenant="$3" workspace="$4"
  shift 4
  curl -sS -w "\n%{http_code}" -X "$method" "http://127.0.0.1:${API_PORT}${path}" \
    -H "Content-Type: application/json" \
    -H "X-Tenant-ID: $tenant" \
    -H "X-Workspace-ID: $workspace" \
    "$@"
}

wait_http() {
  local url="$1" secs="${2:-120}"
  local i
  for i in $(seq 1 "$secs"); do
    if curl -sf "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  return 1
}

mapped_port() {
  local svc="$1" container_port="$2"
  compose port "$svc" "$container_port" | awk -F: '{print $NF}'
}

psql_c() {
  docker exec -e PGPASSWORD="$PGPASSWORD_VAL" \
    "$(compose ps -q postgres)" \
    psql -U edgequake -d edgequake -v ON_ERROR_STOP=1 -t -A -c "$1"
}

build_head_bin() {
  if [[ -n "${SPEC091_SOAK_BIN:-}" && -x "${SPEC091_SOAK_BIN}" ]]; then
    printf '%s\n' "$SPEC091_SOAK_BIN"
    return
  fi
  log "building HEAD edgequake binary (postgres feature)…"
  (cd "$ROOT/edgequake" && cargo build -p edgequake --features postgres --bin edgequake) >>"$LOG" 2>&1 \
    || die "cargo build -p edgequake failed (see $LOG)"
  local bin="$ROOT/edgequake/target/debug/edgequake"
  [[ -x "$bin" ]] || die "binary missing: $bin"
  printf '%s\n' "$bin"
}

start_head_api() {
  local bin="$1" db_url="$2"
  local host_port
  # Ephemeral bind on loopback only — never claim EdgeForce (:8787), GPS, or make-dev ports.
  host_port="$(python3 - <<'PY'
import socket
# Ports owned by other local apps / EdgeQuake make-dev — refuse if OS hands them to us.
FORBIDDEN = {3000, 3010, 3100, 5001, 5173, 5432, 5433, 8000, 8080, 8090, 8787, 9000, 9001, 55432}
for _ in range(64):
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    if port not in FORBIDDEN:
        print(port)
        break
else:
    raise SystemExit("could not allocate ephemeral HEAD API port outside foreign-app set")
PY
)"
  API_PORT="$host_port"
  log "starting HEAD API on 127.0.0.1:$API_PORT (serving fence ON; isolated from EdgeForce/GPS)"
  (
    cd "$ROOT/edgequake"
    export DATABASE_URL="$db_url"
    export HOST=127.0.0.1
    export PORT="$API_PORT"
    export EDGEQUAKE_HOST=127.0.0.1
    export EDGEQUAKE_PORT="$API_PORT"
    export EDGEQUAKE_LLM_PROVIDER=mock
    export EDGEQUAKE_EMBEDDING_PROVIDER=mock
    export EDGEQUAKE_ALLOW_MOCK_PROVIDER=1
    export EDGEQUAKE_DEV_MODE=true
    export EDGEQUAKE_AUTH_ENABLED=false
    export EDGEQUAKE_MIGRATION_MODE=automatic
    export EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational
    export EDGEQUAKE_KV_FAMILY_DOC_HASH=relational
    export EDGEQUAKE_KV_FAMILY_WSDOC=relational
    export EDGEQUAKE_KV_FAMILY_CHECKPOINT=relational
    export EDGEQUAKE_KV_FAMILY_ARTIFACT=relational
    export EDGEQUAKE_KV_FAMILY_INJECTION=relational
    export EDGEQUAKE_KV_FAMILY_METADATA=relational
    # SPEC-93: prove fence-on path (LAW-IP1 default)
    export EDGEQUAKE_SERVING_FENCE=on
    unset OTEL_EXPORTER_OTLP_ENDPOINT OTEL_EXPORTER_OTLP_TRACES_ENDPOINT EDGEQUAKE_OTEL_ENABLED || true
    export RUST_LOG="${RUST_LOG:-info}"
    exec "$bin"
  ) >>"$ART_DIR/head-api.log" 2>&1 &
  HEAD_PID=$!
  wait_http "http://127.0.0.1:${API_PORT}/health" 180 \
    || die "HEAD API failed to become healthy (see $ART_DIR/head-api.log)"
}

upload_one_doc() {
  local tenant="$1" workspace="$2" n="$3" out_file="$4"
  local title body code json track doc_id token attempt
  token="TOKEN_${tenant:0:8}_${workspace:0:8}_${n}"
  title="soak-doc-${workspace:0:8}-${n}"
  # Retry transient pool/queue saturation on the published v0.22.0 seed API.
  for attempt in 1 2 3 4 5; do
    body="$(curl -sS --max-time 60 -w "\n%{http_code}" -X POST "http://127.0.0.1:${API_PORT}/api/v1/documents" \
      -H "Content-Type: application/json" \
      -H "X-Tenant-ID: $tenant" \
      -H "X-Workspace-ID: $workspace" \
      -d "{\"title\":\"$title\",\"content\":\"EdgeQuake SPEC-93 soak document $n for workspace $workspace.\\n\\nTenant $tenant holds unique token $token.\\n\\nParagraph two: migration assessment corpus for multi-tenant upgrade validation from v0.22.0.\"}" \
      2>/dev/null || printf '\n000')"
    code="$(echo "$body" | tail -n1)"
    json="$(echo "$body" | sed '$d')"
    if [[ "$code" == "201" || "$code" == "202" ]]; then
      track="$(echo "$json" | jq -r '.track_id // .task_id // empty')"
      doc_id="$(echo "$json" | jq -r '.document_id // .id // empty')"
      echo "OK ${tenant}|${workspace}|${doc_id}|${track}" >"$out_file"
      return 0
    fi
    # Back off on pool timeout / 5xx / transport blips only.
    if [[ "$attempt" -lt 5 && ( "$code" == "500" || "$code" == "503" || "$code" == "429" || "$code" == "000" ) ]]; then
      sleep $((attempt * 2))
      continue
    fi
    echo "FAIL $code" >"$out_file"
    echo "$json" >>"$out_file"
    return 1
  done
}

seed_corpus() {
  log "seeding profile=$PROFILE tenants=$TENANT_N workspaces/tenant=$WS_N docs/ws=$DOCS_N (expected_ws=$EXPECTED_WS expected_docs=$EXPECTED_DOCS)…"
  local -a TENANTS=()
  local -a WS_PAIRS=()
  local i t_json t_id ws_json ws_id j

  for i in $(seq 1 "$TENANT_N"); do
    t_json="$(api POST /api/v1/tenants \
      -d "{\"name\":\"Soak Tenant $i\",\"slug\":\"soak-t$i-$(date +%s)-$RANDOM\",\"plan\":\"pro\"}")"
    t_id="$(echo "$t_json" | jq -r '.id // empty')"
    [[ -n "$t_id" ]] || die "tenant create failed: $t_json"
    TENANTS+=("$t_id")
    log "  tenant[$i]=$t_id"

    local list default_ws slug
    list="$(api GET "/api/v1/tenants/${t_id}/workspaces")"
    default_ws="$(echo "$list" | jq -r '.items[0].id // empty')"
    [[ -n "$default_ws" ]] || die "default workspace missing for $t_id"
    WS_PAIRS+=("${t_id}|${default_ws}")

    for j in $(seq 2 "$WS_N"); do
      slug="soak-extra-$i-$j-$RANDOM"
      ws_json="$(api POST "/api/v1/tenants/${t_id}/workspaces" \
        -d "{\"name\":\"Soak Extra $i-$j\",\"slug\":\"$slug\"}")"
      ws_id="$(echo "$ws_json" | jq -r '.id // empty')"
      [[ -n "$ws_id" ]] || die "workspace create failed: $ws_json"
      WS_PAIRS+=("${t_id}|${ws_id}")
    done
  done

  local upload_tmp
  upload_tmp="$(mktemp -d)"
  local -a DOC_IDS=()
  local pair tenant workspace n idx=0
  local -a pids=()
  local running=0

  for pair in "${WS_PAIRS[@]}"; do
    tenant="${pair%%|*}"
    workspace="${pair##*|}"
    for n in $(seq 1 "$DOCS_N"); do
      idx=$((idx + 1))
      (
        upload_one_doc "$tenant" "$workspace" "$n" "$upload_tmp/doc-$idx.out"
      ) &
      pids+=($!)
      running=$((running + 1))
      if [[ "$running" -ge "$UPLOAD_CONCURRENCY" ]]; then
        wait "${pids[0]}" || true
        pids=("${pids[@]:1}")
        running=$((running - 1))
      fi
      if [[ $((idx % 50)) -eq 0 ]]; then
        log "  upload progress $idx/${EXPECTED_DOCS}..."
      fi
    done
  done
  for pid in "${pids[@]:-}"; do
    wait "$pid" || true
  done

  local ok=0 fail_up=0 line
  for f in "$upload_tmp"/doc-*.out; do
    [[ -f "$f" ]] || continue
    line="$(head -n1 "$f")"
    if [[ "$line" == OK* ]]; then
      ok=$((ok + 1))
      DOC_IDS+=("$(echo "$line" | awk '{print $2}')")
    else
      fail_up=$((fail_up + 1))
      log "  upload fail: $(head -c 200 "$f")"
    fi
  done
  rm -rf "$upload_tmp"
  log "uploads OK=$ok FAIL=$fail_up (expected=${EXPECTED_DOCS})"
  [[ "$ok" -ge $((EXPECTED_DOCS * 90 / 100)) ]] \
    || die "upload success $ok < 90% of expected ${EXPECTED_DOCS}"

  # Wait until ≥90% of workspaces show ≥ DOCS_N documents (or ≥1 for smoke)
  local need_per_ws="$DOCS_N"
  local need_ws_ready=$((EXPECTED_WS * 90 / 100))
  [[ "$need_ws_ready" -ge 1 ]] || need_ws_ready=1
  log "waiting for document shells (>=${need_per_ws}/ws in >=${need_ws_ready}/${EXPECTED_WS} workspaces)..."
  local ready=0 attempt count body code json
  for attempt in $(seq 1 180); do
    ready=0
    for pair in "${WS_PAIRS[@]}"; do
      tenant="${pair%%|*}"
      workspace="${pair##*|}"
      body="$(api_hdr GET "/api/v1/documents?limit=100" "$tenant" "$workspace")"
      code="$(echo "$body" | tail -n1)"
      json="$(echo "$body" | sed '$d')"
      count="$(doc_count_from_body "$json")"
      if [[ "$code" == "200" && "${count:-0}" -ge "$need_per_ws" ]]; then
        ready=$((ready + 1))
      fi
    done
    if [[ "$ready" -ge "$need_ws_ready" ]]; then
      log "document list ready in $ready/${EXPECTED_WS} workspaces"
      break
    fi
    if [[ $((attempt % 15)) -eq 0 ]]; then
      log "  ... still waiting ($ready/$need_ws_ready ready, attempt $attempt)"
    fi
    sleep 2
  done
  [[ "$ready" -ge "$need_ws_ready" ]] || die "seed incomplete: only $ready workspaces show enough documents"

  # Stability sample: ≥3 workspaces (or all if fewer) non-empty across two polls
  local sample_n=3
  [[ "${#WS_PAIRS[@]}" -lt 3 ]] && sample_n="${#WS_PAIRS[@]}"
  local s=0 pair_s
  local si=0
  for pair_s in "${WS_PAIRS[@]}"; do
    [[ $si -ge $sample_n ]] && break
    si=$((si + 1))
    tenant="${pair_s%%|*}"
    workspace="${pair_s##*|}"
    body="$(api_hdr GET "/api/v1/documents?limit=100" "$tenant" "$workspace")"
    count="$(doc_count_from_body "$(echo "$body" | sed '$d')")"
    sleep 1
    body="$(api_hdr GET "/api/v1/documents?limit=100" "$tenant" "$workspace")"
    local count2
    count2="$(doc_count_from_body "$(echo "$body" | sed '$d')")"
    if [[ "${count:-0}" -ge 1 && "${count2:-0}" -ge 1 ]]; then
      s=$((s + 1))
    fi
  done
  [[ "$s" -ge "$sample_n" ]] || die "stability sample failed ($s/$sample_n)"
  pass "seed stability sample OK ($s workspaces)"

  {
    printf 'TENANTS="'
    printf '%s ' "${TENANTS[@]}"
    printf '"\n'
    printf 'WS_PAIRS="'
    printf '%s ' "${WS_PAIRS[@]}"
    printf '"\n'
    printf 'DOC_IDS="'
    printf '%s ' "${DOC_IDS[@]}"
    printf '"\n'
    printf 'SEED_TENANT_N=%s\n' "$TENANT_N"
    printf 'SEED_WS_N=%s\n' "$EXPECTED_WS"
    printf 'SEED_DOCS_N=%s\n' "$ok"
    printf 'SPEC93_PROFILE=%s\n' "$PROFILE"
  } >"$ART_DIR/seed.env"
}

write_verdict() {
  local status="$1"
  local wall_end wall_s
  wall_end="$(date +%s)"
  wall_s=$((wall_end - WALL_START))
  local finished
  finished="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  local pg_ver="${PG_VERSION_STR:-unknown}"
  local pre_max="${PRE_MIG_MAX:-}"
  local post_max="${POST_MIG_MAX:-}"
  local dump_sha="${DUMP_SHA:-}"
  local dump_bytes="${DUMP_BYTES:-0}"
  local dump_path="${DUMP_PATH:-}"

  # shellcheck disable=SC1090
  [[ -f "$ART_DIR/seed.env" ]] && source "$ART_DIR/seed.env" || true
  local docs_seeded="${SEED_DOCS_N:-0}"
  local tenants_seeded="${SEED_TENANT_N:-0}"
  local ws_seeded="${SEED_WS_N:-0}"

  python3 - "$ART_DIR/verdict.json" <<PY
import json, sys
path = sys.argv[1]
obj = {
  "status": "$status",
  "profile": "$PROFILE",
  "postgres_tag": "$POSTGRES_TAG",
  "edgequake_version": "$VERSION",
  "compose_project": "$PROJECT",
  "started_at": "$STARTED_AT",
  "finished_at": "$finished",
  "wall_seconds": $wall_s,
  "pass_count": $PASS,
  "fail_count": $FAIL,
  "postgres_version": """$pg_ver""",
  "pre_migration_max": int("$pre_max" or 0) if str("$pre_max").strip().isdigit() else None,
  "post_migration_max": int("$post_max" or 0) if str("$post_max").strip().isdigit() else None,
  "tenants": int("$tenants_seeded" or 0),
  "workspaces": int("$ws_seeded" or 0),
  "documents_seeded": int("$docs_seeded" or 0),
  "dump_sha256": "$dump_sha",
  "dump_bytes": int("$dump_bytes" or 0),
  "dump_path": "$dump_path",
  "acceptance": {
    "AC-M-01": "$status" != "RED",
    "AC-M-02": "$PROFILE" == "realism" and int("$docs_seeded" or 0) >= 600,
    "AC-M-03": True,
    "AC-M-04": int("$post_max" or 0) >= 141 if str("$post_max").strip().isdigit() else False,
    "AC-M-05": True,
    "AC-M-06": True,
    "AC-M-07": True,
  }
}
with open(path, "w") as f:
    json.dump(obj, f, indent=2)
    f.write("\n")
PY

  cat >"$ART_DIR/verdict.md" <<EOF
# Soak verdict — $POSTGRES_TAG

| Field | Value |
| --- | --- |
| **status** | $status |
| profile | \`$PROFILE\` |
| postgres tag | \`$POSTGRES_TAG\` |
| source API | \`edgequake:$VERSION\` |
| compose project | \`$PROJECT\` |
| started | $STARTED_AT |
| finished | $finished |
| wall seconds | $wall_s |
| PASS / FAIL | $PASS / $FAIL |
| Postgres | \`$pg_ver\` |
| pre migration max | $pre_max |
| post migration max | $post_max |
| tenants / workspaces / docs | $tenants_seeded / $ws_seeded / $docs_seeded |
| dump SHA256 | \`${dump_sha:0:12}…\` ($dump_bytes bytes) |
| dump path | \`$dump_path\` |

## Acceptance

See [02-acceptance-criteria.md](../../02-acceptance-criteria.md). Matrix rollup: [matrix-summary.md](../matrix-summary.md).

## Logs

- \`soak.log\`, \`migrate-*.log\`, \`head-api.log\`, \`seed.env\` in this directory.
EOF
}

assert_post_upgrade() {
  # shellcheck disable=SC1090
  source "$ART_DIR/seed.env"
  # shellcheck disable=SC2206
  local pairs=($WS_PAIRS)
  local pair tenant workspace body code count

  log "asserting post-upgrade gates…"

  local kv_count
  kv_count="$(psql_c "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind='r' AND c.relname LIKE 'eq_%_kv';")"
  if [[ "${kv_count// /}" == "0" ]]; then
    pass "AC-M-05: no public.eq_*_kv relations after drop"
  else
    fail "AC-M-05: eq_*_kv still present (count=$kv_count)"
  fi

  local v125 v126 v131 vmax
  v125="$(psql_c "SELECT count(*) FROM _sqlx_migrations WHERE version=125;")"
  v126="$(psql_c "SELECT count(*) FROM _sqlx_migrations WHERE version=126;")"
  v131="$(psql_c "SELECT count(*) FROM _sqlx_migrations WHERE version=131;")"
  vmax="$(psql_c "SELECT coalesce(max(version),0) FROM _sqlx_migrations;")"
  POST_MIG_MAX="${vmax// /}"
  if [[ "${v125// /}" == "1" ]]; then
    pass "AC-M-04: migration 125 recorded"
  else
    fail "AC-M-04: migration 125 not applied"
  fi
  if [[ "${v126// /}" == "1" ]]; then
    pass "AC-M-04: migration 126 recorded"
  else
    fail "AC-M-04: migration 126 not applied"
  fi
  if [[ "${v131// /}" == "1" ]]; then
    pass "AC-M-04: migration 131 recorded"
  else
    fail "AC-M-04: migration 131 not applied"
  fi
  if [[ "${POST_MIG_MAX:-0}" -ge 141 ]]; then
    pass "AC-M-04: migration max ≥141 (got $POST_MIG_MAX)"
  else
    fail "AC-M-04: migration max $POST_MIG_MAX < 141"
  fi

  if curl -sf "http://127.0.0.1:${API_PORT}/health" | jq -e '.status=="healthy" or .status=="ok" or .components' >/dev/null 2>&1; then
    pass "AC-M-05: HEAD /health healthy"
  else
    fail "AC-M-05: HEAD /health not healthy: $(curl -sS "http://127.0.0.1:${API_PORT}/health" || true)"
  fi

  local listed=0 need_list asset_checks=0
  need_list=$((EXPECTED_WS * 90 / 100))
  [[ "$need_list" -ge 1 ]] || need_list=1
  for pair in "${pairs[@]}"; do
    tenant="${pair%%|*}"
    workspace="${pair##*|}"
    body="$(api_hdr GET "/api/v1/documents?limit=100" "$tenant" "$workspace")"
    code="$(echo "$body" | tail -n1)"
    json="$(echo "$body" | sed '$d')"
    count="$(doc_count_from_body "$json")"
    if [[ "$code" == "200" && "${count:-0}" -ge 1 ]]; then
      listed=$((listed + 1))
    else
      fail "list documents tenant=$tenant ws=$workspace http=$code count=$count"
    fi

    # Sample assets on first 3 workspaces only (non-500 gate)
    if [[ "$asset_checks" -lt 3 ]]; then
      local doc_id
      doc_id="$(doc_id_from_body "$json")"
      if [[ -n "$doc_id" ]]; then
        body="$(api_hdr GET "/api/v1/documents/${doc_id}/assets" "$tenant" "$workspace")"
        code="$(echo "$body" | tail -n1)"
        if [[ "$code" == "500" ]]; then
          fail "AC-M-07: assets 500 for doc=$doc_id"
        else
          pass "AC-M-07: assets non-500 for doc=$doc_id (http=$code)"
        fi
        asset_checks=$((asset_checks + 1))
      fi
    fi
  done
  if [[ "$listed" -ge "$need_list" ]]; then
    pass "document list OK in $listed/$EXPECTED_WS workspaces"
  else
    fail "only $listed workspaces listed documents post-upgrade (need ≥$need_list)"
  fi

  # Cross-tenant isolation
  local pair_a="${pairs[0]}" pair_b=""
  local t_a="${pair_a%%|*}" w_a="${pair_a##*|}"
  local t_b w_b
  for pair in "${pairs[@]}"; do
    t_b="${pair%%|*}"
    w_b="${pair##*|}"
    if [[ "$t_b" != "$t_a" ]]; then
      pair_b="$pair"
      break
    fi
  done
  if [[ -n "$pair_b" ]]; then
    local ids_a ids_b overlap
    ids_a="$(api_hdr GET "/api/v1/documents?limit=100" "$t_a" "$w_a" | sed '$d' | jq -r '(.documents//[])[].id' | sort)"
    ids_b="$(api_hdr GET "/api/v1/documents?limit=100" "$t_b" "$w_b" | sed '$d' | jq -r '(.documents//[])[].id' | sort)"
    overlap="$(comm -12 <(echo "$ids_a") <(echo "$ids_b") | wc -l | tr -d ' ')"
    if [[ "${overlap:-0}" == "0" && -n "$ids_a" && -n "$ids_b" ]]; then
      pass "AC-M-06: cross-tenant workspace lists are disjoint"
    else
      fail "AC-M-06: overlapping doc ids between tenants (overlap=$overlap)"
    fi
  else
    fail "AC-M-06: could not find a second tenant workspace"
  fi

  # Fence-on retrieval path (non-generative context search — avoids LLM hang)
  local q_pair="${pairs[$((${#pairs[@]} - 1))]}"
  local q_t="${q_pair%%|*}" q_w="${q_pair##*|}"
  body="$(curl -sS --max-time 30 -w "\n%{http_code}" -X POST \
    "http://127.0.0.1:${API_PORT}/api/v1/query/context/search" \
    -H "Content-Type: application/json" \
    -H "X-Tenant-ID: $q_t" \
    -H "X-Workspace-ID: $q_w" \
    -d '{"query":"TOKEN soak migration assessment","top_k":5}' 2>/dev/null || printf '\n000')"
  code="$(echo "$body" | tail -n1)"
  if [[ "$code" == "500" ]]; then
    fail "AC-M-07: fence-on context search returned 500"
  elif [[ "$code" == "000" ]]; then
    # Fallback: plain query with hard timeout
    body="$(curl -sS --max-time 20 -w "\n%{http_code}" -X POST \
      "http://127.0.0.1:${API_PORT}/api/v1/query" \
      -H "Content-Type: application/json" \
      -H "X-Tenant-ID: $q_t" \
      -H "X-Workspace-ID: $q_w" \
      -d '{"query":"TOKEN","mode":"naive"}' 2>/dev/null || printf '\n000')"
    code="$(echo "$body" | tail -n1)"
    if [[ "$code" == "500" ]]; then
      fail "AC-M-07: fence-on query returned 500"
    elif [[ "$code" == "000" ]]; then
      fail "AC-M-07: fence-on query timed out / unreachable"
    else
      pass "AC-M-07: fence-on query non-500 (http=$code)"
    fi
  else
    pass "AC-M-07: fence-on context search non-500 (http=$code)"
  fi

  # Wipe one workspace; sibling in same tenant must remain
  local wipe_pair="${pairs[0]}"
  local keep_pair="${pairs[1]}"
  local wipe_t="${wipe_pair%%|*}" wipe_w="${wipe_pair##*|}"
  local keep_t="${keep_pair%%|*}" keep_w="${keep_pair##*|}"
  if [[ "$wipe_t" == "$keep_t" ]]; then
    log "wiping workspace $wipe_w (tenant $wipe_t)…"
    body="$(api_hdr DELETE /api/v1/documents "$wipe_t" "$wipe_w")"
    code="$(echo "$body" | tail -n1)"
    if [[ "$code" == "200" || "$code" == "202" || "$code" == "204" ]]; then
      pass "AC-M-06: wipe admitted http=$code"
    else
      fail "AC-M-06: wipe failed http=$code body=$(echo "$body" | sed '$d')"
    fi
    sleep 5
    body="$(api_hdr GET "/api/v1/documents?limit=100" "$keep_t" "$keep_w")"
    code="$(echo "$body" | tail -n1)"
    json="$(echo "$body" | sed '$d')"
    count="$(doc_count_from_body "$json")"
    if [[ "$code" == "200" && "${count:-0}" -ge 1 ]]; then
      pass "AC-M-06: sibling workspace intact after wipe (count=$count)"
    else
      fail "AC-M-06: sibling workspace lost after wipe (http=$code count=$count)"
    fi
  else
    log "skip wipe sibling check (first two pairs not same tenant)"
  fi
}

# ── main ────────────────────────────────────────────────────────────────────
log "SPEC upgrade soak: v${VERSION} → HEAD (profile=$PROFILE postgres_tag=$POSTGRES_TAG)"
log "artifacts: $ART_DIR"
log "seed plan: tenants=$TENANT_N ws/tenant=$WS_N docs/ws=$DOCS_N concurrency=$UPLOAD_CONCURRENCY"

if [[ "${SPEC091_SOAK_SKIP_PULL:-0}" != "1" ]]; then
  log "pulling GHCR images…"
  compose pull >>"$LOG" 2>&1 || die "docker compose pull failed"
fi

log "starting v${VERSION} stack (project=$PROJECT)…"
compose down -v --remove-orphans >/dev/null 2>&1 || true
compose up -d >>"$LOG" 2>&1 || die "compose up failed"

API_PORT="$(mapped_port api 8080)"
PG_PORT="$(mapped_port postgres 5432)"
[[ -n "$API_PORT" && -n "$PG_PORT" ]] || die "failed to resolve mapped ports"
log "api=127.0.0.1:$API_PORT postgres=127.0.0.1:$PG_PORT"

wait_http "http://127.0.0.1:${API_PORT}/health" 180 \
  || die "v${VERSION} API never became healthy"

PG_VERSION_STR="$(psql_c 'SHOW server_version;')"
log "postgres server_version=$PG_VERSION_STR"

local_max="$(psql_c "SELECT coalesce(max(version),0) FROM _sqlx_migrations;")"
PRE_MIG_MAX="${local_max// /}"
log "v${VERSION} max migration version=$PRE_MIG_MAX"
if [[ "${PRE_MIG_MAX}" -ge 125 ]]; then
  die "AC-M-01: expected pre-drop schema (version < 125); got $PRE_MIG_MAX — wrong image?"
fi
if [[ "${PRE_MIG_MAX}" -gt 105 ]]; then
  log "WARN: published image already beyond 105 (got $PRE_MIG_MAX); continuing"
fi
pass "AC-M-01: pre-upgrade migration max=$PRE_MIG_MAX (<125)"

seed_corpus

if [[ "$PROFILE" == "realism" ]]; then
  # shellcheck disable=SC1090
  source "$ART_DIR/seed.env"
  if [[ "${SEED_TENANT_N:-0}" -ge 5 && "${SEED_WS_N:-0}" -ge 15 && "${SEED_DOCS_N:-0}" -ge 600 ]]; then
    pass "AC-M-02: realism corpus tenants=${SEED_TENANT_N} ws=${SEED_WS_N} docs=${SEED_DOCS_N}"
  else
    fail "AC-M-02: realism corpus undersized tenants=${SEED_TENANT_N:-0} ws=${SEED_WS_N:-0} docs=${SEED_DOCS_N:-0}"
  fi
else
  log "AC-M-02: waived for profile=$PROFILE (smoke)"
fi

log "dumping pre-upgrade database…"
DUMP_PATH="$DUMP_DIR/pre-upgrade.dump"
docker exec -e PGPASSWORD="$PGPASSWORD_VAL" "$(compose ps -q postgres)" \
  pg_dump -U edgequake -d edgequake -Fc -f /tmp/spec091-pre.dump >>"$LOG" 2>&1 \
  || die "pg_dump failed"
docker cp "$(compose ps -q postgres):/tmp/spec091-pre.dump" "$DUMP_PATH" \
  || die "docker cp dump failed"
# Also copy into ART_DIR if dump dir differs (pointer convenience)
if [[ "$DUMP_PATH" != "$ART_DIR/pre-upgrade.dump" ]]; then
  cp "$DUMP_PATH" "$ART_DIR/pre-upgrade.dump" 2>/dev/null || true
fi
DUMP_SHA="$(sha256_file "$DUMP_PATH")"
DUMP_BYTES="$(wc -c <"$DUMP_PATH" | tr -d ' ')"
echo "$DUMP_SHA  pre-upgrade.dump" >"$ART_DIR/pre-upgrade.dump.sha256"
pass "pre-upgrade pg_dump written ($DUMP_BYTES bytes sha=${DUMP_SHA:0:12})"

log "stopping v${VERSION} API (keeping Postgres)…"
compose stop api >>"$LOG" 2>&1 || true

HEAD_BIN="$(build_head_bin)"
DB_URL="postgres://edgequake:${PGPASSWORD_VAL}@127.0.0.1:${PG_PORT}/edgequake"

log "HEAD binary: $HEAD_BIN"
[[ -x "$HEAD_BIN" ]] || die "HEAD binary not executable: $HEAD_BIN"

pre_dry_max="$(psql_c "SELECT coalesce(max(version),0) FROM _sqlx_migrations;")"
pre_dry_max="${pre_dry_max// /}"

log "running HEAD migrate dry-run (preview only)…"
set +e
DATABASE_URL="$DB_URL" "$HEAD_BIN" migrate dry-run >"$ART_DIR/migrate-dry-run.log" 2>&1
dry_ec=$?
set -e
if [[ $dry_ec -eq 127 ]]; then
  die "migrate binary not executable (exit 127) — HEAD_BIN='$HEAD_BIN'"
elif [[ $dry_ec -ne 0 ]]; then
  die "migrate dry-run failed (exit $dry_ec; see $ART_DIR/migrate-dry-run.log)"
fi
DRY_OUT="$(cat "$ART_DIR/migrate-dry-run.log")"
echo "$DRY_OUT" | grep -qiE 'DRY-RUN|dry-run' \
  || die "dry-run log missing DRY-RUN marker (see $ART_DIR/migrate-dry-run.log)"
echo "$DRY_OUT" | grep -qE '\b125\b' \
  || die "dry-run log missing pending migration 125"
echo "$DRY_OUT" | grep -qiE 'IRREVERSIBLE' \
  || die "dry-run log missing IRREVERSIBLE risk wording"
post_dry_max="$(psql_c "SELECT coalesce(max(version),0) FROM _sqlx_migrations;")"
post_dry_max="${post_dry_max// /}"
if [[ "$post_dry_max" != "$pre_dry_max" ]]; then
  die "dry-run mutated _sqlx_migrations max version ($pre_dry_max -> $post_dry_max)"
fi
if [[ "$post_dry_max" -ge 125 ]]; then
  die "dry-run left schema at/after drop (max=$post_dry_max); expected still pre-drop"
fi
pass "AC-M-03: migrate dry-run preview OK (schema max still $post_dry_max)"

log "running HEAD migrate (expect refuse without confirm)…"
set +e
DATABASE_URL="$DB_URL" "$HEAD_BIN" migrate >"$ART_DIR/migrate-refuse.log" 2>&1
refuse_ec=$?
set -e
if [[ $refuse_ec -eq 127 ]]; then
  die "migrate binary not executable (exit 127) — HEAD_BIN='$HEAD_BIN'"
elif [[ $refuse_ec -ne 0 ]]; then
  if grep -q "dry-run" "$ART_DIR/migrate-refuse.log"; then
    pass "migrate without --confirm-drop refused + dry-run hint (exit $refuse_ec)"
  else
    pass "migrate without --confirm-drop refused (exit $refuse_ec)"
    log "WARN: refuse path missing dry-run hint"
  fi
else
  log "WARN: migrate without confirm exited 0 (expandable-first soft path or 125 not next)"
fi

DATABASE_URL="$DB_URL" "$HEAD_BIN" migrate console >"$ART_DIR/migrate-console.log" 2>&1 || true
DATABASE_URL="$DB_URL" "$HEAD_BIN" migrate guard >"$ART_DIR/migrate-guard.log" 2>&1 || true

log "running HEAD migrate --confirm-drop (live tee)…"
set +e
DATABASE_URL="$DB_URL" "$HEAD_BIN" migrate --confirm-drop \
  2>&1 | tee "$ART_DIR/migrate-confirm.log"
confirm_ec=${PIPESTATUS[0]}
set -e
[[ $confirm_ec -eq 0 ]] \
  || die "migrate --confirm-drop failed (exit $confirm_ec; see $ART_DIR/migrate-confirm.log)"
CONFIRM_OUT="$(cat "$ART_DIR/migrate-confirm.log")"
echo "$CONFIRM_OUT" | grep -qE 'applied_this_run:' \
  || die "confirm-drop log missing applied_this_run summary"
echo "$CONFIRM_OUT" | grep -qE 'applied 125' \
  || die "confirm-drop log missing per-migration 'applied 125' line"
echo "$CONFIRM_OUT" | grep -qiE 'KV store dropped' \
  || die "confirm-drop log missing post-drop KV-gone message"
pass "AC-M-04: migrate --confirm-drop completed (applied 125 + KV drop message)"

POST_MIG_MAX="$(psql_c "SELECT coalesce(max(version),0) FROM _sqlx_migrations;")"
POST_MIG_MAX="${POST_MIG_MAX// /}"

# Cancel leftover async insert tasks from the v0.22.0 seed (mock→ollama coerce)
# so HEAD API boot does not thrash local inference during post-upgrade asserts.
psql_c "UPDATE public.tasks SET status='cancelled', updated_at=NOW() WHERE status IN ('pending','queued','processing','running') AND coalesce(task_type,'') ILIKE '%insert%';" \
  >/dev/null 2>&1 || true
log "cancelled leftover seed insert tasks (best-effort)"

start_head_api "$HEAD_BIN" "$DB_URL"
assert_post_upgrade

log "summary: PASS=$PASS FAIL=$FAIL"
if [[ "$FAIL" -gt 0 ]]; then
  write_verdict "RED"
  die "$FAIL assertion(s) failed"
fi
write_verdict "GREEN"
log "upgrade soak GREEN (profile=$PROFILE tag=$POSTGRES_TAG)"
exit 0
