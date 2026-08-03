# 11 — V22 Docker Repro (SPEC-104)

## Goal

Reproduce Steven’s error **classes** locally with `ghcr.io/raphaelmansuy/edgequake:0.22.0` + synthetic SQL (no prod dump).

## Prerequisites

Images (already on many dev machines):

```bash
docker pull ghcr.io/raphaelmansuy/edgequake:0.22.0
docker pull ghcr.io/raphaelmansuy/edgequake-postgres:0.22.0
```

## Isolated compose

Use the isolated compose (avoids fixed `container_name` clashes):

```bash
cd /path/to/edgequake
COMPOSE_PROJECT_NAME=eq104v22 EDGEQUAKE_VERSION=0.22.0 \
  SPEC104_API_PORT=18080 SPEC104_PG_PORT=15432 \
  docker compose -f specs/104-fix-datalayer/fixtures/docker-compose.v22-repro.yml up -d
curl -sf http://localhost:18080/health | tee specs/104-fix-datalayer/measurements/v22-health.json
```

Requires host Ollama (or change provider env) and `EDGEQUAKE_ALLOW_BOOT_MIGRATE=1` (already in the compose file).

## Seed fixtures

```bash
# Adjust container name from `docker compose ps`
docker compose -f docker-compose.quickstart.yml exec -T postgres \
  psql -U edgequake -d edgequake \
  < specs/104-fix-datalayer/fixtures/seed_v22_issue_classes.sql
```

Fixture forces:

| Issue | Seed action |
|-------|-------------|
| #1 | Create `eq_<uuid>_kv` without matching `workspaces` row → INV-D2 probes `id` |
| #2 | Ensure AGE graph is `eq_eq_default_graph` only (no `edgequake` graph) |
| #3 | Insert `documents` status=indexed with no chunk KV keys |
| #4 | (API) parallel `POST /api/v1/tenants` same name |
| #5 | Optional: large Node set without waiting for hourly — run count SQL manually |

## Trigger inspector

- Restart API (startup inspect) or wait ≤1h for hourly monitor.
- Or hit admin storage inspect if auth allows in quickstart (`EDGEQUAKE_DEV_MODE=true`).

## Expected log / SQL matches

```text
#1  undefined_column / 42703 / workspaces WHERE id
#2  undefined_table  / 42P01 / edgequake."Node"
#3  INV-03 CRITICAL / indexed documents have no KV chunks
#4  23505 / tenants_slug_key
#5  57014 / DATA-AGE-GRAPH-NODE-COUNTS-BY-SOURCE-PREFIXES (load-dependent)
```

## V23 residual check

Against HEAD binary + migrated DB:

| Issue | Expect pre-104 fix | Expect post-104 fix |
|-------|--------------------|---------------------|
| #1 | still 42703 | gone |
| #2 | still 42P01 | gone |
| #3 | silent if KV dropped | INV-03 on `chunks` |
| #4 | still 23505 under race | idempotent |
| #5 | soft timeout possible | GIN finding if missing |

Record outputs under `measurements/`. Full deploy/compatibility matrix: [13-fix-assessment.md](13-fix-assessment.md) §2.

## Tear down

```bash
COMPOSE_PROJECT_NAME=eq104v22 docker compose -f specs/104-fix-datalayer/fixtures/docker-compose.v22-repro.yml down -v
```
