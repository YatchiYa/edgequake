# 05 — Target Specification

> The target data layer, specified as contracts (schema, ports, state machine), each traceable to a law ([02](02-first-principles.md)) and a decision ([README LD-01..14](README.md#locked-decisions)). DDL below is implementation-oriented pseudocode pending migration review; it fixes *semantics*, not final syntax.
>
> **Progress after Waves A–D:** typed replacements for retired KV families are **largely achieved** in-tree (migrations 106–125). The **BEFORE** ASCII still describes pin v0.22.0. The **AFTER** topology for embeddings (`chunk_embeddings`, UUID FKs, no runtime vector DDL) remains a **W3 target** — schema 108 exists but the live fleet is still `eq_*_vectors`. See [16-post-cutover-assessment.md](16-post-cutover-assessment.md).

## Topology: before → after (ASCII)

```ascii
 BEFORE (v0.22.0): 4 representations, 3 writers, 2 identities, 0 shared commits
 ┌────────────────────────────────────────────────────────────────────────────────┐
 │                         INGESTION (3 independent commits)                      │
 │   ┌──────────────┐   ┌────────────────────┐   ┌─────────────────┐              │
 │   │ eq_*_kv      │   │ eq_*_vectors       │   │ AGE graph       │              │
 │   │ text (SSOT)  │   │ halfvec+metadata   │   │ Node/EDGE (SSOT)│              │
 │   │ key={d}-chunk│   │ id TEXT PK         │   │ cypher MERGE    │              │
 │   │ -{n} (string)│   │ content_ref only   │   │                 │              │
 │   └──────┬───────┘   └─────────┬──────────┘   └────────┬────────┘              │
 │        no FK possible — identities: string key vs TEXT PK vs uuid (unminted)   │
 │   ┌──────┴─────────────────────────────────────────────┴────────┐              │
 │   │ chunks: DECLARED (uuid, content NOT NULL), WRITTEN BY NOBODY │◀─ stats read│
 │   │ entities/relationships: CQRS read model, often disabled      │  (returns 0)│
 │   └─────────────────────────────────────────────────────────────┘              │
 └────────────────────────────────────────────────────────────────────────────────┘

 AFTER (target): 1 identity, 1 authority per fact, 1 commit + fenced projections
 ┌────────────────────────────────────────────────────────────────────────────────┐
 │  ONE BOUNDED RELATIONAL COMMIT            ASYNC PROJECTIONS (idempotent)       │
 │  ┌──────────────────────────────────┐     ┌──────────────────────────────────┐ │
 │  │ chunks (uuid PK = uuidv7)        │     │ chunk_embeddings (model,chunk)   │ │
 │  │  content + lineage + generated   │────▶│  halfvec, FK ON DELETE CASCADE   │ │
 │  │  content_tsv (STORED)            │     │ AGE graph (traversal authority)  │ │
 │  │ chunk_serving_state (machine)    │     │ entities/relationships read model│ │
 │  │ outbox_events (same TX)          │     │  (optional, labeled freshness)   │ │
 │  └───────────────┬──────────────────┘     └───────────────┬──────────────────┘ │
 │                  │ FK + cascade + forced RLS              │ serving fence:     │
 │                  ▼                                        ▼ state='ready' only │
 │           documents (lifecycle)                    query path filters          │
 └────────────────────────────────────────────────────────────────────────────────┘
```

## Schema contract (migration-owned; LD-03)

### `chunks` — consolidated, single definition (fixes F-091-02/03/13)

```sql
CREATE TABLE chunks (
    id            uuid PRIMARY KEY DEFAULT uuidv7(),        -- PG18: time-ordered, index-friendly (LAW-D2)
    document_id   uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    tenant_id     uuid     REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    workspace_id  uuid     REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    chunk_index   integer NOT NULL,
    content       text NOT NULL,                            -- LAW-D6 authority; TOASTed out of line
    content_tsv   tsvector GENERATED ALWAYS AS (to_tsvector('english', content)) STORED,
                  -- PG18: virtual is now the default; STORED must be explicit (LAW-D6)
    start_offset  integer, end_offset integer, token_count integer,
    metadata      jsonb NOT NULL DEFAULT '{}',
    created_at    timestamptz NOT NULL DEFAULT now(),
    UNIQUE (document_id, chunk_index)                       -- idempotent writer + backfill key
);
-- RLS forced (as migration 096 today); GIN on content_tsv created per size policy (W1 § backfill rules)
```

Notes: the legacy KV key is *derivable* (`{document_id}-chunk-{chunk_index}`), so no mapping column is stored (DRY); dual-read fallback computes it. `docker/init.sql` stops defining `chunks`; it consumes the migration-owned definition or is generated from it (F-091-13).

### `embedding_models` + `chunk_embeddings` — typed vector storage (fixes F-091-01/03/04)

```sql
CREATE TABLE embedding_models (
    id          uuid PRIMARY KEY DEFAULT uuidv7(),
    name        text NOT NULL,                              -- e.g. 'text-embedding-3-small'
    dimensions  integer NOT NULL,
    metric      text NOT NULL DEFAULT 'cosine' CHECK (metric = 'cosine'),  -- LAW: enforced single metric
    UNIQUE (name, dimensions)
);

CREATE TABLE chunk_embeddings (
    model_id      uuid NOT NULL REFERENCES embedding_models(id),
    chunk_id      uuid NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,   -- LAW-D1: presence by FK
    workspace_id  uuid NOT NULL,                            -- typed routing (RLS + filters)
    embedding     halfvec NOT NULL,                         -- unconstrained dims, checked below
    dimensions    integer NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (model_id, chunk_id),
    CHECK (vector_dims(embedding) = dimensions)
);
-- One partial expression HNSW per supported model (pgvector mixed-dimension pattern):
-- CREATE INDEX ... ON chunk_embeddings USING hnsw ((embedding::halfvec(1536)) halfvec_cosine_ops)
--   WHERE model_id = '<uuid>';                           -- few distinct values => partial indexes
```

pgvector guidance honored: halfvec ceiling 4,000 dims; "filtering by few distinct values → partial indexing; many → partitioning" — model scoping uses partial indexes now, list partitioning only via Wave-5 measurement gate (LD-10).

### `chunk_serving_state` — the fence (LAW-D1/D3, LD-09)

```sql
CREATE TABLE chunk_serving_state (
    chunk_id      uuid PRIMARY KEY REFERENCES chunks(id) ON DELETE CASCADE,
    state         text NOT NULL,
    attempt_count integer NOT NULL DEFAULT 0,
    last_error    jsonb,
    updated_at    timestamptz NOT NULL DEFAULT now(),
    CHECK (state IN ('declared','embedded','graphed','ready','quarantined','deleting'))
);
-- Serving rule (fail-closed): query-visible ⇔ chunks row exists ∧ state='ready'.
```

### Typed replacements for KV key families (fixes F-091-10; W2)

| KV key family (today, `kv_key_schema.rs`) | Typed authority (target) | Key constraint |
| --- | --- | --- |
| `{doc_id}-chunk-{n}` | `chunks` | `UNIQUE(document_id, chunk_index)` |
| `{doc_id}-metadata` | `documents` | existing PK + workspace/status indexes |
| `wsdoc:{workspace}:{doc}` | `documents.workspace_id` | `INDEX (workspace_id, id)` |
| `staging:hash:{workspace}:...` | `ingestion_dedup` | `UNIQUE(workspace_id, content_hash, pipeline_version)` |
| `compensation_quarantine:...` | `compensation_quarantine` (typed DLQ) | status + `next_attempt_at` indexes |

## Serving state machine (ASCII)

```ascii
   committed in one bounded TX (chunk + text + dedup + outbox)
                          │
                          ▼
                 ┌─────────────────┐   embedding worker confirms (idempotent)
                 │    declared     │───────────────────────────┐
                 └────────▲────────┘                           ▼
                          │                            ┌─────────────┐   graph worker confirms
          reconciler      │ retry budget exhausted     │  embedded   │───────────────┐
          retry (bounded, │                            └──────┬──────┘               ▼
          SLO M-4.1)      │                                  │ retry budget  ┌────────────┐
                          │                                  │ exhausted     │  graphed   │
                 ┌────────┴────────┐                         │               └─────┬──────┘
                 │  quarantined    │◀────────────────────────┘                     │
                 └─────────────────┘                                               │ fence opens
                                                                                   ▼
                                                                            ┌────────────┐
                                                                            │   ready    │
                                                                            └─────┬──────┘
                                                                                  │ deletion requested
                                                                                  ▼
                                                                            ┌────────────┐   cascade + absence
                                                                            │  deleting  │──────────▶ (gone)
                                                                            └────────────┘   both verified
```

## Domain ports (SOLID; LD-05)

```rust
// Batch-first, storage-agnostic. No SQL, relation names, halfvec, Cypher,
// or key strings cross this boundary (LAW-D7; enforcement: CI dependency lint).
trait ChunkRepository {
    async fn insert_batch(&self, tx: &mut UnitOfWork, chunks: &[Chunk]) -> Result<InsertReport>;
    async fn load_texts(&self, ids: &[ChunkId]) -> Result<Vec<ChunkText>>;
    async fn scan_from(&self, cursor: Option<ChunkCursor>, limit: u32) -> Result<Page<Chunk>>;
    async fn delete_for_document(&self, tx: &mut UnitOfWork, id: DocumentId) -> Result<u64>;
}

trait EmbeddingIndex {
    fn capabilities(&self) -> EmbeddingCapabilities;   // filters, metric, rerank, recall reporting
    async fn upsert_batch(&self, model: ModelId, rows: &[EmbeddingRow]) -> Result<UpsertReport>;
    async fn search(&self, req: &VectorQuery) -> Result<Vec<ScoredChunk>>;
    async fn delete_for_workspace(&self, ws: WorkspaceId) -> Result<u64>;
}
```

Full port set: `DocumentRepository`, `ChunkRepository`, `FullTextIndex`, `EmbeddingIndex`, `GraphProjection`, `BlobStore`, `MigrationLedger`. `UnitOfWork` carries a declared **atomicity capability**: the PostgreSQL adapter commits chunk+text+state+outbox in one TX; a split-provider deployment reports non-atomic and the fence (not hope) preserves correctness — visibility degrades, integrity never (LD-09).

### Boundary rules (enforced by lint, not review)

| Concern | Inside adapter | May cross boundary |
| --- | --- | --- |
| Query language | SQL, Cypher, filter syntax | typed requests with explicit bounds |
| Schema | relation/column names, partitions, views | domain entities + typed IDs |
| Vector mechanics | halfvec, HNSW params, ef_search, opclasses | model id, dims, metric, recall target, scores |
| Keys | derived string encodings | `ChunkId/DocumentId/WorkspaceId/ModelId` |
| Transactions | isolation, savepoints, lock timeouts | `UnitOfWork` + atomicity capability |
| Errors | driver codes | closed domain-error set with retryability |
| Tuning | batch sizes, pools, index policy | nothing |

### Conformance suite (LSP, proven)

Runs against every registered adapter in CI (PostgreSQL + in-memory). Covers: idempotency under retry, partial-failure behavior, cursor stability under concurrent writes, deletion completeness, filter semantics, ordering, recall reporting where declared — **plus a cost budget: no port operation may require a round trip per row** (LAW-D7). An adapter that fails is not shipped.

## SSOT map: before → after

| Fact | Before (de facto) | After (declared + enforced) |
| --- | --- | --- |
| Chunk text | `eq_*_kv` JSONB (unreachable by constraints) | `chunks.content` (NOT NULL, TOAST, FK-reachable) |
| Chunk identity | `{doc}-chunk-{n}` string + unminted uuid | `chunks.id uuid` (uuidv7) everywhere |
| Full-text index | writable `content_tsv` via cross-store backfill (M091) | generated column over the authoritative value |
| Embeddings | `eq_*_vectors` (runtime DDL) | `chunk_embeddings` (migration-owned, FK) |
| Counts | 3 disagreeing mechanisms | projections of `chunks` / `chunk_serving_state` |
| Readiness | none (presence probed, M093) | `chunk_serving_state.state = 'ready'` |
| Schema | migrations + runtime DDL + init.sql (3 narrators) | `migrations/` only, digest-verified |
| HNSW policy | 32 / 64 / 128 in three files (F-091-14) | one policy module, one benchmarked value |
| Deletion | cross-store reconciliation + quarantine | FK cascade + verified projection sweep |
| Migration progress | runbooks + human observation | job/batch ledgers → CLI/API/SQL/metrics ([07](07-migration-engine.md)) |

## Explicit non-goals

- Replacing AGE or moving traversal to recursive SQL (LD-04).
- Changing the query API surface during Waves 0–3 (flags only).
- Partitioning or quantization before a measured threshold breach (LD-10).
- Provider-agnostic *spine*: the relational spine is a deliberate PostgreSQL commitment; portability is scoped to vector/lexical/graph/blob engines via ports (LD-05).
