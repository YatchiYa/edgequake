---
title: 'Deep Dive: Data Layer (Postgres / AGE / pgvector / FTS)'
---

# Data Layer — PostgreSQL, AGE, pgvector, and Text Search

> **Product: v0.19.0** · Contract: [OpenAPI snapshot](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

Where EdgeQuake stores information, how it is indexed, and how each query mode reads it. **Code is law** — physical names and SQL come from `edgequake-storage` adapters and `edgequake/migrations/`.

**Related:** [Graph Storage](graph-storage.md) · [Vector Storage](vector-storage.md) · [Query Modes](query-modes.md) · [Lineage Tracking](../architecture/lineage-tracking.md) · [Data Flow](../architecture/data-flow.md)

---

## Contents

1. [Mental model](#1-mental-model--three-stores--relational-sidecar)
2. [Physical naming and tenancy](#2-physical-naming-and-tenancy)
3. [PostgreSQL ER (relational)](#3-postgresql-er-schema-relational)
4. [KV store](#4-kv-store-document-text-ssot)
5. [Apache AGE](#5-apache-age-property-graph)
6. [pgvector](#6-pgvector)
7. [Text search (FTS)](#7-text-search-fts)
8. [Query mode × store matrix](#8-how-information-is-queried)
9. [Ingest write path](#9-write-path-summary-ingest)
10. [Migration and index map](#10-migration--index-map)
11. [Operator SQL cookbook](#11-operator-debugging-cookbook)

---

## 1. Mental model — three stores + relational sidecar

```
┌──────────────────────────────────────────────────────────────────────┐
│ EdgeQuake data layer (v0.19.0)                                       │
│                                                                      │
│  WRITE (ingest)                         READ (query)                 │
│  -------------                          ------------                 │
│  KV  doc/chunk text SSOT  <-----------  hydrate + FTS join           │
│  AGE Node / EDGE entities <-----------  Local/Global expand          │
│  pgvector embeddings      <-----------  ANN / filter_ids             │
│  relational sidecar       <-----------  lineage, PDF, tasks          │
└──────────────────────────────────────────────────────────────────────┘
```

| Store | Role | SSOT for RAG? |
| ----- | ---- | ------------- |
| **KV** (`eq_*_kv`) | Document metadata + chunk text JSON | **Yes** — chunk text |
| **AGE** (`eq_*_graph`) | Entities (Node) + relationships (EDGE) | **Yes** — graph |
| **pgvector** (`eq_*_vectors`) | Chunk / entity / relationship embeddings | **Yes** — vectors |
| **Relational** | PDF bytes, mm-assets, lineage links, tasks, CQRS `entities` | Sidecar / ops / analytics |

**Dual-SSOT warning:** Do not treat `public.documents` / `public.chunks` alone as the RAG corpus. Pipeline ingest writes **KV + AGE + vectors**. Relational `documents`/`chunks` support PDF linkage, lineage columns, and CQRS; `entities`/`relationships` are a **CQRS read model** (M039) optionally dual-written via `entity_sync_mode`.

EdgeQuake keeps vectors and the property graph in **one PostgreSQL instance** (same pattern as unified Graph-RAG deployments: pgvector ANN + Apache AGE traversal under one transaction boundary). Embeddings are **not** stored as AGE node properties — they live in dedicated `eq_*_vectors` tables linked by id / `source_*` properties.

---

## 2. Physical naming and tenancy

### Namespace → table prefix

From [`PostgresConfig::table_prefix`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/config.rs):

```text
namespace "default"
  --> table_prefix "eq_default"
  --> KV:      public.eq_eq_default_kv
  --> vectors: public.eq_eq_default_vectors
  --> graph:   eq_eq_default_graph   (AGE schema name)
```

Helpers prepend another `eq_` to the prefix:

```rust
// config.rs
format!("public.eq_{prefix}_kv")       // prefix already starts with eq_
format!("public.eq_{prefix}_vectors")
```

API boot typically uses `.with_namespace("default")`.

### Workspace vector tables

[`WorkspaceVectorConfig`](../../edgequake/crates/edgequake-storage/src/traits/workspace_vector.rs) builds a workspace namespace `default_ws_{first8-of-uuid}`, then `PgVectorStorage` qualifies it:

```text
workspace_id = 4e32a055-...
logical name (trait / logs):  eq_default_ws_4e32a055_vectors
physical table (create path): public.eq_eq_default_ws_4e32a055_vectors
chunk text KV (FTS join):     public.eq_eq_default_kv   (shared default KV)
```

- **Vectors:** table-per-workspace (isolation + per-workspace embedding dimension).
- **Graph:** one AGE graph per namespace; isolation via Node/EDGE properties `workspace_id` / `tenant_id`.
- **RLS:** `set_tenant_context` / `current_tenant_id()` (M001/M009); optional AGE RLS when `EDGEQUAKE_AGE_RLS=true` (M081).

---

## 3. PostgreSQL ER schema (relational)

```
┌──────────────────────────────────────────────────────────────────┐
│ Relational ER (fixed migrations)                                 │
│                                                                  │
│  tenants 1--* workspaces 1--* memberships *--1 users             │
│                                                                  │
│  workspaces 1--* pdf_documents --? documents                     │
│  workspaces 1--* document_mm_assets *--1 documents               │
│  workspaces 1--* document_originals --1 documents                │
│                                                                  │
│  documents 1--* chunks                                           │
│  chunks *--* chunk_entity_links *--* entity_name                 │
│  chunks *--* chunk_relation_links                                │
│                                                                  │
│  entities *--* relationships (CQRS mirror of AGE)                │
│  tasks / failed_chunks (async delivery + retry)                  │
└──────────────────────────────────────────────────────────────────┘
```

### Identity and tenancy

| Table | Purpose | Writers |
| ----- | ------- | ------- |
| `tenants`, `workspaces`, `users`, `memberships` | Multi-tenant identity | auth / workspace APIs |
| RLS helpers | `set_tenant_context`, policies on core tables | M001, M009 |

### Content sidecar

| Table | Purpose | Notes |
| ----- | ------- | ----- |
| `documents` | Relational document row (status, hashes, PDF link) | Not sole RAG text SSOT |
| `chunks` | Relational chunks + M066 `char_*` / `page_*` / `embedding_id` | Links to vector id |

### Lineage (M066)

| Table | PK / keys | Indexes |
| ----- | --------- | ------- |
| `chunk_entity_links` | `(chunk_id, entity_name, workspace_id)` | entity→chunks lookups |
| `chunk_relation_links` | chunk + source/target entity | relation provenance |

API writers: `postgres_lineage_sink`, `postgres_chunk_lineage`.

### CQRS entities (M039)

| Table | Purpose |
| ----- | ------- |
| `entities` | Analytics / FTS read model; GIN on `source_chunk_ids`, generated `tsv` |
| `relationships` | Same for edges; `sync_status` tracks AGE dual-write |

`server_config.entity_sync_mode` defaults to **disabled** — AGE remains the graph SSOT until sync is enabled.

### PDF and multimodal

| Table | Migration | Role |
| ----- | --------- | ---- |
| `pdf_documents` | M022+ | BYTEA PDF + `markdown_content`; status includes **`cancelled`** (M087) |
| `document_originals` | M082 | Non-PDF originals |
| `document_mm_assets` | M084/085 | Page/chart PNGs; stable `asset_id` + workspace RLS |

Rust: `pdf_storage_impl.rs`, `mm_asset_storage_impl.rs`, `original_storage_impl.rs`.

### Async delivery

| Object | Migration | Role |
| ------ | --------- | ---- |
| `tasks` | M002+ | Job rows; **lease_owner / lease_token / lease_expires_at** (M088) |
| `edgequake.tasks` view | M031 → M089 | Must refresh after lease columns or view hides them |
| `failed_chunks` | M021 | Extraction retry queue |

Claim indexes: `idx_tasks_claimable_pending`, `idx_tasks_stale_processing_lease`. Ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md).

### Other (pointer)

`conversations` / `messages` / `folders`, partitioned `audit_logs`, `server_config`, `workspace_metrics_history` — not on the RAG hot path.

---

## 4. KV store (document text SSOT)

### Table shape

Runtime DDL creates JSONB key-value storage (plus `_kv_stats` for O(1) counts):

```text
public.eq_eq_default_kv
  key   TEXT PRIMARY KEY
  value JSONB
  ...
```

Adapter: `adapters/postgres/kv.rs`.

### Key taxonomy (SSOT module)

All reads/writes must use [`kv_key_schema.rs`](../../edgequake/crates/edgequake-storage/src/kv_key_schema.rs) / `kv_keys::*`:

| Key pattern | Payload |
| ----------- | ------- |
| `{doc_id}-metadata` | DocumentMetadata JSON |
| `{doc_id}-chunk-{n}` | Chunk content JSON (text, offsets, tokens) |
| `{doc_id}-chunk-` | Prefix scan all chunks |
| `wsdoc:{workspace_id}:{document_id}` | Workspace document index |
| `staging:{doc_id}-…` | Admit saga staging (SPEC-026) |
| `compensation_quarantine:{doc}:{entry}` | Saga DLQ (SPEC-057) |
| `{hash}-cache` / `{hash}-kwcache` | LLM / keyword caches |

### Query hydrate

When vector metadata lacks content, query code calls `batch_fetch_chunk_contents` ([`chunk_content.rs`](../../edgequake/crates/edgequake-storage/src/chunk_content.rs)) against the **shared default KV** table — even when ANN runs on a workspace vector table (SPEC-024 2.5).

---

## 5. Apache AGE property graph

AGE stores a labeled property graph inside PostgreSQL ([AGE graphs overview](https://age.apache.org/age-manual/master/intro/graphs.html)): parent tables `_ag_label_vertex` / `_ag_label_edge`, plus **child** label tables for each label.

### Graph and labels

```text
graph name:  eq_eq_default_graph
labels:      "Node" (vertex), "EDGE" (edge)
created via: create_graph / create_vlabel / create_elabel
             (graph_lifecycle.rs — eager bootstrap)
```

### Property map (conceptual)

| Object | Important properties |
| ------ | -------------------- |
| **Node** | `node_id` (entity name), `entity_type`, `description`, `source_ids` / `source_chunk_ids`, `tenant_id`, `workspace_id`, `community_id` |
| **EDGE** | `source_id`, `target_id`, weight / keywords / description, same lineage + tenancy |

Communities are **not** separate AGE labels; `community_id` is written onto Node properties.

### Native SQL vs Cypher

| Path | Used for |
| ---- | -------- |
| **Native SQL** on `{graph}."Node"` / `"EDGE"` | Batch upsert, degrees, incident edges, workspace stats, lineage GIN probes |
| **Cypher** via `ag_catalog.cypher()` | Traversals, some deletes/clears, searches |

Hot query expansion prefers native batch helpers (O(log E) on child tables after M070/M086). Parent `_ag_label_*` tables are not the indexed hot path.

### Indexes (child tables)

| Index | Purpose | Source |
| ----- | ------- | ------ |
| `idx_node_prop_node_id_unique` | Native upsert ON CONFLICT | M074 / M083 |
| `idx_node_source_ids_gin` / `idx_edge_source_ids_gin` | Doc→entity lineage | M038 + bootstrap |
| `idx_edge_source_id` / `idx_edge_target_id` | BFS / incident edges / degrees | M086 |
| `idx_edge_start_id` / `idx_edge_end_id` | graphid navigation | M072 + ensure_indexes |
| `idx_node_workspace_id` / `idx_node_tenant_id` | Isolation filters | M078 reconcile |

### Lineage probes

[`source_lineage_sql.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/graph/helpers/source_lineage_sql.rs):

- Predicates match `source_ids` / `source_chunk_ids` JSON arrays (GIN `@>`).
- `SOURCE_CHUNK_PROBE_LIMIT = 256` caps batch probes.

Fallback tables `graph_nodes` / `graph_edges` (M013) exist if AGE is missing — production expects AGE loaded.

---

## 6. pgvector

### DDL (runtime)

From [`vector/ddl.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/vector/ddl.rs):

```sql
CREATE TABLE IF NOT EXISTS public.eq_eq_default_vectors (
    id TEXT PRIMARY KEY,
    embedding vector(D) NOT NULL,  -- or halfvec(D)
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
-- plus document_id, tenant_id, workspace_id
-- plus generated content_tsv (FTS)
```

### Dimension / halfvec policy

[`AnnIndexPolicy`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/capabilities.rs) (aligned with [pgvector HNSW limits](https://github.com/pgvector/pgvector#hnsw)):

| Dimension | Column | HNSW |
| --------- | ------ | ---- |
| ≤ 2000 | `vector` or `halfvec` per `EDGEQUAKE_VECTOR_STORAGE` | Yes |
| 2001–4000 | promote to **`halfvec`** | Yes |
| > 4000 | configured type | **No ANN** (seq scan) |

Env: `EDGEQUAKE_VECTOR_STORAGE=full|halfvec` (default `full`). Marker M080 + bootstrap reconcile.

### HNSW

- Opclass: `vector_cosine_ops` or `halfvec_cosine_ops`
- Defaults: `m = 16`, `ef_construction = 32` (overridable via env / config)
- Index name pattern: `eq_{prefix}_vectors_embedding_idx`
- Fail-closed: ANN DDL errors are not swallowed (SPEC-046)
- Search tuning: `SET LOCAL hnsw.ef_search = …`; optional `hnsw.iterative_scan` when pgvector ≥ 0.8.0 ([`search_tuning.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/vector/search_tuning.rs))

### Query shape

```sql
SELECT id, metadata,
       (1 - (embedding <=> $1::vector))::float4 AS score
FROM public.eq_eq_default_vectors
WHERE ... MetadataFilter / id = ANY(...)
ORDER BY embedding <=> $1::vector
LIMIT $k;
```

Also: btree on `document_id`, `(tenant_id, workspace_id)`.

---

## 7. Text search (FTS)

### Chunk sparse retrieval (vectors + KV)

[`fts.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/vector/fts.rs):

```sql
SELECT v.id, v.metadata,
       ts_rank_cd(
         coalesce(v.content_tsv,
                  to_tsvector('english', coalesce(v.metadata->>'content',
                                                 k.value->>'content', ''))),
         websearch_to_tsquery('english', $1)
       )::float4 AS score
FROM public.eq_eq_default_ws_XXXXXXXX_vectors v
LEFT JOIN public.eq_eq_default_kv k ON k.key = v.id
WHERE coalesce(...) @@ websearch_to_tsquery('english', $1)
ORDER BY score DESC
LIMIT $k;
```

- GIN on generated `content_tsv` (M045 / ensure_content_fts; duplicate dropped M069).
- Workspace vector tables hold embeddings; **chunk text SSOT remains default KV**.

### Entity CQRS FTS

`entities.tsv` GENERATED from name/type/description + GIN (M039) — analytics / search over the relational CQRS mirror, not the AGE hot path.

### Historical AGE name FTS

M015 added tsvector/trgm on graph name fields; after M070 index consolidation, prefer child-table indexes and entity vector + AGE expand for RAG.

---

## 8. How information is queried

Pipeline: prepare (keywords + embeddings) → retrieve by mode → finalize (truncate / LLM). Entry: `edgequake-query` `query_pipeline.rs`.

### Mode × store matrix

| Mode | Vector | AGE | KV / FTS |
| ---- | ------ | --- | -------- |
| **Naive** | Chunk ANN (`query_filtered` / modality preference) | — | Hydrate; optional FTS fuse |
| **Local** | Entity vectors (`filter_by_type(Entity)`) → chunk re-score by ids | `get_nodes_batch`, degrees, neighborhood expand | Hydrate |
| **Global** | Relationship (+ optional `CommunityReport`) vectors | Same expand path | Hydrate |
| **Hybrid** | Parallel Local / Global / Naive (intent-gated) | Via Local/Global | Via Naive + hydrate |
| **Mix** (default) | Same three arms → weighted / RRF merge | Via Local/Global | Via Naive + hydrate |
| **Bypass** | — | — | — |

Code: `modes/{naive,local,global,hybrid,mix}.rs`, `chunk_retrieval.rs`, `chunk_hydration.rs`.

### Local / Mix bridge (ASCII)

```
┌──────────────────────────────────────────────────────────────┐
│ Local / Mix retrieval bridge                                 │
│                                                              │
│  Query embedding                                             │
│       |                                                      │
│       v                                                      │
│  pgvector ANN (entity or relationship vectors)               │
│       |                                                      │
│       v                                                      │
│  AGE expand (batch nodes / edges / BFS or PPR)               │
│       |                                                      │
│       v                                                      │
│  Collect chunk ids from source_ids / source_chunk_ids        │
│       |                                                      │
│       v                                                      │
│  pgvector re-score with filter_ids (+ workspace filter)      │
│       |                                                      │
│       v                                                      │
│  KV hydrate chunk text (if metadata empty)                   │
│       |                                                      │
│       v                                                      │
│  Context --> LLM (unless context_only)                       │
└──────────────────────────────────────────────────────────────┘
```

---

## 9. Write path summary (ingest)

```
┌──────────────────────────────────────────────────────────────┐
│ Ingest write path (simplified)                               │
│                                                              │
│  HTTP admit --> tasks Pending (claim/lease)                  │
│       |                                                      │
│       +--> PdfProcessing: pdf_documents + markdown           │
│       |         |                                            │
│       |         v markdown barrier                           │
│       +--> Insert:                                           │
│              KV metadata + chunks                            │
│              AGE Node / EDGE upsert                          │
│              eq_*_vectors upsert                             │
│              chunk_*_links / mm_assets (when applicable)     │
└──────────────────────────────────────────────────────────────┘
```

Full sequence, cancel, and convert≠ingest: [Data Flow](../architecture/data-flow.md), [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md).

---

## 10. Migration & index map

| Migration | Concern | Runtime note |
| --------- | ------- | ------------ |
| 001 / 008 / 009 | Extensions, tenancy, RLS | Base |
| 013–015 | AGE helpers, graph indexes, early FTS | Child indexes later |
| 022 / 084–087 | PDF + mm-assets + Cancelled | pdf_storage / mm_asset |
| 028–029 / 071 | Vector columns + HNSW optimize | DDL + bootstrap |
| 038 | `source_ids` GIN on AGE | `migration_bootstrap` M038 |
| 039 | CQRS entities + `tsv` | Optional dual-write |
| 045 / 069 | `content_tsv` FTS | ensure_content_fts |
| 066 | chunk lineage tables | lineage sink |
| 070 / 074 / 083 / 086 | AGE child indexes, unique node_id, BFS | graph_lifecycle ensure_indexes |
| 080 | halfvec promotion | capabilities + apply |
| 081 | AGE RLS (optional) | `EDGEQUAKE_AGE_RLS` |
| 088 / 089 | Task leases + view refresh | Must refresh `edgequake.tasks` |

Marker migrations often pair with `migrations/support/*/apply.sql` and API bootstrap reconcile.

---

## 11. Operator debugging cookbook

Replace graph/table names with your namespace. Examples assume `namespace=default`.

### Inventory

```sql
-- KV document metadata keys
SELECT count(*) FROM public.eq_eq_default_kv
WHERE key LIKE '%-metadata';

-- Vector rows
SELECT count(*) FROM public.eq_eq_default_vectors;

-- Workspace vector tables
SELECT tablename FROM pg_tables
WHERE schemaname = 'public'
  AND tablename LIKE 'eq_eq_default_ws_%_vectors';

-- AGE graphs
SELECT name FROM ag_catalog.ag_graph ORDER BY name;

-- Approximate Node count (child table; adjust schema)
SELECT reltuples::bigint
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname = 'eq_eq_default_graph' AND c.relname = 'Node';
```

### Cosine ANN (smoke)

```sql
-- Requires a real query vector of matching dimension
EXPLAIN (ANALYZE, BUFFERS)
SELECT id, 1 - (embedding <=> '[0,0,...]'::vector) AS score
FROM public.eq_eq_default_vectors
ORDER BY embedding <=> '[0,0,...]'::vector
LIMIT 10;
```

### FTS

```sql
SELECT v.id,
       ts_rank_cd(v.content_tsv, websearch_to_tsquery('english', 'your query')) AS score
FROM public.eq_eq_default_vectors v
WHERE v.content_tsv @@ websearch_to_tsquery('english', 'your query')
ORDER BY score DESC
LIMIT 10;
```

### Lineage GIN probe (AGE child table)

```sql
-- Example: nodes citing a document prefix in source_ids
SELECT id, properties
FROM "eq_eq_default_graph"."Node"
WHERE properties->'source_ids' @> '["your-doc-id-chunk-0"]'::jsonb
LIMIT 20;
```

### Task leases

```sql
SELECT id, status, task_type, lease_owner, lease_expires_at, created_at
FROM edgequake.tasks
WHERE status IN ('pending', 'processing')
ORDER BY created_at
LIMIT 50;
```

If lease columns are missing from the view, apply M089 / refresh the view (see release notes for M031 class drift).

### Queue pressure (API)

```bash
curl -s http://localhost:8080/api/v1/pipeline/queue-metrics | jq
curl -s http://localhost:8080/ready
```

---

## Code map (quick)

| Concern | Path under `edgequake/crates/` |
| ------- | ------------------------------ |
| Naming | `edgequake-storage/.../postgres/config.rs` |
| KV keys | `edgequake-storage/src/kv_key_schema.rs` |
| KV adapter | `edgequake-storage/.../postgres/kv.rs` |
| Vectors | `edgequake-storage/.../postgres/vector/` |
| Workspace vectors | `edgequake-storage/.../postgres/workspace_vector.rs` |
| AGE | `edgequake-storage/.../postgres/graph/` |
| Lineage SQL | `.../graph/helpers/source_lineage_sql.rs` |
| Query modes | `edgequake-query/src/engine_impl/modes/` |
| Migrations | `edgequake/migrations/` |

Optional live dump for a concrete database: `specs/044-upgrate-issue-study/edgequakeSchema.sql`.
