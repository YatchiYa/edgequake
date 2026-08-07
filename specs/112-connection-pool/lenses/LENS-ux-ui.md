# LENS — UX / UI (SPEC-112)

## User (operator) job

When the fleet is connection-stressed, the operator must answer in under a minute:

1. Is EdgeQuake healthy but **holding** the DB?
2. Which **role** pool is saturated?
3. What **action** unblocks co-tenants without a blind restart?

## Current surfaces

| Surface | Today | Gap |
|---------|-------|-----|
| `/health` | Component liveness | Little connection-budget storytelling |
| `/ready` | Can block on pool util / store contention | May not show configured max per role |
| `/metrics` | size + idle per role | Configured max not always first-class |
| Docs | Pool env vars scattered | Need SPEC-112 formula front-and-center |

## UX principles for Wave D

- **One sentence operator_action** when budget or util fails readiness — e.g. “Lower `EDGEQUAKE_DB_POOL_SIZE_*` or scale replicas; shared PG near max_connections.”
- **Show numbers, not vibes:** `ingest 12/12 busy` beats “database busy.”
- **Never** recommend “stop EdgeQuake” as primary copy.
- Failures should distinguish **app pool full** vs **PostgreSQL refused connection** when detectable.

## Copy sketches

```text
Ready blocker:
  store_contention_critical(pool_util=0.97)
Operator action:
  Ingest pool saturated (12/12). Reduce ingest concurrency or raise
  EDGEQUAKE_DB_POOL_SIZE_INGEST only if Σ pools still fit shared max_connections.
```

```text
Boot warning:
  pool_budget: need=68 (2 instances × 34) exceeds pg max_connections=100 − reserve=13
```

## Non-goals

- New marketing dashboard.
- Alerting product inside the WebUI beyond existing health patterns.
