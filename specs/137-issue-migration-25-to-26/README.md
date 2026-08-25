# SPEC-137 — 0.25→0.26 migrate (mid-cutover DROP OLD)

> **Trigger:** Field fleet on **v0.25.0** schema serving, upgrading to **v0.26.0**.
> `edgequake migrate` applies SAFE SCHEMA; remaining DROP OLD does not complete
> when the operator passes `--drop-confirm` (pre-fix) or when SQL guards abort.
> **Method:** First principles (code is law) + reconstructed logs + e2e gates.
> **Broken through:** **v0.26.0** CLI consent token + 0.26 upgrade runbook gap.
> **Target cut:** **v0.26.1**.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  v0.26.0 only adds SAFE SCHEMA 149 (tasks.document_id).                      │
│  Serving on 0.25 with pending 125/126/131 is legal (LD-15).                  │
│  Ticket is the SPEC-091 mid-cutover ladder, not a 149 bug.                   │
│                                                                              │
│  Track A: --drop-confirm was not consent (--confirm-drop only).              │
│           Unknown apply flags were ignored.                                  │
│  Track B: --confirm-drop runs SQL; abort is fail-closed (keep guards).       │
│           Abort hint wrongly pointed at tasks/pg_locks.                      │
│                                                                              │
│  Fix: consent SSOT + alias; reject unknown flags; classify aborts;           │
│       honest 0.26 upgrade runbook; e2e 137-01..09. Do NOT weaken DROP SQL.   │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| ID | Item | Verdict | Evidence |
|----|------|---------|----------|
| F1 | 0.26 is additive 149 only | **Locked** | [00-why](00-why.md), mig 149 |
| F2 | Leftover 125/126/131 legal on 0.25 serve | **Locked** | LD-15 / `pending_ok_to_serve` |
| F3 | `--drop-confirm` ≠ consent (pre-fix) | **Closed (code)** | [03](03-root-cause.md) Track A |
| F4 | Unknown `--*` apply flags ignored (pre-fix) | **Closed (code)** | `dispatch_migrate` |
| F5 | Abort hint Wave-D-only + generic pg_locks | **Closed (code)** | [11-lens-fullstack](11-lens-fullstack.md) |
| F6 | 0.26 runbook omits 091 ladder | **Closed (docs)** | [upgrade-to-0.26.0.md](../../docs/operations/upgrade-to-0.26.0.md) |
| F7 | SQL guards must stay fail-closed | **Locked** | LAW-137-3 |
| E2E | CLI + abort class + soak proof | **Gates** | [05](05-e2e-test-matrix.md) |

## Document map

```ascii
 00-why / 00-issue-data / raw-logs/
   → 01-first-principles (LAW-137-1..8)
   → 02-cross-ref-matrix
   → 03-root-cause
   → 04-fix-plan
   → 05-e2e-test-matrix
   → 06-edge-cases
   → 07-similar-issues
   → 08-operator-reply
   → 09-ops-runbook
   → 10-lens-product-owner
   → 11-lens-fullstack
   → 12-lens-database
   → 13-lens-ai-engineer
   → measurements/
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Consent tokens | `--confirm-drop` canonical; `--drop-confirm` alias; env unchanged |
| Unknown apply flags | Non-zero exit + hint (did you mean `--confirm-drop`?) |
| `--confirm` alone | **Not** drop consent (too broad) |
| SQL 125/126/131/142 | Do not weaken; abort is correct on uncovered rows |
| AGE | Never `DROP SCHEMA CASCADE` on graph namespaces |
| 149 | SAFE SCHEMA; no confirm required |
| Proof | `make spec137-migrate-025-026-proof` |

## Cross-spec anchors

| Spec / doc | Relevance |
|------------|-----------|
| [SPEC-091](../091-simplify-data-layer/) | Expand vs destroy; console; 125/126/131 |
| [SPEC-105](../105-fix-legacy/) | Migration 142 deferred assert |
| [SPEC-110](../110-migration-issue/) | Blocking migrate pack template |
| [SPEC-111](../111-issues/) | Coverage / provenance; checksum repair |
| [upgrade-to-0.26.0](../../docs/operations/upgrade-to-0.26.0.md) | Operator 0.25→0.26 |
| [AGE graphs](https://age.apache.org/age-manual/master/intro/graphs.html) | `drop_graph`, not DROP SCHEMA |

## DRY rule

Consent token, abort class, and irreversible version set each have **one** SSOT
(`migrate_console` + `IRREVERSIBLE_DROP_VERSIONS`). If this pack and
`NOTES.md` disagree on editing applied drop SQL, **do not edit** 125/126/131
bodies unless a field DB is stuck *inside* a failing version (LAW-MIG / SPEC-110
LAW-M3). Track A/B are CLI/docs/honesty, not new drop SQL.

## Out of scope

- Auto `--confirm-drop`
- crates.io publish of workspace crates
- Acc re-score
- AGE graph rebuild

## Start here

1. [00-why.md](00-why.md)
2. [00-issue-data.md](00-issue-data.md)
3. [01-first-principles.md](01-first-principles.md)
4. [03-root-cause.md](03-root-cause.md)
5. [04-fix-plan.md](04-fix-plan.md)
6. [09-ops-runbook.md](09-ops-runbook.md)
