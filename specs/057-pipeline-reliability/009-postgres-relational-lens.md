# 009 — Postgres Relational Lens

**Spec:** SPEC-057  
**Key question:** How should relational Postgres be the ledger for task delivery and recovery?

---

## Scope

`tasks` table lifecycle, heartbeats, orphan recovery, admission, indexes, claim patterns. Graph/vector internals → [010](./010-age-pgvector-lens.md).

---

## Durable vs ephemeral (ASCII)

```text
  ┌─────────────────────────────────────────────────────────┐
  │  Postgres tasks row (DURABLE SSOT — SPEC-057 P1)        │
  │  + lease_owner / lease_token / lease_expires_at (M088)  │
  └───────────────────────────┬─────────────────────────────┘
                              │  claim_next: FOR UPDATE SKIP LOCKED
                              ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Channel / NOTIFY (WAKE ONLY)                            │
  │  Payload not authoritative — workers always claim       │
  └───────────────────────────┬─────────────────────────────┘
                              │  missed wake ⇒ ~2s poll still claims
                              ▼
  Boot: Pending left claimable; Processing → Failed or reclaim
  (EDGEQUAKE_STARTUP_AUTO_RESUME only affects Processing)
```

---

## Task lifecycle (relational)

| Status | Meaning | Restart behavior (default, P1) |
| ------ | ------- | ------------------------------ |
| Pending | Accepted, not running | **Survives** — claimable without AUTO_RESUME |
| Processing | Worker holds lease | Stale/expired → Failed (“Interrupted — use Reprocess”) |
| Indexed | Success terminal | Stable |
| Failed | Failure terminal | Stable; Reprocess creates new work |
| Cancelled | Cancel terminal | Stable; **never claimed** |

Heartbeats: worker `refresh_lease` (~60s) CAS on `lease_owner`+`lease_token`. Periodic reaper prefers `lease_expires_at`; falls back to 10m `updated_at` for pre-migration rows.

---

## Industry claim pattern (P1 — implemented)

```sql
-- Live shape: edgequake-tasks PostgresTaskStorage::claim_next (mig 088)
WITH candidate AS (
  SELECT track_id FROM tasks
  WHERE status = 'pending'
     OR (status = 'processing' AND (lease_expires_at IS NULL OR lease_expires_at < NOW()))
  ORDER BY created_at ASC
  FOR UPDATE SKIP LOCKED
  LIMIT 1
)
UPDATE tasks t SET status = 'processing', lease_owner = $1, ...
FROM candidate WHERE t.track_id = candidate.track_id
RETURNING *;
```

Postgres-native durable queue primitive. Channel demoted to wake (REQ-057-01, 10).

---

## Roadblocks (relational)

| Roadblock | Why it blocks | Mitigation |
| --------- | ------------- | ---------- |
| ~~Channel-only wake~~ | ~~Pending rows inert after crash~~ | **P1 done:** SKIP LOCKED claim + wake/poll |
| Auto-resume off | Cost policy OK; Processing needs Reprocess CTA | Interrupted copy + Reprocess (Pending auto-claims) |
| ~~Processing without lease expiry~~ | ~~Orphans stick~~ | **P1 done:** lease TTL + reaper |
| Cancel intent not a column | Set lost on restart | Status `Cancelled` durable — claim never picks it |
| Rate-limit stats stub | `rate_limited: false // TODO` in storage | Wire real limiter metrics |

---

## Admission & pressure

- `services/ingest_admission.rs` — fairness pressure before enqueue  
- `task_queue_pressure.rs` — warn/critical pending thresholds  
- Relational implication: prefer **reject/503 with Retry-After** over unbounded channel fill  

---

## Recommendations → REQ

| Change | REQ |
| ------ | --- |
| Claim loop over `tasks` + channel wake | REQ-057-01 |
| On every dequeue, re-read status (Cancelled wins) | REQ-057-02, 06 |
| Lease TTL for Processing + startup reaper | REQ-057-01, 05 |
| Keep auto-resume default off; improve Interrupted UX | REQ-057-05 |
| Indexes: `(status, created_at)`, `(status, updated_at)` for claim/reaper | REQ-057-10 |

**Out of scope:** Sharding tasks across databases; logical replication design.

Next: [010-age-pgvector-lens.md](./010-age-pgvector-lens.md)
