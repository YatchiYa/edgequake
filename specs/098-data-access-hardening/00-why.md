# 00 — WHY (SPEC-098)

## Symptom A — Fleet spine miss (Waves 0–5)

Documents fail at knowledge-graph persist with:

```text
SPEC-091: typed fleet mirror resolved 0/N rows
(relational entity/rel FK miss or name mismatch —
bare entities.name must match entity:NAME; ensure
PostgresEntitySink wrote the spine before fleet mirror)
```

UI truncation can show this as “SPEC-691”. The in-repo id is **SPEC-091** (fleet); SPEC-098 hardens the spine invariant that SPEC-091 assumed.

### Five WHYs (A)

1. Why does persist fail? Fleet mirror inserts into `entity_embeddings` with FK → `entities.id`, and resolved **0** of N names.  
2. Why are rows missing? For saturated KEEP entities, the merger skipped `PostgresEntitySink` while still emitting vectors.  
3. Why skip the sink? SOURCE_IDS KEEP was designed to skip AGE description updates; the skip incorrectly bundled relational identity.  
4. Why wasn’t this caught? Contracts seeded `entities` before mirror; no e2e covered “AGE present + relational absent + saturated”.  
5. Why harden now? Typed embeddings are default; every re-ingest of saturated graph-only entities can fail production persist.

### Causal chain (A)

```ascii
 AGE entity already saturated (SOURCE_IDS KEEP)
   → merge skips node_batch AND sink_rows
     → vectors still collected (entity:NAME)
       → fleet mirror SELECT entities → 0 rows
         → typed hard fail → document Failed
```

## Symptom B — Edge upsert cardinality (Waves 6–8)

Documents fail at knowledge-graph persist with:

```text
Knowledge graph persist failed: Graph error: 1 knowledge-graph merge error(s)
during persist: Storage error: Database error: Native SQL edge batch upsert
failed: error returned from database: ON CONFLICT DO UPDATE command cannot
affect row a second time
```

PostgreSQL 16/17/18 raise SQLSTATE `21000` when one `INSERT … ON CONFLICT DO UPDATE` proposes two rows that collide on the arbiter **after BEFORE INSERT triggers** ([PG INSERT](https://www.postgresql.org/docs/18/sql-insert.html)).

### Five WHYs (B)

1. Why does persist fail? Native AGE edge upsert hits cardinality_violation on `EDGE`.  
2. Why cardinality? Two proposed rows share `(eq_source_id, eq_target_id, eq_rel_type)` (or an endpoint-only arbiter under schema drift).  
3. Why duplicates reach SQL? Cross-chunk / multi-type extractions; trigger can collapse keys; legacy 2-col UNIQUE may remain after boot early-exit.  
4. Why wasn’t this fully closed? v0.15.2 added Rust dedupe + 3-col arbiter, but `eq_id_schema_ready` skips legacy UNIQUE drop and `edge_eq_ok` accepts 2-col.  
5. Why harden now? Large PDF KG persist (e.g. hyper-connection papers) still fails production Materialize.

### Causal chain (B)

```ascii
 Multi-rel / duplicate (src,tgt[,rel]) in one batch
   → (schema drift: endpoint-only UNIQUE still present)
     OR (trigger collapses distinct eq_rel_type from properties)
       → INSERT ON CONFLICT DO UPDATE sees same arbiter twice
         → SQLSTATE 21000 → document Failed
```

## Reply

1. Ensure relational spine on every typed entity path (including saturated KEEP).  
2. Normalize relation types; fail closed with miss evidence; migration 139 for historical entity gaps.  
3. Enforce a **single** EDGE arbiter `(eq_source, eq_target, eq_rel_type)`; drop legacy UNIQUEs every boot; restore `eq_merge_graph_properties`; Cypher MERGE includes `relation_type`; dedupe every ON CONFLICT batch writer (AGE + relationship sink).  
4. Prove with CI e2e + measured upsert performance on PG16/17/18.
