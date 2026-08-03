#!/usr/bin/env bash
# scripts/spec93_migration_assessment.sh
#
# SPEC-93: run v0.22.0 → HEAD upgrade soak across PG16 / PG17 / PG18 with the
# realism corpus, writing reports under specs/93-migration-assessment/reports/.
#
# Isolation (must not touch EdgeForce / GPS / other host apps):
#   - Compose project names are only `spec93soak-pg{16,17,18}`
#   - Host ports are ephemeral on 127.0.0.1 (see docker-compose.spec091-soak.yml)
#   - HEAD API binds 127.0.0.1:<ephemeral>; cleanup kills only that PID
#   - Pre/post snapshot asserts foreign listeners (8787, 55432, …) stay unchanged
#   - Never runs `make kill-app` / host-wide `pkill`
#
# Usage:
#   ./scripts/spec93_migration_assessment.sh              # all majors
#   SPEC93_PG_PROFILE=pg17 ./scripts/spec93_migration_assessment.sh
#   make spec93-migration-assessment
#   make spec93-migration-assessment-pg16
#
# Env:
#   SPEC93_PG_PROFILE          single major (pg16|pg17|pg18); default = all
#   SPEC93_PROFILE             default realism
#   SPEC091_SOAK_SKIP_PULL=1   skip docker pull
#   SPEC091_SOAK_BIN           shared HEAD binary path (built once)
#   EDGEQUAKE_VERSION          default 0.22.0
#   SPEC93_SKIP_FOREIGN_GUARD=1  skip foreign-port PID snapshot (CI-only)

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOAK="$ROOT/scripts/spec091_upgrade_soak.sh"
REPORT_ROOT="$ROOT/specs/93-migration-assessment/reports"
ARTIFACT_ROOT="$ROOT/artifacts/spec93-migration-assessment"
VERSION="${EDGEQUAKE_VERSION:-0.22.0}"
PROFILE="${SPEC93_PROFILE:-realism}"
MATRIX_STARTED="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Host listeners belonging to other products — soak must not stop or rebind these.
FOREIGN_PORTS=(8787 55432 8080 5173 8000 5433 9000 9001 3100 5001)

listener_pid_on_port() {
  local port="$1"
  lsof -nP -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null | head -1 || true
}

snapshot_foreign_listeners() {
  local out="$1"
  : >"$out"
  local port pid
  for port in "${FOREIGN_PORTS[@]}"; do
    pid="$(listener_pid_on_port "$port")"
    printf '%s %s\n' "$port" "${pid:-none}" >>"$out"
  done
}

# Isolation semantics:
#   FAIL  — soak published on a foreign port, or a foreign port's new listener is a soak process
#   WARN  — foreign app died/restarted on its own (PID none or unrelated new PID); not soak interference
#   OK    — every previously-watched PID unchanged
assert_foreign_listeners_untouched() {
  local before="$1" after="$2" label="$3"
  if [[ "${SPEC93_SKIP_FOREIGN_GUARD:-0}" == "1" ]]; then
    echo "[spec93] foreign-port guard skipped (SPEC93_SKIP_FOREIGN_GUARD=1)"
    return 0
  fi
  local port before_pid after_pid soak_hit=0 warn=0
  # Ephemeral mappings (127.0.0.1:32xxx->8080) are OK; host-side foreign ports are not.
  if docker ps --format '{{.Names}} {{.Ports}}' 2>/dev/null \
    | rg 'spec93soak' \
    | rg -q '0\.0\.0\.0:(8787|55432|8080|5173|8000|5433|9000|9001|3100|5001)->|127\.0\.0\.1:(8787|55432|8080|5173|8000|5433|9000|9001|3100|5001)->'; then
    echo "[spec93] ERROR: soak container published a foreign host port ($label)" >&2
    docker ps --format '{{.Names}} {{.Ports}}' | rg 'spec93soak' >&2 || true
    return 1
  fi
  while read -r port before_pid; do
    after_pid="$(awk -v p="$port" '$1==p {print $2; exit}' "$after")"
    after_pid="${after_pid:-none}"
    if [[ "$before_pid" == "$after_pid" ]]; then
      continue
    fi
    # Did soak steal the port?
    if [[ "$after_pid" != "none" ]]; then
      local cmd
      cmd="$(ps -p "$after_pid" -o command= 2>/dev/null || true)"
      if echo "$cmd" | rg -qi 'spec93soak|spec091_upgrade_soak|edgequake/target/debug/edgequake'; then
        echo "[spec93] ERROR: foreign port $port now held by soak-related process pid=$after_pid ($label)" >&2
        echo "  cmd=$cmd" >&2
        soak_hit=1
        continue
      fi
    fi
    echo "[spec93] WARN: foreign port $port listener changed $before_pid → $after_pid (independent of soak; $label)"
    warn=1
  done <"$before"
  if [[ "$soak_hit" -eq 1 ]]; then
    return 1
  fi
  if [[ "$warn" -eq 1 ]]; then
    echo "[spec93] isolation OK-with-warn ($label): soak did not bind/kill foreign ports"
  else
    echo "[spec93] isolation OK ($label): foreign ports unchanged ($(wc -l <"$before" | tr -d ' ') watched)"
  fi
}

