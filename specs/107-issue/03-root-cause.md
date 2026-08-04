# 03 — Root Cause (SPEC-107)

Short RCA per partner symptom. Deep write-ups: SPEC-104 issue docs linked in [02](02-cross-ref-matrix.md).

## E1 — `column "id" does not exist` on `workspaces`

**Not a missing migration.** DDL PK has been `workspace_id` since [`001_init_database.sql`](../../edgequake/migrations/001_init_database.sql).

Pre-fix StorageInspector INV-D2 ran:

```sql
SELECT EXISTS (SELECT 1 FROM workspaces WHERE id::text = $1)
```

once per `eq_%_kv` / `eq_%_vectors` table each hourly inspect → ~96 failures/hour ≈ **~2300/day**. Fail-open `.unwrap_or(true)` hid orphans while Postgres logged every probe.

**Fix (HEAD):** `WHERE workspace_id::text = $1` in [`storage_inspector.rs`](../../edgequake/crates/edgequake-api/src/storage_inspector.rs).

## E2 — `relation "edgequake.Node" does not exist`

Partner CTE is INV-C `inv_c_gin_node_counts_by_prefixes` joining `{graph}."Node"` on `source_ids` containment.

0.22.0 inspector defaulted `graph_name = "edgequake"`. Live AGE graph for namespace `default` is **`eq_eq_default_graph`** via `age_graph_name_for_namespace`. Hourly run → **24×/day** `42P01`.

This is **wrong graph name**, not SPEC-039 missing labels (those fail on the correct schema name during ingest).

**Fix (HEAD):** `InspectorConfig::for_namespace("default")` → `eq_eq_default_graph`.

## E3 — INV-03 CRITICAL (20 indexed docs, no KV chunks)

**True data integrity alarm** on 0.22.0. Documents with `status = 'indexed'` and neither `public.chunks` nor legacy KV `{id}-chunk-%`. Sample `LIMIT 20` is a **ceiling**, not a proven exact total; severity Critical when count ≥10.

Plausible producers: forward-compensating SAGA (chunk delete without status rollback), incomplete delete cascade, legacy import. Healthy persist refuses completed-with-zero-chunks; orphans are usually **post-success residue**.

Monitor dual-read fixed in SPEC-104. **Orphan rows remain until ops requeue or delete** — see [04](04-residual-ops.md). SPEC-107 adds `RepairAction::LogOnly` guidance so repair recommendations are not silent (`_ => None`).

## E4 — `tenants_slug_key` unique violation

Plain `INSERT INTO tenants` with a **new** `tenant_id` every call collided on UNIQUE `slug` under retry (`novagen-orga` / suffixed slug race in ~1 minute).

**Fix (HEAD):** `INSERT ... ON CONFLICT (slug) DO UPDATE ... RETURNING` get-or-create; same name → Ok (HTTP 200); different name → `Error::Conflict` (HTTP 409). Raw `23505` should not surface on the app path.
