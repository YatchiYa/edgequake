# SPEC-112 — Connection Pool Cross-Ref Pack

> **Trigger:** PPD shared-PostgreSQL incident — EdgeQuake held many **idle** backends; stopping EdgeQuake freed slots so QL could reconnect; server `max_connections` had been raised to **400**.  
> **Method:** First principles — **code is law** — prove behavior from `PgPoolBundle` / `with_session_hygiene` / shutdown + partner `pg_stat_activity` snapshot.  
> **Audience:** Engineering (fix train) + partners (ops runbook without waiting for a release).  
> **Ship vehicle:** **v0.24.3** (SPEC-112 Waves A–E on HEAD; pack + code).

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  INCIDENT — Shared PG slot starvation (PPD)                                  │
│    Symptom: co-tenants (QL) cannot connect while EdgeQuake is up             │
│    Observed: stop EdgeQuake → slots free → QL reconnects                     │
│    Band-aid seen: max_connections = 400 (not the product fix)                │
│                                                                              │
│  CODE FACTS (HEAD)                                                           │
│    PgPoolBundle: query16 + ingest12 + queue4 + admin2  = ≤34 / process       │
│    application_name unset → pg_stat_activity attribution blind               │
│    HTTP drain (SPEC-083) exists; pool.close() on shutdown MISSING            │
│    Bundle connect: no explicit idle_timeout / max_lifetime                   │
│                                                                              │
│  SNAPSHOT HONESTY                                                            │
│    CSV shows 10 idle edgequake backends — NOT peak saturation vs 400         │
│    Evidence class: idle-hold + identity gap + shared-budget math             │
│                                                                              │
│  FIX TRAIN (see 04-fix-plan) — Waves A–E after pack approval                 │
│    A identity + close  B budget  C session nets  D observability  E e2e      │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| Thread                                                | Severity | Present on HEAD? | Fix needed?                            |
| -------------------------------------------------------| ----------| ------------------| ----------------------------------------|
| Idle pools starve shared PG co-tenants                | P0 ops   | Mitigated        | Budget check + sizing docs + env       |
| Empty `application_name`                              | P0 ops   | **Fixed**        | `edgequake:<role>` in session baseline |
| No `pool.close()` after graceful drain                | P1       | **Fixed**        | `Server::close_db_pools` after drain   |
| Bundle lacks explicit idle/max lifetime               | P1       | **Fixed**        | idle 600s / max_lifetime 1800s         |
| No startup slot-budget check                          | P1       | **Fixed**        | `check_pool_budget` warn/fail          |
| No connect-time `idle_in_transaction_session_timeout` | P2       | **Fixed**        | 60s default in baseline                |
| Health/metrics pool util exists                       | —        | **Extended**     | per-role max + `db_pools` on /health   |

## Document map

```ascii
 00-why / 00-incident-data
   → 01-first-principles (LAW-112-*)
   → 02-cross-ref-matrix
   → 03-root-cause
   → 04-fix-plan (DRY / SOLID waves A–E)
   → 05-e2e-test-matrix
   → 06-edge-cases
   → 07-ops-runbook
   → measurements/ (CSV + BRUTAL-HONESTY)
   → lenses/ (PO, fullstack, DB, UX/UI, front, marketing)
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Truth source | `pool_bundle.rs`, `connection.rs`, `state/postgres.rs`, `server.rs` + partner CSV |
| Snapshot class | Attribution / idle-hold evidence — **not** peak-exhaustion proof |
| Product fix | App budget + identity + reaping + shutdown close — **not** raise `max_connections` to 400 |
| Pool API | Extend existing `PgPoolBundle` / `with_session_hygiene` — do not invent a second pool stack |
| `application_name` | `edgequake:<role>` (`query` \| `ingest` \| `queue` \| `admin`) |
| Docs vs code | Pack + **Waves A–E implemented** on HEAD (see `measurements/e2e112-gates.txt`); ship **v0.24.3** |

## Start here

1. [00-why.md](00-why.md)  
2. [00-incident-data.md](00-incident-data.md) + [measurements/BRUTAL-HONESTY.md](measurements/BRUTAL-HONESTY.md)  
3. [01-first-principles.md](01-first-principles.md)  
4. [03-root-cause.md](03-root-cause.md)  
5. [04-fix-plan.md](04-fix-plan.md)  
6. [07-ops-runbook.md](07-ops-runbook.md) (partner can act now)  
7. Lenses: [lenses/README.md](lenses/README.md)

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-090](../090-performance/) | `PgPoolBundle`, LAW-P4 session hygiene, multi-pool isolation |
| [SPEC-083](../083-improvements/) | Graceful HTTP drain (X-31) — extend with pool close |
| [SPEC-089](../089-health-check/) | Statement timeouts complementary to session idle nets |
| [SPEC-057](../057-pipeline-reliability/) | Pipeline / store reliability context; pool util in health |
| [SPEC-111](../111-issues/) | Pack structure / brutal honesty pattern |

## Out of scope (this pack)

- Partner infra change of PostgreSQL `max_connections`
- Mandatory PgBouncer deployment (recommended in ops runbook only)
- Reworking QL service pools (partner-owned)
