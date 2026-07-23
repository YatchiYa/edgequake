# INCIDENT — Production eq_* schema (SPEC-062 / P0 / X-03)

> **Severity**: P0  
> **Product**: EdgeQuake v0.20.2  
> **Companion studies**: [P0.md](defects/P0.md) · [X-03.md](defects/X-03.md) · [Cluster 00](clusters/00-schema-readiness/)  
> **Status of companion in source report**: missing from repo — recreated here

---

## 1. WHY (user-visible)

On large production AGE graphs (~178k+ nodes):

1. **Chat breaks** — SQL errors like `column e.eq_source_id does not exist`, or silent empty neighborhoods when columns exist but are NULL.  
2. **Ingest loops** — native merge `ON CONFLICT (eq_*)` without arbiter when columns/indexes missing.  
3. **PPD/staging shows nothing** — small graphs finish DDL instantly → zero symptom → ship to prod.

---

## 2. Root cause (first principles)

```
  SPEC-062 goal: denormalize node_id/source_id/target_id onto AGE child tables
                 for btree-friendly degree / incident-edge / upsert paths

  Mechanism:
    ADD COLUMN / CREATE INDEX need strong locks
         |
         v
    concurrent long agtype scans hold AccessShare for tens of minutes
         |
         v
    ALTER waits --> lock_timeout (5s in support/092) or statement timeout
         |
         v
    columns NEVER created (or partially backfilled)
         |
         +--> hot path SQL uses ONLY eq_*  (X-03) --> ERROR or empty
         +--> upsert ON CONFLICT (eq_*) without arbiter --> retry storm
```

**LAW-2 violated**: hot paths assumed optional schema was present.

Mitigations already in tree (PARTIAL):

- `migrations/support/092/apply.sql` — every-boot reconcile, `lock_timeout=5s`, `IF NOT EXISTS`
- SPEC-069 single-flight `ensure_indexes_lock` in graph lifecycle

**D-30 follow-up (2026-07-23, closed)**: native EDGE upserts require `eq_rel_type`, but
readiness / M092 previously treated the 2-col schema as complete and skipped DDL.
Fix (checksum-safe): extend **`migrations/support/092/apply.sql` only** (not versioned
`092_*` / `097_*` markers), align `eq_id_schema_ready` + `graph_eq_columns_ready` to
require `eq_rel_type` + `idx_edge_eq_source_target_rel`, and surface first merge cause
in persist errors.

**Incomplete AGE stubs**: leftover graphs without both `Node` and `EDGE` (e.g. historical
`bind_probe*`) used to fail `/ready` even though M092 correctly skips them. Ops:  
`SELECT * FROM ag_catalog.drop_graph('<name>', true);` — and M092 readiness now scores
only graphs that already have both child tables.

**Still monitor**:

- Boot gate / property fallback coverage on partial backfill graphs  
- Prefix scans with `eq_* IS NOT NULL` on non-backfilled edges  

Locus:

