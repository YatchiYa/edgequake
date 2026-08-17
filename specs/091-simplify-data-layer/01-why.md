# 01 — WHY

> Derives from [00-raw-needs.md](00-raw-needs.md) "Starting with WHY"; sharpened by code verification in [03-assessment.md](03-assessment.md).

## The problem in one paragraph

EdgeQuake declares one logical object — a chunk of a document — across four physical representations and populates three of them with **three independent commits and two different identities**. Chunk text lives in a generic key-value table under a derived string key (`{doc_id}-chunk-{n}`); embeddings live in runtime-created `eq_*_vectors` tables under a `TEXT` primary key; entities and relationships live in the AGE graph. The relational `chunks` table — the only representation that can hold a foreign key, cascade a deletion, or prove a count — is created by three separate files, read by live statistics code, and **written by no one**. Every symptom this spec fixes (orphaned retrieval units, zero-valued workspace statistics, deletion residue, impossible foreign keys, quarantined compensations) is a downstream consequence of that single arrangement.

## Five WHYs

### WHY 1 — Why do workspace statistics report zero chunks and zero embeddings for every workspace?

Because `pg_get_workspace_stats` computes **both** figures from `(SELECT COUNT(*) FROM chunks WHERE workspace_id = $1)` — twice, verbatim — and nothing has ever inserted a row into `chunks` (`edgequake/crates/edgequake-core/src/workspace_service_impl/workspace_ops.rs:444-454`). Two distinct facts share one expression over an empty relation. (F-091-11, F-091-02)

### WHY 2 — Why does the live read path depend on a table nobody writes?

Because the schema declares intent (`chunks.content TEXT NOT NULL`, migration `001`, `edgequake/migrations/001_init_database.sql:205-219`) while the write path implements something else: `build_chunk_kv_records` persists text to KV and `build_chunk_vector_metadata` deliberately omits inline content, citing `content_ref` only (`edgequake/crates/edgequake-pipeline/src/chunk_storage.rs:1,12-22,63-108`). Schema and writer evolved in different files, in different crates, with no contract between them. (F-091-02, F-091-15)

### WHY 3 — Why was the drift never caught by a constraint, a test, or a type?

