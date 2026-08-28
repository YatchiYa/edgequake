# SPEC-139 — Mid-cutover engine stall (0.26.1 field)

> **Trigger:** Field fleet on **v0.26.1**. `edgequake migrate` applies SAFE SCHEMA
> (through 149). `guard` stays RED. Server engine hits `w3` verify FAIL then
> `iw2` Postgres `21000`, then `--confirm-drop` Wave D ABORTs.
> **Method:** First principles (code is law) + field logs + unfakable Postgres e2e.
> **Broken through:** **v0.26.1** (and HEAD until this pack).
> **Target cut:** **v0.26.3**. Schema train stays **149**.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  SAFE SCHEMA 149 is already applied. DROP OLD 125/126/131 is blocked by      │
│  unfinished copy — not by CLI consent (SPEC-137 is fine on 0.26.1).          │
│                                                                              │
│  Track A: iw2 UNNEST + ON CONFLICT DO UPDATE proposes the same entity_id     │
│           twice (normalize join). Postgres 21000. Fleet never moves.         │
│  Track B: w3 verify SUMs per-table expected but takes MAX(global typed).     │
│           Job goes failed; claim_lease never reclaims.                       │
│  Track C: sqlx 119 (artifacts) runs BEFORE 122 (shells). No remainder job.   │
│           lineage/MM/hash residue plateaus.                                  │
│                                                                              │
│  Fix: within-batch arbiter dedupe; coverage-sum verify; reclaim verify-fail; │
│       remainder jobs (dedup / shell / artifact); engine continues after Err. │
│  Do NOT weaken DROP SQL 125/126/131.                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| ID | Item | Verdict | Evidence |
|----|------|---------|----------|
| F1 | iw2 21000 = within-batch duplicate arbiter | **Code** | [03](03-root-cause.md) Track A |
| F2 | W3 `actual = max(COUNT(*) global)` | **Code** | [03](03-root-cause.md) Track B |
| F3 | `failed` + verify is not reclaimable | **Code** | `claim_lease` state filter |
| F4 | 119-before-122 leftover artifacts | **Code** | migrations 119 then 122 |
| F5 | DROP guards stay fail-closed | **Locked** | LAW-139-7 / LAW-137-3 |
| E2E | Unfakable 139-01..08 + proof target | **Gates** | [05](05-e2e-test-matrix.md) |

## Document map

```ascii
 00-why / 00-issue-data / raw-logs/ / logs-folder/
   → 01-first-principles (LAW-139-1..7)
   → 02-cross-ref-matrix
   → 03-root-cause
   → 04-fix-plan
   → 05-e2e-test-matrix
   → 06-edge-cases
   → 07-similar-issues
   → 08-operator-reply
   → 09-ops-runbook
   → measurements/
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| iw2 SQL | Keep `ON CONFLICT DO UPDATE` provenance COALESCE; dedupe in Rust |
| W3 verify actual | Per-table coverage COUNT (≡ 126), then SUM |
| Equality vs coverage | Default `passes()` = coverage (`actual >= expected`). Equality only if `EDGEQUAKE_MIGRATION_VERIFY_EQUALITY=1` |
| Failed verify jobs | Reclaim to `pending`, reset cursor |
| Remainder | Engine jobs (`w2-dedup` / `wc-shell` / `w5-artifact`), not sqlx 150; do not edit applied 117–122 |
| Schema train | Stays **149** |
| Proof | `make spec139-migrate-engine-proof` |

## Cross-spec anchors

| Spec / doc | Relevance |
|------------|-----------|
| [SPEC-091](../091-simplify-data-layer/) | Engine, 117–122, DROP 125/126/131 |
| [SPEC-110](../110-migration-issue/) | LAW-M1 ON CONFLICT cardinality |
| [SPEC-111](../111-issues/) | LAW-C3 provenance; stamp stalls |
| [SPEC-137](../137-issue-migration-25-to-26/) | Consent CLI already honest on 0.26.1 |
| [Postgres INSERT](https://www.postgresql.org/docs/current/sql-insert.html) | DO UPDATE must not affect a row twice |
| [pganalyze U126](https://pganalyze.com/docs/log-insights/app-errors/U126) | SQLSTATE 21000 |

## DRY rule

Conflict-key last-write-wins is one helper used by entity / relationship / report
batches. Verify coverage SQL for W3 must match `count_uncovered_chunk_rows` /
migration 126 (LAW-C3). If this pack and DROP SQL disagree, **do not edit** DROP
bodies.

## Out of scope

- Auto `--confirm-drop`
- Changing `entity_embeddings` PK to many-embeddings-per-entity
- Acc re-score / AGE drop
- HNSW manifest drift warnings

## Start here

1. [00-why.md](00-why.md)
2. [00-issue-data.md](00-issue-data.md)
3. [01-first-principles.md](01-first-principles.md)
4. [03-root-cause.md](03-root-cause.md)
5. [04-fix-plan.md](04-fix-plan.md)
6. [09-ops-runbook.md](09-ops-runbook.md)
