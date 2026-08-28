# 00 — Why SPEC-139

## Trigger

A field fleet on **v0.26.1** (`ghcr.io/raphaelmansuy/edgequake:0.26.1`) has
already applied SAFE SCHEMA through **149**. DROP OLD **125 / 126 / 131** remain
(legal to serve — SPEC-137). The operator then:

1. `edgequake migrate` (expandable — OK)
2. `edgequake guard` (RED — expected)
3. Starts the API so the SPEC-091 engine can copy residue
4. Engine logs `w3-chunk-embedding-backfill` verification FAILED, then
   `iw2 entity insert failed: ON CONFLICT DO UPDATE command cannot affect row a second time`
5. `guard` still RED; `--confirm-drop` Wave D ABORTs (~2592 KV rows)

Consent tokens are not the bug (SPEC-137 shipped in 0.26.1). The **copy engine**
cannot finish, so DROP stays correctly fail-closed.

## User impact

| Layer | Impact |
|-------|--------|
| Ops | Engine dies; `uncovered_fleet` never decreases; leftover KV families plateau |
| Product | 0.26.1 looks like a migrate regression after 137 “fixed migrate” |
| Serving | API may already be legal (SAFE SCHEMA complete) |
| Trust | Guard RED + crash log with no honest remainder path |

## Why this is a product defect

- Postgres `21000` on a single UNNEST is deterministic when normalize-join maps
  two legacy ids to one typed `entity_id`. The serving vector upsert already
  dedupes (QW2); iw2 did not.
- W3 fleet verify compares summed per-table **expected** to `max(global typed count)`.
  That cannot equal 126’s coverage predicate. A false FAIL marks the job
  `failed`; boot never reclaims it.
- sqlx 119 copies artifacts only when `documents` exists; 122 creates those
  shells **later**. 119 will not re-run. The engine had no remainder descriptor.

SQL abort on uncovered rows is **not** a defect. Weakening 125/126/131 would be.

## Non-goals

- Auto-applying DROP OLD
- Editing applied 117–122 / 125–131 bodies
- Re-scoring Acc / PDF geometry
- Multi-embedding PK for alias legacy keys (stalls stay SPEC-111)

## Success condition

1. Two normalize-colliding `entity:` keys in one batch do not 21000; typed row exists.
2. W3 `verify.actual` is SUM of per-table coverage, not `max(COUNT(*) FROM chunk_embeddings)`.
3. A `failed`+`verify_failed` W3 job is reclaimed on boot.
4. After 122 creates a shell, remainder copies leftover lineage.
5. One job `Err` does not skip stamp / remainder.
6. `make spec139-migrate-engine-proof` green.
