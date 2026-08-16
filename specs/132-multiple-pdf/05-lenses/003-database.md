# Lens 003 — Database Expert

## Stake

Admit durability lives in Postgres: PDF BYTEA + `tasks` row. Wake channel is **not** durability.

## Invariants

```ascii
  task row COMMITTED  ──►  Plane A durable
  wake send failure   ──►  hydrate / claim_next still finds Pending
  BYTEA write slow    ──►  client XHR timeout (honest fail), not silent orphan
```

## Concurrent multi-PDF

- N× single-file posts each write their own BYTEA — body limit is **per request**, avoiding `/pdf/batch` sum>50 MiB.
- Watch for pool saturation under concurrent admits (SPEC-090 / connection pool) — classify as infra if p95 admit >> timeout.
- No CHECK / migration required for SPEC-132 unless dual-write status bugs appear (orthogonal SPEC-129).

## Observability

- Queue depth / pending count (existing `task_queue_pressure`)
- Admit latency logs around BYTEA + enqueue
- Never treat channel depth alone as backlog truth (DB pending is SSOT)

## Cross-refs

- Laws: [../01-first-principles.md](../01-first-principles.md)
- SPEC-091: [../../091-simplify-data-layer/](../../091-simplify-data-layer/)
