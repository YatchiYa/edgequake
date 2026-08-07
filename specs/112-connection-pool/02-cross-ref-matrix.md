# 02 — Cross-ref matrix

## Spec ↔ law ↔ code

| LAW | Spec anchor | Code symbol / file | Env / API | Tests to write (Wave E) |
|-----|-------------|--------------------|-----------|-------------------------|
| 112-1 | This pack | `PgPoolBundle`, `with_session_hygiene`, `AppState::from_postgres` | — | Contracts that cite paths |
| 112-2 | SPEC-090 F-090-28 | `pool_bundle.rs` `connect_role`; idle backends in CSV | `EDGEQUAKE_DB_POOL_SIZE_*` | Idle count ≤ configured max |
| 112-3 | SPEC-090 / ops | `PgPoolBundle::total_max_connections` | `EDGEQUAKE_DB_POOL_INSTANCE_COUNT` (proposed) | Budget formula unit test |
| 112-4 | — | `with_session_hygiene` / role connect | — | `SHOW application_name` e2e |
| 112-5 | [SPEC-083 X-31](../083-improvements/defects/X-31.md) | `server.rs` `run` + proposed close | drain budget | Contract: close after drain |
| 112-6 | Ops runbook | — | Partner `max_connections` | Docs only |
| 112-7 | SPEC-090 LAW-P4, [SPEC-089](../089-health-check/) | `after_connect` / `after_release` | proposed idle-in-xact GUC | Hygiene source contract |
| 112-8 | Health / store contention | `handlers/metrics.rs`, `handlers/health.rs`, `store_contention` | Prometheus gauges | Metrics field / ready payload |

## File map (current HEAD)

| Path | Role today | SPEC-112 change (planned) |
|------|------------|---------------------------|
| [`pool_bundle.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/pool_bundle.rs) | Four pools; defaults 16/12/4/2; `min_connections=1`; no idle/max lifetime | Explicit timeouts; role `application_name`; optional budget helper |
| [`connection.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/connection.rs) | `search_path` + `RESET ALL`; single-pool path | Role-aware hygiene SSOT |
| [`config.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/config.rs) | Single-pool default max=32; role env resolve | Document shared-DB recommendations |
| [`state/postgres.rs`](../../edgequake/crates/edgequake-api/src/state/postgres.rs) | Boots `PgPoolBundle::connect` | Wire budget warn/fail; keep admin for migrate |
| [`server.rs`](../../edgequake/crates/edgequake-api/src/server.rs) | Graceful HTTP drain | Call pool close after drain |
| [`handlers/metrics.rs`](../../edgequake/crates/edgequake-api/src/handlers/metrics.rs) | Per-role size/idle gauges | Also expose configured max |
| [`handlers/health.rs`](../../edgequake/crates/edgequake-api/src/handlers/health.rs) | Pool util readiness | Surface budget / per-role configured max |
| [`e2e_spec090_multi_pool.rs`](../../edgequake/crates/edgequake-storage/tests/e2e_spec090_multi_pool.rs) | Isolation + size env | Extend for identity + close |

## Env vars (existing + proposed)

| Variable | Status | Meaning |
|----------|--------|---------|
| `DATABASE_URL` | Existing | Primary URL for ingest/queue/admin |
| `DATABASE_READ_URL` | Existing | Optional query pool URL |
| `EDGEQUAKE_DB_POOL_SIZE_QUERY` | Existing | Clamp 1–128, default 16 |
| `EDGEQUAKE_DB_POOL_SIZE_INGEST` | Existing | Default 12 |
| `EDGEQUAKE_DB_POOL_SIZE_QUEUE` | Existing | Default 4 |
| `EDGEQUAKE_DB_POOL_SIZE_ADMIN` | Existing | Default 2 |
| `EDGEQUAKE_DB_POOL_ROLE` | Existing | Legacy single-pool role label |
| `EDGEQUAKE_DB_POOL_INSTANCE_COUNT` | **Proposed** | Replica count for startup budget check |
| `EDGEQUAKE_DB_POOL_BUDGET_MODE` | **Proposed** | `warn` (default) \| `fail` |

## Related findings (SPEC-090)

| Finding | Status | Note for 112 |
|---------|--------|--------------|
| F-090-28 multi-pool | FIXED | Isolation exists; shared-DB **budget** still open |
| F-090-07 session hygiene | FIXED | Missing `application_name` + idle-in-xact |
| F-090-31 read URL | FIXED | Query pool may hit different host — still needs identity |

## External references

- [sqlx `PoolOptions`](https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html) — `idle_timeout`, `max_lifetime`, `after_connect`, `after_release`
- [PostgreSQL `max_connections`](https://www.postgresql.org/docs/current/runtime-config-connection.html)
- [Connection exhaustion diagnosis](https://www.netdata.cloud/guides/postgres/postgres-connection-exhaustion/)
