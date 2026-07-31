#!/usr/bin/env bash
# scripts/spec93_migration_assessment.sh
#
# SPEC-93: run v0.22.0 → HEAD upgrade soak across PG16 / PG17 / PG18 with the
# realism corpus, writing reports under specs/93-migration-assessment/reports/.
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

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOAK="$ROOT/scripts/spec091_upgrade_soak.sh"
REPORT_ROOT="$ROOT/specs/93-migration-assessment/reports"
ARTIFACT_ROOT="$ROOT/artifacts/spec93-migration-assessment"
VERSION="${EDGEQUAKE_VERSION:-0.22.0}"
PROFILE="${SPEC93_PROFILE:-realism}"
MATRIX_STARTED="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

chmod +x "$SOAK"

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

for major in "${PROFILES[@]}"; do
  run_one "$major"
done

# Aggregate matrix-summary.md
MATRIX_FINISHED="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
OVERALL="PASS"
{
  echo "# Matrix summary — SPEC-93 migration assessment"
  echo
  echo "> Generated: $MATRIX_FINISHED (started $MATRIX_STARTED)"
  echo "> Source: \`ghcr.io/raphaelmansuy/edgequake:${VERSION}\`"
  echo "> Target: HEAD migrations through **137**"
  echo "> Profile: \`$PROFILE\`"
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
