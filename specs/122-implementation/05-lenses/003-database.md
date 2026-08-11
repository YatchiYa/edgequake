# Lens 003 — Database Expert

## Stake

Raising ingest concurrency re-exposes SPEC-090 write amplification (stats tuple locks, long upsert TX, claim_next cost). Throughput “wins” that tank the pool are regressions.

## Invariants (DB-122)

| ID | Invariant |
|----|-----------|
| DB-122-1 | Tenant lane raises must watch `store_contention` on queue-metrics |
| DB-122-2 | `/ready` stays honest under critical contention |
| DB-122-3 | claim_next must not become O(N²) under bulk Pending depth |
| DB-122-4 | Workspace isolation on claim/lease preserved |
| DB-122-5 | PDF blob list paths must not TOAST-blow interactive reads (SPEC-090) |

## As-is

- Task rows + SKIP LOCKED claim (edgequake-tasks / storage)
- Vector/KV upserts under insert pipeline
- Fairness park releases DB claim before wait (SPEC-057)

## Guidance

1. Phase B load tests include SPEC-090 metrics, not only docs/min.
2. Prefer measuring LLM busy vs lock waits before blaming Postgres.
3. Pool size (`DATABASE_POOL_SIZE`) must track worker×tenant product with headroom for queries.

## Cross-refs

- SPEC-090: [../../090-performance/00-why.md](../../090-performance/00-why.md)
- Reliability lens: [007-reliability-engineer.md](007-reliability-engineer.md)
