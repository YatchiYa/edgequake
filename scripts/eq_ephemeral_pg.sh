#!/usr/bin/env bash
# SPEC-065 — shared ephemeral Postgres for capacity/battle harnesses (DRY).
#
# Usage (from a runner script):
#   source scripts/eq_ephemeral_pg.sh
#   eq_ephemeral_pg_start "pg18" "edgequake-cap"
#   # sets DATABASE_URL, EQ_POSTGRES_PROFILE, EQ_POSTGRES_MAJOR, container cleanup trap
#   eq_ephemeral_pg_migrate
#   ... run cargo test ...
#   # trap removes container on EXIT
#
# Env:
#   POSTGRES_PASSWORD (default edgequake_secret)
#   EQ_EPHEMERAL_PG_SHM (default 2g)
#   EQ_EPHEMERAL_PG_SHARED_BUFFERS (default 512MB)
#   EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM (default 1GB)
set -euo pipefail

eq_ephemeral_pg_start() {
  local profile="${1:?profile}"
  local name_prefix="${2:-edgequake-eph}"
  local root script_dir
  # bash: BASH_SOURCE; zsh when sourced: use $0 fallback only for bash -c path
  if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  else
    script_dir="$(cd "$(dirname "$0")" && pwd)"
  fi
  # When sourced from repo runners, prefer ROOT if already set
  if [[ -n "${ROOT:-}" && -d "$ROOT/edgequake/docker" ]]; then
    root="$ROOT"
  else
    root="$(cd "$script_dir/.." && pwd)"
  fi
  local docker_dir="$root/edgequake/docker"
  local pins="$docker_dir/extension-pins.sh"
  local pgpassword="${POSTGRES_PASSWORD:-edgequake_secret}"
  local shm="${EQ_EPHEMERAL_PG_SHM:-2g}"
  local shared_buffers="${EQ_EPHEMERAL_PG_SHARED_BUFFERS:-512MB}"
  local maint_mem="${EQ_EPHEMERAL_PG_MAINTENANCE_WORK_MEM:-1GB}"

  # shellcheck source=/dev/null
  EQ_POSTGRES_PROFILE="$profile" source "$pins"
  local image="$EQ_POSTGRES_IMAGE_TAG"
  local container="${name_prefix}-${profile}-$$"

  if ! docker image inspect "$image" >/dev/null 2>&1; then
    if [[ "$profile" == "pg18" ]] && docker image inspect "edgequake-postgres:pg18" >/dev/null 2>&1; then
      docker tag "edgequake-postgres:pg18" "$image"
    elif [[ "$profile" == "pg18-vectorscale" ]] && docker image inspect "edgequake-postgres:pg18-vectorscale" >/dev/null 2>&1; then
      : # already tagged via pins
    else
      echo "ERROR: image $image missing; build with make postgres-image-build(-pg17|-pg18|-pg18-vectorscale)"
      return 1
    fi
  fi

  docker rm -f "$container" >/dev/null 2>&1 || true
  docker run -d --name "$container" \
    --shm-size="$shm" \
    -e POSTGRES_USER=edgequake \
    -e POSTGRES_PASSWORD="$pgpassword" \
    -e POSTGRES_DB=edgequake \
    -p 127.0.0.1::5432 \
    "$image" \
    -c shared_buffers="$shared_buffers" \
    -c maintenance_work_mem="$maint_mem" \
    -c work_mem=64MB \
    -c max_parallel_maintenance_workers=2 >/dev/null

  # shellcheck disable=SC2064
  trap "docker rm -f '$container' >/dev/null 2>&1 || true" EXIT

  local i
  for i in $(seq 1 120); do
    if docker exec -e PGPASSWORD="$pgpassword" "$container" \
      psql -U edgequake -d edgequake -c 'SELECT 1' >/dev/null 2>&1; then
      sleep 2
      if docker exec -e PGPASSWORD="$pgpassword" "$container" \
        psql -U edgequake -d edgequake -c 'SELECT 1' >/dev/null 2>&1; then
        break
      fi
    fi
    if [[ "$i" -eq 120 ]]; then
      echo "Postgres did not become ready"
      docker logs "$container" 2>&1 | tail -40 || true
      return 1
    fi
    sleep 1
  done

  # SPEC-070: enable vectorscale when the opt-in profile image is used.
  local vs_sql=""
  if [[ "$profile" == "pg18-vectorscale" ]] || [[ -n "${EQ_PGVECTORSCALE_MIN:-}" ]]; then
    vs_sql="CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE;"
  fi
  docker exec -i -e PGPASSWORD="$pgpassword" "$container" \
    psql -U edgequake -d edgequake -v ON_ERROR_STOP=1 <<SQL
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS age;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
${vs_sql}
LOAD 'age';
ALTER EXTENSION vector UPDATE;
ALTER EXTENSION age UPDATE;
SQL

  local host_port
  host_port="$(docker port "$container" 5432/tcp | head -1 | awk -F: '{print $NF}')"
  export DATABASE_URL="postgres://edgequake:${pgpassword}@127.0.0.1:${host_port}/edgequake?options=-c%20search_path%3Dpublic"
  echo "$DATABASE_URL" >/tmp/edgequake-db-url
  export EQ_POSTGRES_PROFILE="$profile"
  export EQ_POSTGRES_MAJOR
  export EQ_EPHEMERAL_PG_CONTAINER="$container"
  export EQ_EPHEMERAL_PG_ROOT="$root"
  export EDGEQUAKE_DIR="$root/edgequake"
  export PGPASSWORD="$pgpassword"
}

eq_ephemeral_pg_migrate() {
  local root="${EQ_EPHEMERAL_PG_ROOT:?run eq_ephemeral_pg_start first}"
  sqlx migrate run --source "$root/edgequake/migrations" --database-url "$DATABASE_URL"
  export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
  export EDGEQUAKE_NATIVE_GRAPH_WRITES=1
}