Because the two identities — relational `chunks.id uuid` vs. derived `{doc}-chunk-{n}` text — live in different type systems joined only by convention. No foreign key can span them (`eq_*_vectors` is created by application code at runtime, outside the migration system, so migrations cannot reference it), no Rust type encodes "this string is a chunk key," and presence can only be *probed* (migration 093's read-only function, self-described as "not RAG ANN SSOT") rather than *asserted*. Integrity that cannot be declared cannot be enforced. (F-091-03, F-091-04; LAW-D2)

### WHY 4 — Why does the system span three uncoordinated stores for one chunk tuple?

Because the generic KV abstraction promised provider portability, and the vector/graph stores grew as adapters behind storage-shaped traits (`KVStorage`, `VectorStorage`, `GraphStorage` in `edgequake/crates/edgequake-storage/src/traits/`). The released product ships exactly two production adapters — PostgreSQL and memory — so the abstraction preserves no actual portability, yet it forces every chunk fact through string keys and opaque JSONB, beyond the reach of constraints, joins, and cascades. Portability was paid for but never delivered, because it was placed at the wrong boundary (LAW-D7; LD-05).

### WHY 5 — Why is the wrong boundary so expensive to keep?

Because it converts ordinary relational facts into operational risk. One logical write becomes three commits with after-the-fact compensation (`edgequake/crates/edgequake-storage/src/compensation.rs`) and a durable quarantine with no guaranteed drainer (F-091-05). One logical delete becomes a reconciliation across stores. One logical count becomes three disagreeing counters (F-091-08). Schema lifecycle moves into request-serving code whose `ALTER TABLE` errors are discarded (`edgequake/crates/edgequake-storage/src/adapters/postgres/vector/ddl.rs:267-285`), so relations silently fall a generation behind. The migration history is already paying the bill in arrears: 039 dropped an embedding column nothing wrote, 091 converted a generated tsvector that nothing populated, 093 added a presence probe for what a foreign key should assert.

**Root cause (LAW-D2 + LAW-D5 + LAW-D6):** the system never settled *one identity, one authoritative row, one schema owner* for a chunk. Everything else is interest on that omission.

## Causal chain (ASCII)

```ascii
  ROOT CAUSE                          MECHANISM                            SYMPTOM (Finding ID)
 ┌──────────────────────┐   ┌──────────────────────────────┐   ┌─────────────────────────────────────┐
 │ Two chunk identities │   │ No FK can span uuid <-> text │   │ Orphaned retrieval unit possible    │
 │ uuid spine (unused)  │──▶│ Joins need casts/fallbacks   │──▶│ (F-091-01)                          │
 │ vs {doc}-chunk-{n}   │   │ Presence probed, not asserted│   │ Stats read empty table -> zeros     │
 └──────────────────────┘   └──────────────────────────────┘   │ (F-091-11)                          │
 ┌──────────────────────┐   ┌──────────────────────────────┐   │ Deletion reconciled, not cascaded   │
 │ Three stores, three  │   │ Partial chunk states visible │──▶│ (F-091-01, F-091-05)                │
 │ commits, no fence    │──▶│ Compensation + quarantine    │   │ Quarantine without drainer          │
 │ (LAW-D3 violated)    │   │ Counters disagree (LAW-D4)   │   │ (F-091-05, F-091-08)                │
 └──────────────────────┘   └──────────────────────────────┘   └─────────────────────────────────────┘
 ┌──────────────────────┐   ┌──────────────────────────────┐   ┌─────────────────────────────────────┐
 │ Runtime DDL owns     │   │ Migrations cannot govern the │   │ Relations a generation behind,      │
 │ eq_* relations       │──▶│ whole tuple -> boot patches  │──▶│ silently (F-091-04, F-091-10)       │
 │ (LAW-D5 violated)    │   │ schema, discarding errors    │   │ ef_construction: 3 values, 3 files  │
 └──────────────────────┘   └──────────────────────────────┘   │ (F-091-14)                          │
                                                               │ 8-hex legacy slugs persist (F-091-12)│
                                                               └─────────────────────────────────────┘
```

## Cost of doing nothing

| Axis | Today (verified) | Compounding effect |
| --- | --- | --- |
| Correctness | Orphaned chunk↔embedding states are representable; nothing forbids them (F-091-01) | Every new store or projection multiplies presence combinations |
| Operability | Boot-time DDL with discarded errors; relation discovery via `tablename LIKE 'eq_%_kv'` (F-091-04, F-091-10) | Each workspace generation adds tables migrations cannot see or govern |
| Reporting | Workspace stats return `0` for chunks and embeddings on every deployment (F-091-11) | Product metrics, capacity dashboards, and tenant billing built on zero |
| Deletion | Cross-store reconciliation + quarantine; no completion proof (F-091-05) | Residency/GDPR-grade deletion claims remain unprovable |
| Tuning | Same HNSW knob set to 32, 64, and 128 in three schema sources (F-091-14) | Index rebuilds silently change recall/size depending on which code path runs |
| Delivery | Migrations are hand-run runbooks | Every future data migration re-pays the coordination cost this spec removes once |

## What this spec is NOT doing

- Not replacing PostgreSQL — the opposite: it commits the spine to PostgreSQL 18 and keeps replaceable engines (vector, lexical, graph, blob) behind domain ports (LD-04, LD-05).
- Not touching the vector-engine strengths verified in the assessment: halfvec default, cosine-only enforcement, iterative filtered scans, exact reorder, statement-level counters, idempotent compensation all survive unchanged.
- Not a big-bang rewrite: six sequencing invariants, five gated waves, one irreversible step per release, flags with dual reads throughout (LD-07; [06-implementation-plan.md](06-implementation-plan.md)).