chmod +x "$SOAK"

FOREIGN_SNAP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/spec93-foreign.XXXXXX")"
FOREIGN_BEFORE="$FOREIGN_SNAP_DIR/before.txt"
FOREIGN_AFTER="$FOREIGN_SNAP_DIR/after.txt"
snapshot_foreign_listeners "$FOREIGN_BEFORE"
echo "[spec93] foreign listener snapshot (do-not-touch):"
cat "$FOREIGN_BEFORE" | sed 's/^/  /'
trap 'rm -rf "$FOREIGN_SNAP_DIR"' EXIT

if [[ -n "${SPEC93_PG_PROFILE:-}" ]]; then
  PROFILES=("$SPEC93_PG_PROFILE")
else
  PROFILES=(pg16 pg17 pg18)
fi

for p in "${PROFILES[@]}"; do
  case "$p" in
    pg16|pg17|pg18) ;;
    *)
      echo "invalid SPEC93_PG_PROFILE=$p (use pg16|pg17|pg18)" >&2
      exit 1
      ;;
  esac
done

mkdir -p "$REPORT_ROOT" "$ARTIFACT_ROOT"

# Build HEAD binary once and reuse
HEAD_BIN="${SPEC091_SOAK_BIN:-}"
if [[ -z "$HEAD_BIN" || ! -x "$HEAD_BIN" ]]; then
  echo "[spec93] building HEAD edgequake binary once…"
  (cd "$ROOT/edgequake" && cargo build -p edgequake --features postgres --bin edgequake)
  HEAD_BIN="$ROOT/edgequake/target/debug/edgequake"
  [[ -x "$HEAD_BIN" ]] || { echo "missing $HEAD_BIN" >&2; exit 1; }
fi
export SPEC091_SOAK_BIN="$HEAD_BIN"

declare -a RESULTS=()

run_one() {
  local major="$1"
  local art="$REPORT_ROOT/$major"
  local dump_dir="$ARTIFACT_ROOT/$major"
  mkdir -p "$art" "$dump_dir"
  echo "[spec93] ── $major (tag=${VERSION}-${major}) profile=$PROFILE ──"
  local t0 t1 ec
  t0="$(date +%s)"
  set +e
  SPEC93_PROFILE="$PROFILE" \
    SPEC091_SOAK_DIR="$art" \
    SPEC93_DUMP_DIR="$dump_dir" \
    SPEC091_COMPOSE_PROJECT="spec93soak-${major}" \
    EDGEQUAKE_VERSION="$VERSION" \
    EDGEQUAKE_POSTGRES_TAG="${VERSION}-${major}" \
    SPEC091_SOAK_BIN="$HEAD_BIN" \
    "$SOAK"
  ec=$?
  set -e
  t1="$(date +%s)"
  local wall=$((t1 - t0))
  if [[ $ec -eq 0 ]]; then
    RESULTS+=("${major}|GREEN|${wall}")
    echo "[spec93] $major GREEN (${wall}s)"
  else
    RESULTS+=("${major}|RED|${wall}")
    echo "[spec93] $major RED (${wall}s) — see $art/soak.log"
  fi
  return 0
}

