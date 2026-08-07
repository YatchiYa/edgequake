# BRUTAL HONESTY — SPEC-112 snapshot

## What we can claim

| Claim | Evidence |
|-------|----------|
| EdgeQuake held idle backends on shared PG | 10× `usename=edgequake`, `state=idle`, `ClientRead` |
| Last activity is pool hygiene, not a long query | `SET search_path TO public` on all 10 |
| Attribution is broken today | `application_name` empty on all EdgeQuake rows |
| Stopping EdgeQuake freeing slots is plausible | Idle held slots = capacity; partner observed QL recovery after stop |
| Code can hold up to ~34 slots per process by default | `PgPoolBundle` defaults 16+12+4+2 |
| Graceful shutdown does not close pools | `server.rs` drain only; no `pool.close()` |

## What we must not claim

| Overclaim | Why false / unproven |
|-----------|----------------------|
| “EdgeQuake opened 400 connections” | CSV has 10 EdgeQuake client backends |
| “This CSV is the peak incident window” | Total client backends = 23 ≪ reported `max_connections=400` |
| “sqlx leaked connections outside the pool” | Idle pooled connections are expected; leak ≠ idle |
| “Role X was the culprit” | No `application_name`; single `client_addr` only |
| “Raising max_connections to 400 is the fix” | Band-aid; increases PG process overhead ([LAW-112-6](../01-first-principles.md)) |

## Working theory (ranked)

1. **Most likely:** Shared-DB oversubscription — EdgeQuake’s configured pool ceiling (× replicas / deploy overlap) + QL + tools approaches the server limit; idle EdgeQuake slots tip co-tenants over.
2. **Contributing:** Missing identity and shutdown close make diagnosis and cleanup harder.
3. **Unproven from CSV:** Classic “connection leak” (`idle in transaction` growth) — **zero** `idle in transaction` rows in this file.

## Bar for “fixed”

- [x] Partners can attribute backends by `application_name` (`edgequake:<role>`)
- [x] Documented budget formula holds for PPD replica counts (`EDGEQUAKE_DB_POOL_*`)
- [x] Graceful stop closes pools (verified by e2e `e2e_spec112_close_releases_backends`)
- [x] Stopping EdgeQuake is no longer the recommended recovery step (ops runbook)

Gates: [`e2e112-gates.txt`](e2e112-gates.txt) (7 passed).
