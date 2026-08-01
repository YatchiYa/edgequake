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

## Symptom C — Delete / bulk-delete dual-SSOT (Waves 9–11)

Documents page shows **Deleting N document(s)** with **Document removed** in the feedback zone, while the Documents table still lists the same rows as **Completed / Ready**.

### Five WHYs (C)

1. Why do badges disagree? Feedback zone uses a module deletion-session Map; the table uses React Query from `GET /documents`.  
2. Why does the list stay Completed? List merge prefers SQL terminal success over KV `deleting` because `deleting` is not treated as inflight.  
3. Why is SQL still completed? Admit writes KV `deleting` only; `documents_valid_status` CHECK historically forbade `deleting` / `delete_failed`.  
4. Why is bulk worse? Batch admit never sets per-doc `deleting`; FE poll treats shared `batch_track_id` completion as per-doc success with zero stats → “Document removed”.  
5. Why harden now? Selected multi-delete (GH-317 API) is shipped, but list honesty and FE pins lag — operators cannot trust delete UX on large corpora.

### Causal chain (C)

```ascii
 Batch/single delete 202
   → (batch: no per-doc deleting admit)
     → KV deleting XOR SQL completed (CHECK forbade deleting)
       → merge_document_summaries: SQL success overwrites
         → FE list Completed/Ready + session "Document removed"
```

## Symptom D — Delete failure honesty (Wave 12)

After W9–W11, cascade failures surface as feedback **Delete failed** / **Deletion failed**, but the table shows pipeline **Failed**, the header still says **Deleting N**, and **Retry Failed** offers reprocess for lifecycle deletes.

### Five WHYs (D)

1. Why does the table say Failed? Shell `normalize_documents_column_status` maps `delete_failed`→`failed` and `deleting`→`cancelled`.  
2. Why is the reason generic? Batch task returns `failed_ids` without per-id reasons; FE defaults to “Deletion failed”.  
3. Why “Deleting N” when all failed? Feedback header counts all non-dismissed sessions, not only `active`.  
4. Why Retry Failed? `status_counts.failed` buckets `delete_failed` with pipeline failures; reprocess skips delete lifecycle.  
5. Why harden now? Operators cannot distinguish cascade delete failure from ingest failure, and shell collapse undoes LAW-098-9 dual-write.

### Causal chain (D)

```ascii
 Batch cascade Err → KV delete_failed
   → shell upsert remaps SQL to failed
     → list merge / badge = Failed
       → Retry Failed offers reprocess
         → feedback header still "Deleting N"
```

## Symptom E — Cascade post-proof / shared prune (Wave 13)

Honest **Delete failed** with `Post-proof failed: N nodes and M edges still reference document sources` on dense shared KG (e.g. hyper-connection variants). Counts under discovery LIMIT — not truncation.

### Five WHYs (E)

1. Why does post-proof still see sources? Shared prune upserted remaining `source_ids` via native `ON CONFLICT`.  
2. Why did remaining not stick? `eq_merge_graph_properties` **unions** existing∪incoming → deleted chunks restored.  
3. Why did memory tests pass? Memory adapter replaces properties; AGE merge is different.  
4. Why fail-closed? Post-proof aborts KV wipe so provenance cannot orphan silently.  
5. Why harden now? Operators cannot delete shared-entity docs until subtractive writes use Replace.

### Causal chain (E)

```ascii
 Cascade shared prune → upsert_nodes_batch (MergeSources)
   → eq_merge unions pruned source_ids back
     → post_proof_source_absent finds hits
       → delete_failed (fail-closed)
```

## Symptom F — Edge provenance / multigraph cascade (Wave 14)

Honest **Delete failed** with `Post-proof failed: 0 nodes and N edges still reference document sources` (e.g. `science_one.extracted.md`). Nodes clean; edges remain.

### Five WHYs (F)

1. Why post-proof sees edges? Discovery finds rows whose provenance still names the document.  
2. Why cascade missed sisters? Collapse keyed `(src, tgt)` while arbiter is `(src, tgt, rel_type)`.  
3. Why exclusive edges become “shared updates”? `collect_source_references` treated edge topology `source_id` (`workspace::ENTITY`) as provenance.  
4. Why arrays look like entity ids? Rebuild wrote remaining endpoint strings into `source_ids` / `source_chunk_ids` and left singular `source_chunk_id` uncleared.  
5. Why harden now? Retry can pass GIN-only post-proof while orphan singular citations remain — delete is not trustworthy.

### Causal chain (F)

```ascii
 Edge props: source_id=ws::ENTITY + source_chunk_id=doc-chunk-N
   → collect_source_references keeps endpoint → remaining never empty
     → false shared Replace upsert poisons source_ids
       → (src,tgt) collapse skips multigraph sisters
         → post_proof: 0 nodes, N edges → delete_failed
```

## Reply

1. Ensure relational spine on every typed entity path (including saturated KEEP).  
2. Normalize relation types; fail closed with miss evidence; migration 139 for historical entity gaps.  
3. Enforce a **single** EDGE arbiter `(eq_source, eq_target, eq_rel_type)`; drop legacy UNIQUEs every boot; restore `eq_merge_graph_properties`; Cypher MERGE includes `relation_type`; dedupe every ON CONFLICT batch writer (AGE + relationship sink).  
4. Dual-write lifecycle admit (KV + SQL `deleting`); protect `deleting` in list merge; one FE delete SSOT with pins until absence.  
5. Pass through lifecycle statuses in shell writers; batch per-id failure reasons; UI verbs match lifecycle (not pipeline Failed / Retry Failed).  
6. Cascade shared prune uses property **Replace** (not ingest union merge); prove with AGE e2e.  
7. Provenance SSOT ignores edge topology `source_id`; discovery/cascade/delete key `(src,tgt,rel)`; clear singular citation fields; singular orphan discovery for poisoned arrays.  
8. Prove with CI e2e + measured upsert performance on PG16/17/18 + delete list-honesty + failure-honesty + cascade-prune + edge-provenance gates.
