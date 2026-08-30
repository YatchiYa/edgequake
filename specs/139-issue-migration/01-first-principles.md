# 01 — First principles (LAW-139)

> **Cross-refs**: [WHY](00-why.md) · [Issue data](00-issue-data.md) · [RCA](03-root-cause.md)

## Axioms

1. **PostgreSQL `INSERT … ON CONFLICT DO UPDATE` is deterministic.** One statement
   must not affect any existing row more than once. Duplicate arbiter values in
   the *proposed* set raise `21000` — not last-write-wins
   ([INSERT](https://www.postgresql.org/docs/current/sql-insert.html),
   [U126](https://pganalyze.com/docs/log-insights/app-errors/U126)).
2. **Normalize join is many-legacy → one typed.** `EntityNameIndex` maps
   `entity:Foo` and `entity:FOO` to one `entities.id`. The typed PK is
   `(model_id, entity_id)`. Collision in one UNNEST is expected at field scale.
3. **Drop readiness is coverage, not emptiness** (LAW-111-2). Verify `actual`
   must be the same join as migration 126 / 131, aggregated by **SUM** across
   legacy tables — never a global `COUNT(*)` taken with `max()`.
4. **sqlx one-shot ≠ engine remainder.** An applied 119 body will not re-run
   after 122 creates missing parents. Remainder is a descriptor, not a checksum
   rewrite (LAW-111-10).
5. **Expand ≠ destroy.** Engine bugs must not be “fixed” by weakening DROP SQL.

## Causal diagram

```text
  0.26.1 binary  (SAFE SCHEMA 149 applied)
           │
           ▼
  engine: w1 → w3 → iw2 → stamp
           │
           ├─ w3 verify: SUM(expected_table) vs max(COUNT chunk_embeddings)
           │     → FAIL → state=failed → never reclaim
           │
           ├─ iw2 UNNEST entity_id twice (normalize)
           │     → 21000 → run_engine Err → stamp skipped
           │
           └─ 119 skipped parent-less artifacts
                 122 later creates shells
                 no remainder job → lineage plateau
           │
           ▼
  guard RED; --confirm-drop Wave D / 126 / 131 ABORT (correct)
```

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-139-1** | **Conflict-key cardinality** — Every engine `INSERT … ON CONFLICT DO UPDATE` proposes ≤1 row per arbiter key per statement. Last-write-wins on `(model_id, entity_id)` / `(model_id, relationship_id)` / `(model_id, report_id)`. |
| **LAW-139-2** | **Alias ≠ crash** — Many-legacy → one typed is a stamp/stall (SPEC-111), never 21000. Unique `legacy_vector_id` conflicts increment `failed`. |
| **LAW-139-3** | **Verify actual ≡ drop coverage** — W3 `actual` is per-legacy-table coverage COUNT (chunks ⋈ chunk_embeddings), then SUM. Never `COUNT(*) FROM chunk_embeddings` and never `max()`. |
| **LAW-139-4** | **Verify-failed is retryable** — Boot reclaims `state=failed` with `last_error.verify_failed`, resets cursor to `pending`. Silent skip is a lie. |
| **LAW-139-5** | **Engine isolation** — One job `Err` logs and continues; batch TX still rolls back. Stamp and remainder must still run. |
| **LAW-139-6** | **Remainder after one-shot** — Families 117–122 that skip parent-less rows get idempotent engine jobs. Do not edit applied sqlx bodies. |
| **LAW-139-7** | **Unfakable proof** — Real Postgres 21000 on the broken fixture; patched path commits; 125/126/131 predicates unchanged. |

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | Dedupe helper owns cardinality; verify.rs owns coverage SQL; remainder jobs own 117/119 replay; DROP SQL owns destroy. |
| **O** | New fleet families add an arbiter key to the same helper. |
| **L** | Every DO UPDATE backfill obeys LAW-139-1 (same as LAW-M1). |
| **I** | No “force drop” / skip-guard flag. |
| **D** | Runbooks depend on job step ids + `migrate guard`, not ad-hoc SQL dumps. |
| **DRY** | One `dedupe_last_write_wins`; W3 coverage join shared with `count_uncovered_chunk_rows`. |