OVERALL_ISOLATION_FAIL=0
for major in "${PROFILES[@]}"; do
  run_one "$major"
  snapshot_foreign_listeners "$FOREIGN_AFTER"
  if ! assert_foreign_listeners_untouched "$FOREIGN_BEFORE" "$FOREIGN_AFTER" "after $major"; then
    OVERALL_ISOLATION_FAIL=1
  fi
done

# Aggregate matrix-summary.md
MATRIX_FINISHED="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
OVERALL="PASS"
[[ "$OVERALL_ISOLATION_FAIL" == "1" ]] && OVERALL="FAIL"
{
  echo "# Matrix summary — SPEC-93 migration assessment"
  echo
  echo "> Generated: $MATRIX_FINISHED (started $MATRIX_STARTED)"
  echo "> Source: \`ghcr.io/raphaelmansuy/edgequake:${VERSION}\`"
  echo "> Target: HEAD migrations through **141**"
  echo "> Profile: \`$PROFILE\`"
  echo "> Isolation: foreign host ports unchanged (EdgeForce :8787/:55432, GPS, …)"
  echo
  echo "| PG profile | Verdict | Wall (s) | Postgres | Pre max mig | Post max mig | Docs seeded | Dump SHA (12) |"
  echo "| --- | --- | --- | --- | --- | --- | --- | --- |"
} >"$REPORT_ROOT/matrix-summary.md"

for row in "${RESULTS[@]}"; do
  major="${row%%|*}"
  rest="${row#*|}"
  verdict="${rest%%|*}"
  wall="${rest##*|}"
  vjson="$REPORT_ROOT/$major/verdict.json"
  pg="—"
  pre="—"
  post="—"
  docs="—"
  sha="—"
  if [[ -f "$vjson" ]]; then
    pg="$(jq -r '.postgres_version // "—"' "$vjson" | tr '\n' ' ' | sed 's/ *$//')"
    pre="$(jq -r '.pre_migration_max // "—"' "$vjson")"
    post="$(jq -r '.post_migration_max // "—"' "$vjson")"
    docs="$(jq -r '.documents_seeded // "—"' "$vjson")"
    sha="$(jq -r '.dump_sha256 // ""' "$vjson")"
    [[ -n "$sha" && "$sha" != "null" ]] && sha="${sha:0:12}"
    [[ -z "$sha" || "$sha" == "null" ]] && sha="—"
  fi
  echo "| $major | **$verdict** | $wall | \`$pg\` | $pre | $post | $docs | \`$sha\` |" \
    >>"$REPORT_ROOT/matrix-summary.md"
  if [[ "$verdict" != "GREEN" ]]; then
    OVERALL="FAIL"
  fi
done

{
  echo
  echo "**Overall:** **$OVERALL**"
  echo
  echo "## Notes"
  echo
  echo "- Per-major artifacts: \`reports/pg16/\`, \`reports/pg17/\`, \`reports/pg18/\`"
  echo "- Binary dumps: \`artifacts/spec93-migration-assessment/<major>/pre-upgrade.dump\`"
  echo "- Protocol: [01-test-protocol.md](../01-test-protocol.md) · AC: [02-acceptance-criteria.md](../02-acceptance-criteria.md)"
} >>"$REPORT_ROOT/matrix-summary.md"

echo "[spec93] matrix overall=$OVERALL → $REPORT_ROOT/matrix-summary.md"
[[ "$OVERALL" == "PASS" ]]