- [`nodes_ops/read.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/graph/nodes_ops/read.rs) (~148–171)  
- [`edges_ops.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/graph/edges_ops.rs) (~360–365)  
- [`scan_ops.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/graph/scan_ops.rs)  
- [`support/092/apply.sql`](../../edgequake/migrations/support/092/apply.sql)

---

## 3. Immediate remediation (ops)

### Preferred: maintenance-window reconcile (SSOT)

Do **not** invent ad-hoc DDL that drifts from `migrations/support/092/apply.sql`.

1. **Pause heavy traffic** (query + ingest / workers) on the graph workspace.
2. Set maintenance env and restart **one** API process (or run bootstrap reconcile):

```bash
export EDGEQUAKE_EQ_MAINTENANCE=1
# Optional explicit override (defaults to 120s when maintenance=1):
# export EDGEQUAKE_GRAPH_DDL_LOCK_TIMEOUT=120s
# Keep fallback if columns still partial during the window:
export EDGEQUAKE_EQ_ID_FALLBACK=1
# Then start API / trigger migration bootstrap so reconcile_migration_092 runs.
```

What maintenance mode changes:

| Knob | Boot (default) | `EDGEQUAKE_EQ_MAINTENANCE=1` |
|------|----------------|-----------------------------|
| `lock_timeout` | `5s` (fail-fast) | `120s` |
| NULL-only backfill | single `UPDATE … WHERE eq_* IS NULL` | ctid batches (~10k rows) with NOTICE |
| X-03 readiness | unchanged — still probes columns / `/ready` | unchanged |

3. **Verify** columns + non-null coverage:

```sql
SELECT column_name FROM information_schema.columns
WHERE table_schema = '<graph>' AND column_name LIKE 'eq_%';

SELECT
  (SELECT count(*) FROM "<graph>"."Node" WHERE eq_node_id IS NULL) AS null_nodes,
  (SELECT count(*) FROM "<graph>"."EDGE"
     WHERE eq_source_id IS NULL OR eq_target_id IS NULL) AS null_edges;
```

4. Unset `EDGEQUAKE_EQ_MAINTENANCE`, restart normally. Clear `EDGEQUAKE_EQ_ID_FALLBACK` only when null counts are zero and indexes exist.

Health: chat local/mix mode returns neighbors; ingest upsert no longer loops.

### Manual SQL fallback (only if bootstrap cannot run)

```sql
-- Per AGE graph schema G (repeat for each ag_catalog.ag_graph.name)
SET lock_timeout = '120s';
SET statement_timeout = 0;

-- Retry loop externally until success
ALTER TABLE G."Node" ADD COLUMN IF NOT EXISTS eq_node_id text;
ALTER TABLE G."EDGE" ADD COLUMN IF NOT EXISTS eq_source_id text;
ALTER TABLE G."EDGE" ADD COLUMN IF NOT EXISTS eq_target_id text;

-- NULL-only backfill (batch by ctid on huge tables — see support/092 maintenance path)
UPDATE G."Node"
SET eq_node_id = ag_catalog.agtype_to_json(properties)->>'node_id'
WHERE eq_node_id IS NULL;

UPDATE G."EDGE"
SET eq_source_id = ag_catalog.agtype_to_json(properties)->>'source_id',
    eq_target_id = ag_catalog.agtype_to_json(properties)->>'target_id'
WHERE eq_source_id IS NULL OR eq_target_id IS NULL;

-- Then create unique/partial indexes + sync triggers (see support/092)
```

---

## 4. Engineering fix (required)

| Step | Owner primitive | Detail |
|------|-----------------|--------|
| 1 | `SchemaReadiness` | Probe catalogs at boot; expose `/health` component `eq_id_schema` |
| 2 | Fail closed | If not ready → do not serve query/ingest (503) **or** enable fallback mode flagged in metrics |
| 3 | Fallback SQL | `COALESCE(eq_source_id, agtype_to_json(properties)->>'source_id')` until backfill complete |
| 4 | Scans | Never filter `eq_* IS NOT NULL` without OR property path |
| 5 | Contracts | Replace vacuous native_upsert tests ([C-20](defects/C-20.md)) |
| 6 | DDL | Keep single-flight; prefer maintenance reconcile over hot-path ALTER |

```
  Boot                         Hot path
  ----                         --------
  reconcile eq_* DDL  -->  schema_ready=true/false
  (lock_timeout+retry)         |
                               +--> ready: eq_* btree path
                               +--> !ready: 503 OR COALESCE fallback + metric
```

---

## 5. Edge cases

| ID | Case | Mitigation |
|----|------|------------|
| EC-P0-1 | Multi-graph workspace; one graph ready, one not | Per-graph readiness; refuse only affected workspace |
| EC-P0-2 | Columns exist, backfill incomplete | Fallback + progressive backfill job |
| EC-P0-3 | Trigger missing on new writes | Reconcile creates sync triggers; metric on NULL eq_* inserts |
| EC-P0-4 | lock_timeout storms at boot | Cap retries; surface NOTICE; don't block forever |
| EC-P0-5 | PPD green, prod red | Load test DDL under concurrent agtype before release |

---

## 6. E2E / contract tests

| Test | Assertion |
|------|-----------|
| `e2e_schema_ready_refuses_traffic` | Missing eq_* → 503 on query/ingest |
| `e2e_degrees_match_property_fallback` | Fallback degrees == property-derived degrees |
| `contract_eq_columns_present_after_reconcile` | After 092, columns+indexes exist |
| `contract_native_upsert_eq_arbiter` | Asserts real ON CONFLICT targets ([C-20](defects/C-20.md)) |
| `e2e_chat_local_mode_without_eq_columns_degraded` | Fallback path returns non-empty when props present |

---

## 7. Exit criteria (Sprint 0)

- [ ] Production graphs have eq_* columns + indexes + triggers — **residual P0 PARTIAL** (large-AGE DDL)  
- [x] Boot gate or fallback live; chat no longer hard-fails on missing columns — **X-03 FIXED**  
- [x] Prefix scans do not drop non-backfilled edges — mitigated via COALESCE fallback (X-03)  
- [x] Contract tests assert eq_* arbiter — **C-20 FIXED**  
- [x] Runbook linked from ops docs — maintenance window steps in §3 (`EDGEQUAKE_EQ_MAINTENANCE=1`)
