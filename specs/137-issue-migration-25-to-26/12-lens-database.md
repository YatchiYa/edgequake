# 12 — Lens: Database Expert

## Apply modes

| Mode | What sqlx sees |
|------|----------------|
| ExpandableOnly | Migrator **without** 125/126/131; `ignore_missing` true; 142 omitted while `any_legacy_rows` |
| All | Full `MIGRATOR.run` after consent — applies **pending** 125/126/131 even if 149 already recorded |

sqlx 0.8 applies unapplied older versions after newer ones exist
([PR #1030](https://github.com/launchbadge/sqlx/pull/1030)). That is why
ExpandableOnly-then-confirm-drop is valid.

## Guards (do not weaken)

- **125:** durable KV not represented in typed SSOT → Wave D ABORT; then `DROP TABLE eq_*_kv`.
- **126:** uncovered `*-chunk-*` vectors → W4 ABORT; DELETE chunk rows; drop table if empty.
- **131:** uncovered entity/rel/report (provenance `legacy_vector_id`) → IW2 ABORT; drop remaining `eq_*_vectors`.
- **142:** COUNT>0 on leftover `eq_*` → abort; empty tables `DROP TABLE`.

Cast law (SPEC-111): extract key `::uuid`, do not `(uuid_col)::text = text`.

## Locks

sqlx migrate takes an advisory lock. `print_failure_hint` may mention
`pg_locks` **only** when the error class is lock/timeout on DDL — not on
`RAISE EXCEPTION` abort text.

## AGE

125/126/131 touch `public.eq_*` only. Graph data lives in AGE namespaces.
Use `SELECT * FROM ag_catalog.drop_graph('name', true)` if a graph must go
([manual](https://age.apache.org/age-manual/master/intro/graphs.html)).
Never `DROP SCHEMA … CASCADE` for AGE.

## 149

`ALTER TABLE tasks ADD COLUMN IF NOT EXISTS document_id` on a RANGE-partitioned
parent propagates to partitions. Index `IF NOT EXISTS`. Backfill only
pending/processing rows.
