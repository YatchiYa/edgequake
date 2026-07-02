# EdgeQuake — Complete Architecture & Replication Reference

> **Scope:** This document describes the EdgeQuake *core server* (the Rust workspace in [`edgequake/`](edgequake/)) end-to-end: architecture, services, data model, the ingestion algorithm, the retrieval algorithm, the API, auth, background jobs, observability, and configuration. It is written so that a competent engineer can **rebuild the system from scratch**.
>
> **Version documented:** 0.13.0 · **Language:** Rust 2021 (rustc ≥ 1.95) · **License:** Apache-2.0

---

## Table of Contents

1. [What EdgeQuake Is](#1-what-edgequake-is)
2. [Technology Stack & External Services](#2-technology-stack--external-services)
3. [Crate Topology (the 11 crates)](#3-crate-topology-the-11-crates)
4. [Deployment Topology](#4-deployment-topology)
5. [Data Model & PostgreSQL Schema](#5-data-model--postgresql-schema)
6. [Storage Layer](#6-storage-layer)
7. [LLM & Embedding Layer (`models.toml`)](#7-llm--embedding-layer-modelstoml)
8. [Ingestion Pipeline — Document → Knowledge Graph](#8-ingestion-pipeline--document--knowledge-graph)
9. [Query / Retrieval Engine](#9-query--retrieval-engine)
10. [HTTP API Surface](#10-http-api-surface)
11. [Authentication, RBAC & Multi-Tenancy](#11-authentication-rbac--multi-tenancy)
12. [Background Task & Worker System](#12-background-task--worker-system)
13. [Rate Limiting](#13-rate-limiting)
14. [Observability](#14-observability)
15. [Configuration Reference](#15-configuration-reference)
16. [Server Startup Sequence](#16-server-startup-sequence)
17. [Step-by-Step Replication Guide](#17-step-by-step-replication-guide)

---

## 1. What EdgeQuake Is

EdgeQuake is a **GraphRAG** (knowledge-graph-augmented Retrieval-Augmented Generation) server, heavily inspired by **LightRAG**. It ingests documents (text, Markdown, PDF), extracts a **knowledge graph** of entities + relationships using an LLM, embeds everything into vectors, and answers questions by combining:

- **Dense vector similarity search** (pgvector / HNSW)
- **Knowledge-graph traversal** (Apache AGE, Cypher, multi-hop BFS)
- **Sparse full-text retrieval** (PostgreSQL FTS / BM25-style fusion)
- **LLM keyword extraction & intent routing** to pick a retrieval strategy per query

It is **multi-tenant** (tenant → workspace → user), **API-first** (Axum REST + OpenAPI/Swagger + an MCP server), and ships with an async job system for large-document ingestion.

The whole persistence layer is **a single PostgreSQL instance** carrying three roles: relational store, vector store (pgvector), and graph store (Apache AGE). There is no separate vector DB or graph DB.

**Core value chain:**

```
Upload → PDF→Markdown → Chunk → LLM entity/relation extraction → (gleaning) →
Merge/dedup into graph → Embed (chunks+entities+rels) → Persist (KV+vector+AGE+CQRS)
                                                                     │
Query → keywords+intent → mode select → vector+graph+BM25 retrieve → rerank →
token-budget truncate → build prompt → LLM answer (stream/vision optional)
```

---

## 2. Technology Stack & External Services

| Concern | Choice | Notes |
|---|---|---|
| Language / runtime | **Rust 2021**, Tokio 1.48 async | `lto="thin"`, `codegen-units=1`, `opt-level=3` release |
| HTTP framework | **Axum 0.8** (+ tower, tower-http) | CORS, gzip, tracing middleware |
| OpenAPI | **utoipa 5.4** + `utoipa-swagger-ui 8.0` | `/swagger-ui` |
| Database driver | **sqlx 0.8** (`runtime-tokio`, `tls-rustls`) | compile-checked queries (`.sqlx/`) |
| Relational + Vector + Graph DB | **PostgreSQL 14+** with **pgvector** and **Apache AGE** | one DB, three roles |
| LLM abstraction | external crate **`edgequake-llm` 0.6.26** | 11 providers behind `LLMProvider`/`EmbeddingProvider` traits |
| Tokenization | `tiktoken-rs 0.6` | token counting |
| PDF → Markdown | **`edgequake-pdf2md` 0.9.2** (vision) + **`edgeparse-core` 0.2.5** (heuristic) | two backends |
| Auth | `argon2` (password hash), `jsonwebtoken 9.3` (JWT HS256), OIDC | RBAC + JWT + API keys |
| Metrics | `metrics` + `metrics-exporter-prometheus` | `/metrics` |
| Tracing | `tracing` + `tracing-subscriber` (+ optional OpenTelemetry via `otel` feature) | W3C Trace Context |
| Frontend (separate image) | **Next.js** web UI | not part of the core crate |

**External runtime services required:**

- **PostgreSQL** with `vector` + `age` + `uuid-ossp` + `pg_trgm` extensions (mandatory; AGE degrades to relational-fallback tables if absent).
- **An LLM provider** (any of: OpenAI, Anthropic, Mistral, Gemini, xAI, MiniMax, OpenRouter, Azure OpenAI, Vertex AI, **Ollama** (local), **LM Studio** (local)). The quickstart defaults to **Ollama** on the host.

Cargo feature flags on the top crate: `default = ["postgres","vision"]`; `otel`, `postgres`, `vision`.

---

## 3. Crate Topology (the 11 crates)

The binary is [`edgequake/src/main.rs`](edgequake/src/main.rs); everything else is a workspace crate under [`edgequake/crates/`](edgequake/crates/).

```
edgequake (bin)                    ← startup, worker pool, orphan recovery
├── edgequake-core                 ← domain types, orchestrator, config, multi-tenancy, workspace service
├── edgequake-storage              ← storage TRAITS + Postgres (pgvector/AGE) & memory adapters, RLS
├── edgequake-pipeline             ← ingestion: chunk → extract → glean → merge → embed → persist
├── edgequake-query                ← retrieval engine: modes, keywords, fusion, truncation, prompts
├── edgequake-api                  ← Axum server, handlers, streaming, MCP server, provider catalog
├── edgequake-auth                 ← JWT, API keys, argon2, RBAC, OIDC, tenant context
├── edgequake-tasks                ← async task queue, worker pool, circuit breaker, retries
├── edgequake-rate-limiter         ← token-bucket rate limiting middleware
├── edgequake-observability        ← tracing, Prometheus metrics, correlation IDs, OTEL
├── edgequake-pdf                  ← PDF→Markdown backends (vision LLM + edgeparse), inline images
├── edgequake-audit                ← audit logging / compliance
└── (external) edgequake-llm 0.6.26 ← provider abstraction (not in-tree)
```

**Dependency direction:** `api` depends on `query`, `pipeline`, `storage`, `tasks`, `auth`, `observability`, `core`. `pipeline` and `query` depend on `core` + `storage` + `edgequake-llm`. `storage` defines the trait contracts everything else programs against.

---

## 4. Deployment Topology

Reference stack is [`docker-compose.quickstart.yml`](docker-compose.quickstart.yml) — three containers on a bridge network, all pulled prebuilt from GHCR:

| Service | Image | Port | Role |
|---|---|---|---|
| `postgres` | `ghcr.io/…/edgequake-postgres` | 5432 | PostgreSQL **with pgvector + Apache AGE preinstalled** (`shm_size: 256m`) |
| `api` | `ghcr.io/…/edgequake` | 8080 | The Rust server (this document) |
| `frontend` | `ghcr.io/…/edgequake-frontend` | 3000 | Next.js Web UI |

Key wiring:
- API ↔ DB via `DATABASE_URL=postgres://edgequake:…@postgres:5432/edgequake`.
- LLM provider default is **Ollama on the host**, reached via `host.docker.internal:11434` (Linux uses `extra_hosts: host.docker.internal:host-gateway`).
- Vision provider/model fall back to the main LLM provider/model via nested `${VISION:-${LLM:-default}}` substitution.
- Health endpoints: API `/health`, DB `pg_isready`, frontend HTTP spider.

`make stack` / `make stack-down` wrap compose. Access: Web UI `:3000`, API `:8080`, Swagger `:8080/swagger-ui`.

---

## 5. Data Model & PostgreSQL Schema

Migrations live in [`edgequake/migrations/`](edgequake/migrations/) (`001`…`077`, applied via sqlx at startup; `checksums.lock` guards drift). Migration `001_init_database.sql` builds the whole relational core; later migrations add graph, FTS, materialized vector columns, HNSW tuning, CQRS entity read-model, and lineage.

### 5.1 Extensions
```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "vector";        -- pgvector
CREATE EXTENSION IF NOT EXISTS "age" CASCADE;   -- Apache AGE (optional, degrades gracefully)
CREATE EXTENSION IF NOT EXISTS pg_trgm;         -- trigram fuzzy search (mig 015)
```
`search_path` is pinned to `public` (plus `ag_catalog` when AGE present).

### 5.2 Core relational tables

**Multi-tenancy spine:** `tenants` → `workspaces` → `users` → `memberships` (all UUID PKs, cascade delete, unique slugs per tenant, role checks).

**`documents`** — one row per ingested doc. Notable columns: `content`, `content_hash`, `status ∈ {pending,processing,indexed,failed}`, `track_id`, `chunk_count`, `entity_count`, `relationship_count`, `processing_time_ms`, `tenant_id`, `workspace_id`.

**`chunks`** — `document_id`, `content`, `chunk_index` (unique per doc), `start_offset/end_offset`, `token_count`, `embedding vector(1536)` *(vestigial — see note)*, plus lineage columns added by mig 066: `char_start/char_end/page_start/page_end/embedding_id`.

**`entities`** — `name`, `entity_type`, `description`, `source_ids UUID[]`, `is_manual` + manual-edit audit columns, `embedding vector(1536)` *(vestigial)*. Unique on `(tenant_id, workspace_id, name)`.

**`relationships`** — `source_id`/`target_id` → entities, `relation_type`, `description`, `weight REAL`, `keywords TEXT[]`, `source_chunk_ids UUID[]`. Unique on `(tenant, workspace, source, target, relation_type)`.

**`tasks`** — async job rows: `track_id`, `task_type`, `status`, `priority`, `payload JSONB`, `result JSONB`, `retry_count/max_retries`, `scheduled/started/completed_at`, plus `consecutive_timeout_failures` + `circuit_breaker_tripped` (mig 020) and `tenant_id/workspace_id` (mig 019).

**Conversations:** `folders`, `conversations` (`mode ∈ {local,global,hybrid,naive,mix}`, `share_id`), `messages` (`role ∈ {user,assistant,system}`, `context JSONB`, token/timing stats). Triggers bump `conversations.updated_at` on new message.

**`audit_logs`** — `PARTITION BY RANGE(timestamp)`, 12 monthly partitions pre-created, enum-typed `event_type/result/severity`, retention + archive columns.

> **Critical note on where data actually lives (mig 039 "CQRS"):** The `chunks.embedding` and `entities.embedding` columns are **always NULL in production** — real embeddings live in **per-namespace/per-workspace `eq_*_vectors` tables** (pgvector), and the real graph lives in **Apache AGE**. The `entities`/`relationships` relational tables are a **CQRS read model** used for analytics/FTS/JOINs and dual-written when enabled (`entity_sync_mode` in `server_config`). Treat AGE + `eq_*_vectors` as the source of truth for retrieval; the relational tables as a queryable projection.

### 5.3 Vector tables (pgvector) — created by application code
Per namespace/workspace: `public.eq_{prefix}_vectors` and a maintained `eq_{prefix}_vectors_stats` counter table.
```sql
CREATE TABLE public.eq_{prefix}_vectors (
  id           TEXT PRIMARY KEY,
  embedding    vector({dim}) NOT NULL,          -- dim = embedding model's dimension
  metadata     JSONB DEFAULT '{}',
  document_id  TEXT,  tenant_id TEXT,  workspace_id TEXT,   -- materialized (mig 028) for pre-filtering
  content_tsv  TSVECTOR GENERATED ... STORED,   -- FTS (mig 045)
  created_at   TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX ..._embedding_idx ON ... USING hnsw (embedding vector_cosine_ops)
       WITH (m = 16, ef_construction = 32);     -- mig 071 lowered ef_construction 64→32
```
**Distance metric:** cosine (`<=>`); score returned as `1 - distance`. **Dimension is per embedding model** (1536 for `text-embedding-3-small`, 768 for nomic/embeddinggemma, 1024 for mistral-embed, 3072 for `text-embedding-3-large`). Different workspaces can use different dimensions → separate tables (see [`WorkspaceVectorRegistry`](#63-workspacevectorregistry)).

### 5.4 Graph storage (Apache AGE)
Migration `013_add_age_graph.sql` sets up AGE and, when AGE is unavailable, **fallback relational tables** `graph_nodes` / `graph_edges` (with `upsert_graph_node/edge`, `get_node_neighbors` plpgsql helpers). With AGE present, nodes/edges are stored in AGE label tables (`{graph}._ag_label_vertex`, `_ag_label_edge`) and queried with Cypher via `ag_catalog.cypher(...)`. GIN/trigram indexes are added on the `node_id` property for label search & fuzzy autocomplete (mig 015). Later migrations add native SQL upsert helpers and index consolidation (066/067/070/074/075).

### 5.5 Row-Level Security (RLS) — tenant isolation in the database
All tenant-scoped tables (`documents, chunks, entities, relationships, tasks, conversations, messages, folders, audit_logs, graph_nodes, graph_edges`) have `ENABLE ROW LEVEL SECURITY` + a policy of the form:
```sql
USING ( tenant_id IS NULL
        OR (tenant_id = current_tenant_id()
            AND (current_workspace_id() IS NULL OR workspace_id = current_workspace_id())) )
```
Context is set per-connection by plpgsql functions:
```sql
set_tenant_context(tenant_id, workspace_id, user_id)  -- sets app.current_* GUCs (transaction-local)
current_tenant_id() / current_workspace_id() / current_user_id() / clear_tenant_context()
```
Conversations/messages/folders are additionally **user-scoped** (`user_id = current_user_id()` or a public `share_id`).

---

## 6. Storage Layer

Crate: [`edgequake-storage`](edgequake/crates/edgequake-storage/). It defines **trait contracts** (`src/traits/`) and two adapter families (`postgres`, `memory`). Everything upstream programs against the traits (dependency inversion).

### 6.1 Trait contracts (`src/traits/`)
- **`KVStorage`** — flexible JSONB key-value (documents, chunk text, checkpoints). Methods: `get_by_id/get_by_ids`, `upsert`, `delete`, `keys/keys_with_prefix/suffix/like`, `count` (**O(1) via stats table**), `ping` (**O(1), not COUNT(*)**), and **`transition_if_status(key, expected, new)`** — atomic CAS to prevent TOCTOU races on document state.
- **`VectorStorage`** — `query(embedding, top_k, filter_ids)`, `query_filtered(..., MetadataFilter)`, `text_search_filtered(...)` (native FTS), `upsert`, `delete/delete_entity/…`, `clear_workspace`, `dimension()`, `supports_native_text_search()`. Result = `{id, score = 1−distance, metadata}`.
  - `MetadataFilter { document_ids, tenant_id, workspace_id, vector_type }` where `vector_type ∈ {chunk, entity, relationship}` — pushed into SQL `WHERE` (SPEC-007 tiered pre-filter).
- **`GraphStorage`** — composite of `GraphStorageReadOps` (`get_node`, `node_degree(_batch)`, `get_node_edges`, `get_incident_edges_batch`, `get_knowledge_graph(start, depth, max_nodes, tenant, ws)`, `search_labels/nodes`, `get_neighbors`), `GraphStorageMutateOps` (`upsert_node(s_batch)`, `upsert_edge(s_batch)`, scoped deletes, `clear_workspace`), `GraphScanOps` (paged filtered listing, source-prefix lookups, community filters), and `GraphStorageAnalyticsOps` (`node_count(_fast/_by_workspace)`, etc.). **Batch upserts and per-workspace counts are required (no default impls)** to force honest, single-round-trip, workspace-scoped behavior.
- **`WorkspaceVectorRegistry`** — manages per-workspace vector tables of independent dimension.

### 6.2 PostgreSQL adapter (`src/adapters/postgres/`)
- **Pool** (`connection.rs`): lazy `PgPoolOptions`, default `max_connections≈32` (sized ≥ 2× pipeline concurrency), `min=1`, `acquire_timeout 30s`, `idle 600s`, `after_connect: SET search_path=public`. On init creates `vector` + `age` extensions.
- **Vector (`vector/`)**: HNSW (`vector_cosine_ops`, `m=16`, `ef_construction=32`) or IVFFlat. Queries run in a txn with `SET LOCAL hnsw.ef_search=(top_k*4).clamp(40,1000)` (IVFFlat: `ivfflat.probes`), and `iterative_scan` for filtered queries on pgvector ≥ 0.8. Batch `upsert` uses `UNNEST(...) ON CONFLICT (id) DO UPDATE`, chunked at 1000 rows, dimension-validated fail-fast. `count()` reads a trigger-maintained stats table (O(1)). FTS via `websearch_to_tsquery` + `ts_rank_cd`.
- **Graph (`graph/`)**: `PostgresAGEGraphStorage`. Cypher executed through `ag_catalog.cypher('{graph}', $$…$$ , $1::agtype)` with **bound `agtype` params** (dollar-quoted to prevent injection). Upserts use `MERGE (n:Node {node_id}) SET n.key = …` (AGE 1.6 lacks `ON CREATE SET`); batch upserts use `UNWIND [...] AS row MERGE …` with **adaptive chunking (≤512 KB body, 50–500 rows)**. Optional **native SQL write path** (`EDGEQUAKE_NATIVE_GRAPH_WRITES`) replaces Cypher MERGE with `INSERT … ON CONFLICT` (~69× faster at 50K nodes). `reltuples_estimate` gives O(1) counts via `pg_inherits`.
- **KV (`kv.rs`)**: `eq_{prefix}_kv(key PK, value JSONB, timestamps)` + reverse-key btree index for O(log n) suffix scans; O(1) `count` via stats table.
- **RLS (`rls.rs`)**: `acquire_rls_connection(pool, tenant, ws, user)` / `with_acquired_tenant_context(...)` set the GUCs **on a single checked-out connection** (session vars are transaction-local — must not be set pool-wide, or they leak across tenants).

### 6.3 `WorkspaceVectorRegistry`
Table name = `eq_{namespace}_ws_{workspace_id[..8]}_vectors`. `get_or_create(WorkspaceVectorConfig{workspace_id, dimension, namespace})` returns an `Arc<dyn VectorStorage>`; caches + can `evict`. Rationale: mixing embedding dimensions in one table corrupts similarity — each workspace's embedding model gets its own table.

### 6.4 Memory adapter (`src/adapters/memory/`)
Full in-memory implementations (HashMap + RwLock; brute-force cosine; adjacency-list graph) for tests/dev.

---

## 7. LLM & Embedding Layer (`models.toml`)

All provider/model config is data-driven via [`edgequake/models.toml`](edgequake/models.toml) (the single source of truth) plus env overrides. The `edgequake-llm` crate exposes `LLMProvider` (`complete`, `stream`, `chat`, vision/tool-call capability flags) and `EmbeddingProvider` (`embed`, `embed_one`).

**Config priority:** `EDGEQUAKE_MODELS_CONFIG` env → `./models.toml` → `~/.edgequake/models.toml` → bundled defaults.

### 7.1 `[defaults]`
```toml
[defaults]
llm_provider = "openai"          ; llm_model = "gpt-4.1-mini"      # 1M ctx
embedding_provider = "openai"    ; embedding_model = "text-embedding-3-small"  # 1536-d
vision_provider = "openai"       ; vision_model = "gpt-4o"
```

### 7.2 Providers supported
| Provider | type | api_base | key env | default LLM / embed |
|---|---|---|---|---|
| OpenAI | openai | `api.openai.com/v1` | `OPENAI_API_KEY` | gpt-4.1-mini / text-embedding-3-small |
| Anthropic | anthropic | `api.anthropic.com` | `ANTHROPIC_API_KEY` | claude-sonnet-4-x / — |
| Mistral | openai-compat | `api.mistral.ai/v1` | `MISTRAL_API_KEY` | mistral-small-latest / mistral-embed |
| Google Gemini | openai-compat | `…/v1beta/openai` | `GEMINI_API_KEY` | gemini-2.5/3.x-flash / gemini-embedding-001 |
| xAI (Grok) | openai-compat | `api.x.ai/v1` | `XAI_API_KEY` | grok-4.x / — |
| MiniMax | openai-compat | `api.minimax.io/v1` | `MINIMAX_API_KEY` | MiniMax-M2.x / — |
| OpenRouter | openai-compat | `openrouter.ai/api/v1` | `OPENROUTER_API_KEY` | proxies 600+ models |
| Azure OpenAI | azure | custom | `AZURE_OPENAI_API_KEY` | same as OpenAI via Azure |
| Vertex AI | vertex | dynamic (GCP) | ADC / `GOOGLE_APPLICATION_CREDENTIALS` | Gemini on Vertex |
| **Ollama** (local) | ollama | `localhost:11434` | none | gemma4 / embeddinggemma |
| **LM Studio** (local) | openai-compat | `localhost:1234/v1` | none | gemma-3n-e4b-it / nomic-embed |

Each provider has a `priority` (for auto-fallback) and a list of **model cards**:
```toml
[[providers.models]]
name = "gpt-4.1-mini"  ; model_type = "llm"  ; tags = ["recommended","fast"]
[providers.models.capabilities]
context_length = 1047576 ; max_output_tokens = 32768
supports_vision = true ; supports_function_calling = true ; supports_json_mode = true
supports_streaming = true ; embedding_dimension = 0
[providers.models.cost]
input_per_1k = 0.0004 ; output_per_1k = 0.0016
```
Embedding cards set `embedding_dimension` (1536 / 768 / 1024 / 3072). Vision/multimodal cards set `supports_vision=true` and `image_per_unit` cost.

### 7.3 Provider resolution chain
`explicit request param` → workspace config (SPEC-032) → `EDGEQUAKE_DEFAULT_LLM_*` / `EDGEQUAKE_LLM_*` env → `models.toml [defaults]` → hardcoded fallback. A `CachingEmbeddingProvider` (LRU+TTL) wraps the embedder to dedupe repeated texts.

### 7.4 PDF → Markdown backends (`edgequake-pdf`)
Selected by `EDGEQUAKE_PDF_PARSER_BACKEND` (`vision`|`edgeparse`; default `vision`).
- **Vision** (`backend/vision.rs`, uses `edgequake-pdf2md`): renders pages to images (DPI, default 150), sends to a vision LLM, emits Markdown with `<!-- edgequake-page:N -->` markers enabling page-aware chunking. Checkpoint/resume for large PDFs. On failure, auto-falls back to edgeparse.
- **EdgeParse** (`backend/edgeparse.rs`, uses `edgeparse-core`): heuristic, zero-cost, fast, text-only.
- **Inline images** (`inline_images.rs`): extracts embedded images → vision caption → `ImageAsset{base64, description, page}` for multimodal retrieval (SPEC-026).

---

## 8. Ingestion Pipeline — Document → Knowledge Graph

Crate: [`edgequake-pipeline`](edgequake/crates/edgequake-pipeline/). Domain types come from `edgequake-core` (`Document`, `Chunk`, `GraphEntity`, `GraphRelationship`, `Workspace`, `Tenant`). Orchestrated per-task by `edgequake-api`'s `DocumentTaskProcessor`.

### 8.1 Pipeline stages (also the document `current_stage` values)
```
Upload → PdfConversion → Preprocessing → Chunking → Extracting → Gleaning →
Merging → Summarizing → Embedding → Storing → Finalizing
```
Progress is tracked in three layers: task-level `PipelinePhase` (WebSocket/API), UI `UnifiedStage`, internal `PipelineStage` (with per-stage %, item counts, cost tracker).

### 8.2 Entry & options (`ingestion_pipeline.rs`)
`build_ingestion_pipeline(llm, embedding, entity_schema, options)` assembles: chunker (adaptive config) → `LLMExtractor` (optionally wrapped in `GleaningExtractor`) → embedding provider. `IngestionPipelineOptions{document_size_bytes, enable_gleaning, max_gleaning, chunk_strategy, is_pdf_source}`; `calculate_adaptive_chunk_size(bytes)` scales chunk size with document size.

`PipelineConfig` key defaults: `extraction_batch_size=10`, `embedding_batch_size=100`, `max_concurrent_extractions=16`, `chunk_extraction_timeout_secs=180`, `chunk_max_retries=3`, `initial_retry_delay_ms=1000`, lineage tracking on. Env overrides: `EDGEQUAKE_CHUNK_TIMEOUT_SECS`, `EDGEQUAKE_CHUNK_MAX_RETRIES`, `EDGEQUAKE_MAX_CONCURRENT_EXTRACTIONS`.

Three processing modes: `process` (fail-fast), `process_with_progress` (callbacks), `process_with_resilience` (continue past chunk errors, collect `failed_chunks`) — production uses resilient.

### 8.3 Stage 1 — Chunking (`chunker/`)
`ChunkerConfig` defaults: **`chunk_size = 800` tokens**, **`chunk_overlap = 100`** (~12.5%), `min_chunk_size=100`, hierarchical `separators = ["\n\n","\n",". ","! ","? ","; ",", "," ",""]`, `preserve_sentences=true`.

> **Why 800 not 1200:** dense text (tables/formulas/IDs) is 2–3× denser than the 4-char/token estimate; 800 est-tokens ≈ 3200 chars ≈ ~1600 real tokens, staying safely under embeddinggemma's 2048 limit.

Strategies (trait `ChunkingStrategy`): `TokenBased`, `CharacterBased`, `SentenceBoundary`, `ParagraphBoundary`, **`RecursiveCharacter`** (split by separator hierarchy, recurse until size met, then apply overlap), **`Markdown`** (heading-aware, breadcrumb context, SPEC-026), **`PageAware`** (splits on `<!-- edgequake-page:N -->`, **guarantees no chunk spans two PDF pages**, sets `page_start=page_end`). Output `TextChunk{ id, content, index, start/end_offset, start/end_line, token_count, section, page_start/end }`. IDs are content MD5.

### 8.4 Stage 2 — Entity & Relationship Extraction (`extractor/`, `prompts/`)
Trait `EntityExtractor { extract, extract_batch, model_name, provider_name }`. `LLMExtractor` builds a LightRAG-style prompt per chunk and calls the LLM with a large `max_tokens` (e.g. 16384) and `reasoning_effort="none"` for reasoning models (so chain-of-thought doesn't starve the output).

**Prompt (abridged):** *"You are a Knowledge Graph Specialist…"* — extract entities as `entity<|#|>name<|#|>type<|#|>description` and relationships as `relation<|#|>source<|#|>target<|#|>keywords<|#|>description`; **decompose n-ary relations into binary pairs**; treat relations as undirected; entities first then relations; third-person objective language; end with `<|COMPLETE|>`. A JSON parser variant with truncation recovery also exists.

`EntityExtractionSchema::server_default()` types: `PERSON, ORGANIZATION, LOCATION, EVENT, CONCEPT, TECHNOLOGY, PRODUCT, DATE, DOCUMENT` (`strict=true`). Names normalized **UPPERCASE**.

**Gleaning** (`gleaning.rs`, LightRAG-style): after the first pass, if entities empty (or `always_glean`), iterate up to `max_gleaning` (1–3) asking "what did we miss?" and merge new entities; stop when a pass yields nothing new.

Output per chunk: `ExtractionResult{ entities[], relationships[], source_chunk_id, input/output_tokens, extraction_time_ms }`.

### 8.5 Stage 3 — Merge / Dedup into the graph (`merger/`)
`KnowledgeGraphMerger.merge_extractions(...)`:
- **Entities:** normalize name → if node exists, merge descriptions (LLM summarize if `use_llm_summarization`, else concatenate) and accumulate pipe-separated `source_id` (cap 300); else create node.
- **Relationships:** `generate_id(a,b)` is **direction-agnostic** (alphabetical `A<SEP>B`); if exists, concat description, **sum weight**, merge keywords (dedup, ≤5), accumulate sources; else create edge.
- Writes go to `GraphStorage` (AGE) and optionally a **`LineageSink`** (chunk→entity / chunk→relation links, description history) and a **`RelationalEntitySink`** (CQRS read-model dual-write). Both default to no-op sinks; wired in `main.rs` when the tables/feature exist.

### 8.6 Stage 4 — Embedding
Embed chunk content, entity descriptions, and relationship descriptions via the workspace's `EmbeddingProvider` (batched). Vectors carry metadata (`document_id/tenant_id/workspace_id/type`).

### 8.7 Stage 5 — Persistence (`persistence/`)
`IngestionPersister.persist(ctx, result, chunk_options)`: store chunk text in KV; store chunk/entity/relationship vectors in the workspace's `eq_*_vectors` table; merge graph; record lineage; dual-write CQRS. Returns counts. Cascade delete uses `source_id` back-links: deleting a document unlinks its chunks from entities/relationships and removes any now-orphaned nodes/edges.

### 8.8 Lineage (mig 066 + FEAT0011)
`chunk_entity_links` / `chunk_relation_links` M:M tables + chunk span columns (`char_start/end`, `page_start/end`, `embedding_id`) answer: *which PDF page did entity E come from? which chunks built E's description? which entities orphan if doc X is deleted? which entities are shared by X and Y?*

---

## 9. Query / Retrieval Engine

Crate: [`edgequake-query`](edgequake/crates/edgequake-query/). LightRAG-style multi-mode retrieval. `QueryEngine{ vector_storage, graph_storage, embedding_provider, llm_provider, keyword_extractor, tokenizer, kv_storage, reranker, caches }`.

**Config defaults:** `max_entities=60`, `max_relationships=60`, `max_chunks=20`, `max_context_tokens=30000`, `graph_depth=2`, `min_score=0.1`, `use_keyword_extraction=true`, `use_adaptive_mode=true`, `enable_rerank=true`, `enable_bm25_retrieval=true`, `bm25_candidate_multiplier=5`, mix weights `1.0/1.0/1.0`, keyword cache TTL 24 h.

### 9.1 Three-phase pipeline (`query_pipeline.rs`)
1. **PREPARE:** append last 5 conversation turns → **LLM keyword extraction** (high-level concepts + low-level entities + **query intent**) *in parallel with* query embedding → validate low-level keywords against the graph (drop non-existent) → **select mode** (request override → adaptive-from-intent → default `mix`) → compute 3 embeddings (`query`, `high_level`, `low_level`).
2. **RETRIEVE:** mode-specific (below).
3. **FINALIZE:** filter by `allowed_document_ids` → rerank (BM25 fusion / reranker, `min_rerank_score=0.1`) → sort entities by graph degree → **truncate to token budget** → build prompt → LLM answer (or `prompt_only`/stream/vision).

### 9.2 Keyword extraction & intent (`keywords/`)
`CachedKeywordExtractor` (LRU+TTL) → `LLMKeywordExtractor`: prompt returns JSON `{high_level_keywords, low_level_keywords, query_intent}`. Rule-based fallback for mock/unavailable LLM. Intent → recommended mode:

| Intent | pattern | mode |
|---|---|---|
| Factual | "what is/who is/define" | **local** |
| Relational | "how does X relate to Y" | **global** |
| Exploratory | "tell me about" | **naive** |
| Comparative | "compare / vs" | **local** |
| Procedural | "how to" | **mix** |

### 9.3 Modes (`engine_impl/modes/`)
- **naive** — pure vector ANN on `query` embedding, `min_score` filter, top `max_chunks`, **BM25 fusion** (expand pool ×5, PG native FTS or in-memory reranker, RRF or weighted). Chunks only.
- **local** — entity vectors via `low_level` (×3 pool → top 60); fallback to highest-degree graph nodes if none; batch-load node props + degrees; **`edges_within_depth` BFS (depth 2, ≤60 edges, batched `get_incident_edges_batch`, dedup, no N+1)**; then score-rank chunks from entity/rel `source_chunk_ids`. Returns entities+relationships+chunks.
- **global** — relationship vectors via `high_level` (×3 → top 60); extract endpoint entities; **community expansion** (co-community entities, SPEC-023) when `enable_community_global`; chunks from relation sources.
- **hybrid** — run local+global+naive in parallel; **round-robin merge** chunks (local[i], global[i], naive[i], dedup by id); union entities/rels. Optional `EDGEQUAKE_HYBRID_FUSION=rrf`.
- **mix** (production default) — same three arms; **weighted score fusion**: min-max normalize each arm's chunk scores, `blended = weight[arm]·norm_score`, keep max across arms, sort, top-k. Weights overridable per-request; optional `EDGEQUAKE_MIX_FUSION=rrf`.
- **bypass** — no retrieval; direct LLM prompt (for testing / pass-through).

### 9.4 BM25 sparse fusion (`sparse_retrieval.rs`)
Vector pool ×`bm25_candidate_multiplier`; sparse via native `text_search_filtered` (`ts_rank_cd` over `content_tsv`) or in-memory reranker; fuse with **RRF** (weights vector 1.0 / sparse 1.25) or weighted (sparse-first). Env: `EDGEQUAKE_BM25_RETRIEVAL`, `EDGEQUAKE_SPARSE_FUSION`, `EDGEQUAKE_BM25_CANDIDATE_MULTIPLIER`.

### 9.5 Ranking, context, truncation, prompt
- **Entities** ranked by vector score, then re-sorted by **graph degree** (centrality). **Relationships** ranked by vector score.
- **Context string** = three sections: `### Knowledge Graph Data (Entities)`, `### … (Relationships)`, `### Document Chunks` (`[n] (score: …)`).
- **Truncation** (`balance_context`): budget 30 000 tokens, ~33% each (entities/relationships/chunks), `max_entity_tokens=max_relation_tokens=10000`; per-section greedy fill then proportional reduction if still over; tokenizer default ≈ chars/4 (tiktoken pluggable).
- **Prompt** (`prompt.rs`): strict grounding template ("answer ONLY from Context… do not invent… same language… Markdown… partial answer with specifics beats 'insufficient information'"), with injected `system_prompt_extension`, context, and conversation history. Vision path builds a system message (role+instructions+context) + a user message (query+base64 images), falling back to text-only on failure.
- **Generation:** `provider.complete(prompt)` or `provider.chat(messages)`; empty context → apology (except `bypass`).

### 9.6 Streaming & caching
`query_stream*` returns `BoxStream<Result<String, QueryError>>` via `provider.stream(prompt)` (vision streams as a single chunk). Three caches: keyword (LRU+TTL 24 h), embedding (`CachingEmbeddingProvider`, 10k/1 h), query-result/context (LRU+TTL 5 min, invalidated post-ingestion). `eval/` holds a RAGAS skeleton (entity recall, keyword recall) — not fully implemented.

**Returned:** `QueryResponse{ answer, context, mode, stats{embedding_ms, retrieval_ms, generation_ms, total_ms, context_tokens, generated_tokens, rerank_ms} }`.

---

## 10. HTTP API Surface

Crate: [`edgequake-api`](edgequake/crates/edgequake-api/) — Axum server, `AppState` (`src/state/`) holding storage, query engine, pipeline, task queue, auth config, resource budgets, semaphores. Handlers in `src/handlers/`. OpenAPI at `/swagger-ui`. Full route list (registered `.route(...)` paths; most sit under `/api/v1`):

**Health / ops:** `GET /health`, `GET /ready`, `GET /live`, `GET /version`, `GET /metrics`.

**Documents & ingestion:** `POST /documents`, `GET /documents`, `GET/DELETE /documents/{document_id}`, `POST /documents/upload`, `POST /documents/pdf`, `GET/DELETE /documents/pdf/{pdf_id}`, `POST /documents/reprocess`, `POST /documents/scan`, `GET /documents/search`, `POST /documents/recover-stuck`, `DELETE /chunks/{chunk_id}`.

**Query & chat:** `POST /query`, `POST /query/stream` (SSE), `POST /query/context`, `POST /chat`, `POST /chat/completions`, `POST /generate` (Ollama-compatible), `GET /tags`, `GET /ps` (Ollama-compat).

**Knowledge graph:** `GET /graph`, `GET /graph/stream`, `GET /graph/nodes/{node_id}`, `POST /graph/nodes/search`, `GET /graph/entities/{entity_name}`, `POST /graph/entities/exists`, `POST /graph/entities/merge`, `POST /graph/degrees/batch`, `GET /graph/labels/popular`, `GET /graph/labels/search`.

**Conversations:** `GET/POST /conversations`, `GET/PUT/DELETE /conversations/{id}`, `GET/POST /conversations/{id}/messages`, `PATCH/DELETE /messages/{message_id}`, `GET/POST /folders`, `PUT/DELETE /folders/{folder_id}`, `GET /shared/{share_id}`.

**Pipeline / tasks:** `GET /tasks`, `GET /tasks/{track_id}`, `POST /tasks/{track_id}/cancel`, `POST /tasks/{track_id}/retry`, `POST /pipeline/cancel`, `GET /pipeline/status`, `GET /pipeline/queue-metrics`, `POST /pipeline/costs/estimate`, `GET /pipeline/costs/pricing`. WebSockets: `/ws/pipeline/progress`.

**Costs:** `GET /costs/summary`, `GET /costs/history`, `GET/PUT /costs/budget`.

**Models / providers:** `GET /models`, `GET /models/llm`, `GET /models/embedding`, `GET /models/health`, `GET /models/{provider}`, `GET /models/{provider}/{model}`.

**Auth & tenancy:** `POST /auth/login`, `POST /auth/logout`, `POST /auth/refresh`, `GET /auth/me`, `GET /auth/oidc/login`, `GET /auth/oidc/callback`, `GET/POST /api-keys`, `DELETE /api-keys/{key_id}`, `GET/POST /tenants`, `GET/PUT/DELETE /tenants/{tenant_id}`, `GET/POST /users`, `…/users/{user_id}`, `…/workspaces/{workspace_id}`.

**Admin / config:** `GET /admin/config/defaults`, `GET /admin/storage/inspect`, `POST /admin/storage/repair`, `GET /config/effective`.

**MCP server:** `POST /mcp` — a streamable-HTTP Model-Context-Protocol JSON-RPC gateway (`src/mcp/`) exposing EdgeQuake tools to MCP clients, advertised via a `/.well-known/mcp/server.json` descriptor, with `validate_tool_call_with_role` authorization and OTEL spans (`mcp_tools_call`).

**Streaming internals:** SSE/token streaming buffers via a `StreamAccumulator` with a debounced `StreamFlushManager` (~500 ms flush) so partial LLM tokens are coalesced before being pushed to the client.

---

## 11. Authentication, RBAC & Multi-Tenancy

Crate: [`edgequake-auth`](edgequake/crates/edgequake-auth/). Three tiers of identity, PostgreSQL-only as the source of truth (KV mirror removed by migs 048–065).

- **Passwords:** `argon2` hashing (`password.rs`).
- **JWT** (`jwt.rs`): `jsonwebtoken` HS256, `Claims{ sub/user_id, role, exp, … }`; access token ~1 h TTL (+~30 s leeway) plus a longer-lived refresh token; `validate_exp=true`, `is_expired()`/`expires_in()` helpers. Secret from `JWT_SECRET` (random if unset).
- **API keys:** `/api-keys` CRUD; `EDGEQUAKE_API_KEYS` / `EDGEQUAKE_MASTER_API_KEY` for bootstrap.
- **RBAC** (`rbac.rs`, `types.rs`): `Role{ Admin, User }` (readonly at membership level); workspace membership roles `owner/admin/member/readonly`.
- **OIDC** (`oidc_config.rs`): optional built-in OIDC login/callback (`EDGEQUAKE_OIDC_*`); otherwise front with oauth2-proxy.
- **Tenant context** (`tenant.rs`, `extractors.rs`): request → resolve tenant/workspace/user → Axum extractor → set DB RLS GUCs on the acquired connection (see [§5.5](#55-row-level-security-rls--tenant-isolation-in-the-database) / [§6.2](#62-postgresql-adapter-srcadapterspostgres)).
- **Dev bypass:** `EDGEQUAKE_DEV_MODE=true` disables auth locally; `startup_security` warns/aborts on insecure prod config (`EDGEQUAKE_STRICT_STARTUP`).

Hierarchy: **Tenant** (plan: Free/Basic/Pro/Enterprise, workspace/user caps, default models) → **Workspace** (own LLM/embedding/vision model config, own `eq_*_vectors` table) → **User** ↔ **Membership**.

---

## 12. Background Task & Worker System

Crate: [`edgequake-tasks`](edgequake/crates/edgequake-tasks/). Ingestion runs as async jobs so uploads return immediately.

- **Task types:** `Upload, Insert, Scan, Reindex, PdfProcessing, KnowledgeInjection`.
- **Task status:** `Pending → Processing → Indexed | Failed | Cancelled`.
- **Worker pool** (`WorkerPool`, config in `main.rs`): `num_workers = WORKER_THREADS || num_cpus*4` (IO-bound), `auto_retry`, exponential backoff `5s→60s` (×2), `max_tasks_per_tenant = num_workers*3/4` (tenant fairness), `processing_timeout_secs = 7200` (2 h, clamped ≥60) for giant vision PDFs.
- **Persistence:** `tasks` table (DB) + in-memory queue. Workers heartbeat `updated_at` every 60 s.
- **Circuit breaker** (mig 020): `consecutive_timeout_failures` + `circuit_breaker_tripped` stop hammering a persistently-timing-out task.
- **Checkpoints:** pipeline checkpoints in KV allow resume from mid-pipeline; stale ones (>24 h) cleaned at startup.
- **Recovery (startup, before workers start):** `recover_orphaned_tasks` (processing→pending unconditionally, since 0 workers run), `recover_orphaned_documents` (early "uploading"→failed/re-upload; later stages→pending/resume-from-checkpoint), `requeue_pending_tasks` (DB→queue). **Periodic** check every 5 min marks dead-heartbeat (>10 min) processing tasks as failed.
- **Cancellation:** cooperative — the `/tasks/{id}/cancel` (and `/pipeline/cancel`) handler signals a shared `CancellationRegistry` the worker polls mid-pipeline.

---

## 13. Rate Limiting

Crate: [`edgequake-rate-limiter`](edgequake/crates/edgequake-rate-limiter/). **Token-bucket** algorithm (`limiter.rs`): per-key (tenant/workspace) bucket `{ tokens, capacity, refill_rate/sec, last_refill }`; `refill()` adds `elapsed·refill_rate` up to capacity on each check. Chosen over fixed-window (avoids boundary bursts) — smooths traffic while allowing short bursts. Axum middleware (`middleware.rs`) rejects with HTTP 429 and emits `edgequake_rate_limit_exceeded_total{scope}`. Toggle via `EDGEQUAKE_RATE_LIMIT_ENABLED`.

---

## 14. Observability

Crate: [`edgequake-observability`](edgequake/crates/edgequake-observability/). Features `metrics` (default) and `otel`.

- **Init:** `init_observability(ObservabilityConfig::from_env())` at startup → `tracing-subscriber` (text or `EDGEQUAKE_LOG_FORMAT=json`), optional span-close events, optional OTLP export.
- **Correlation:** every request gets a `request_id` + W3C `traceparent`; propagated to LLM calls, logs, metrics.
- **Prometheus `/metrics`** (pre-seeded at zero): `edgequake_http_requests_total{method,path,status}` + `_duration_seconds`; `edgequake_query_requests_total{mode,outcome}` + `_duration`; `edgequake_llm_requests_total{provider,operation,outcome}` + `_duration`; `edgequake_document_processing_total{task_type,stage,outcome}` (+ chunk-strategy/section-context variants); `edgequake_rate_limit_exceeded_total{scope}`; `edgequake_storage_errors_total` / `edgequake_pipeline_errors_total`; `edgequake_db_pool_connections{state}` (sampled every `EDGEQUAKE_DB_POOL_METRICS_INTERVAL_SECS`); `edgequake_task_queue_{pending,processing,failed}`.
- **OTEL:** `--features otel` + `OTEL_EXPORTER_OTLP_ENDPOINT` + `OTEL_SERVICE_NAME` (+ `EDGEQUAKE_OTEL_ENABLED`) → Jaeger/Collector/Datadog.

---

## 15. Configuration Reference

Selected env vars (see [`.env.example`](.env.example) for the full set).

**Database / server:** `DATABASE_URL` (required), `HOST`/`EDGEQUAKE_HOST` (0.0.0.0), `PORT`/`EDGEQUAKE_PORT` (8080), `WORKER_THREADS`.

**LLM / embedding / vision:** `EDGEQUAKE_DEFAULT_LLM_PROVIDER`/`_MODEL`, `EDGEQUAKE_DEFAULT_EMBEDDING_PROVIDER`/`_MODEL`/`_DIMENSION`, `EDGEQUAKE_VISION_PROVIDER`/`_MODEL`; short forms `EDGEQUAKE_LLM_PROVIDER`/`_MODEL`, `EDGEQUAKE_EMBEDDING_PROVIDER`/`_MODEL`. Keys: `OPENAI_API_KEY` (+`OPENAI_BASE_URL`), `ANTHROPIC_API_KEY`, `MISTRAL_API_KEY`, `GEMINI_API_KEY`, `XAI_API_KEY`, `MINIMAX_API_KEY`, `OPENROUTER_API_KEY`, `AZURE_OPENAI_API_KEY`/`_ENDPOINT`, Vertex ADC. Local: `OLLAMA_HOST` (+`OLLAMA_EMBEDDING_HOST`, `OLLAMA_CONTEXT_LENGTH`), `LMSTUDIO_HOST`.

**Pipeline:** `EDGEQUAKE_CHUNK_TIMEOUT_SECS` (180), `EDGEQUAKE_CHUNK_MAX_RETRIES` (3), `EDGEQUAKE_MAX_CONCURRENT_EXTRACTIONS` (16), `EDGEQUAKE_PDF_PARSER_BACKEND` (vision), `TASK_PROCESSING_TIMEOUT_SECS` (7200), `MAX_TASKS_PER_TENANT`.

**Retrieval:** `EDGEQUAKE_MIN_ENTITY_SCORE`, `EDGEQUAKE_BM25_RETRIEVAL`, `EDGEQUAKE_BM25_CANDIDATE_MULTIPLIER`, `EDGEQUAKE_COMMUNITY_GLOBAL`, `EDGEQUAKE_HYBRID_FUSION`, `EDGEQUAKE_SPARSE_FUSION`, `EDGEQUAKE_MIX_FUSION`.

**Graph / storage:** `EDGEQUAKE_NATIVE_GRAPH_WRITES`, `EDGEQUAKE_MEM_LIMIT`.

**Security:** `EDGEQUAKE_DEV_MODE`, `JWT_SECRET`, `EDGEQUAKE_API_KEYS`, `EDGEQUAKE_MASTER_API_KEY`, `EDGEQUAKE_STRICT_STARTUP`, `EDGEQUAKE_CORS_ORIGINS`, `EDGEQUAKE_RATE_LIMIT_ENABLED`, `EDGEQUAKE_OIDC_*`.

**Observability:** `RUST_LOG`, `EDGEQUAKE_LOG_FORMAT`, `EDGEQUAKE_LOG_SPAN_EVENTS`, `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME`, `EDGEQUAKE_OTEL_ENABLED`, `EDGEQUAKE_DB_POOL_METRICS_INTERVAL_SECS`.

---

## 16. Server Startup Sequence

From [`edgequake/src/main.rs`](edgequake/src/main.rs):

1. `init_observability` (tracing/metrics guard).
2. Read `OPENAI_API_KEY` (optional), `DATABASE_URL` (required).
3. `AppState::new_postgres(db_url, api_key)` — connect pool, **run migrations**, build storage/query/pipeline/tasks, resource budgets (graph scan threshold, query timeout, page size).
4. `initialize_defaults()` — default tenant + workspace for non-auth mode.
5. Spawn DB-pool metrics sampler.
6. Build `DocumentTaskProcessor` (strict workspace isolation) wiring pipeline, LLM/embedding, KV/vector/graph storage, progress broadcaster, task storage, PDF-vision semaphore, query engine, CQRS entity sink, lineage sink, PDF storage.
7. `WorkerPoolConfig` (workers = `num_cpus*4`, retries, per-tenant cap, 2 h timeout).
8. **Recovery** (before workers): orphaned tasks → pending, orphaned documents → pending/failed, requeue pending, clean stale checkpoints.
9. `WorkerPool::start()`; share `CancellationRegistry` with `AppState`; spawn 5-min periodic orphan check.
10. `ServerConfig{host,port,cors,compression,swagger}`; print banner; `enforce_startup_security`; `Server::run()` (blocks); graceful worker shutdown on exit.

---

## 17. Step-by-Step Replication Guide

To rebuild EdgeQuake from zero:

1. **Provision PostgreSQL 14+** with extensions `vector`, `age`, `uuid-ossp`, `pg_trgm`. (Use the `edgequake-postgres` image recipe, or install pgvector + Apache AGE manually.)
2. **Create the schema** by applying `migrations/001…077` in order (or point sqlx `migrate!` at the folder). This yields: multi-tenancy spine, `documents/chunks/entities/relationships/tasks`, conversations, partitioned audit logs, RLS policies + context functions, AGE graph (or fallback tables), FTS/trigram indexes, materialized vector columns, HNSW tuning, CQRS entity read-model, chunk-lineage tables.
3. **Stand up the Rust workspace** (11 crates as in [§3](#3-crate-topology-the-11-crates)). Define storage **traits** first (`KVStorage`, `VectorStorage`, `GraphStorage` family, `WorkspaceVectorRegistry`), then Postgres adapters (pgvector cosine/HNSW + AGE Cypher-with-bound-params + JSONB KV + RLS-per-connection), then a memory adapter for tests.
4. **Provide an LLM abstraction** (`LLMProvider` + `EmbeddingProvider`) with the `models.toml` schema and the 11 providers; support per-workspace model/dimension resolution and an embedding cache.
5. **Build the ingestion pipeline** ([§8](#8-ingestion-pipeline--document--knowledge-graph)): PDF→Markdown (vision + edgeparse, page markers) → adaptive/recursive/markdown/page-aware chunking (800/100 tokens) → LightRAG `<|#|>`-tuple entity+relation extraction (+ gleaning) → direction-agnostic merge/dedup into AGE (+ lineage + CQRS sinks) → embed chunks/entities/relations → persist to KV + `eq_*_vectors` + AGE.
6. **Build the query engine** ([§9](#9-query--retrieval-engine)): keyword+intent extraction → mode routing (naive/local/global/hybrid/mix/bypass) → vector ANN + `edges_within_depth` BFS + BM25 fusion → degree-sort + rerank → 30k-token 33/33/33 truncation → strict-grounding prompt → LLM answer (stream/vision).
7. **Expose the Axum API** ([§10](#10-http-api-surface)) with OpenAPI, SSE streaming, WebSocket progress, and the MCP gateway.
8. **Add auth** (argon2 + JWT HS256 + API keys + optional OIDC + RBAC) and enforce **RLS-per-connection** tenant/workspace isolation.
9. **Add the async task system** ([§12](#12-background-task--worker-system)): DB-backed queue + in-memory worker pool, heartbeats, circuit breaker, checkpoints, startup + periodic orphan recovery, cooperative cancellation.
10. **Add rate limiting** (token bucket) and **observability** (Prometheus `/metrics`, W3C tracing, optional OTEL).
11. **Package** the deployment stack ([§4](#4-deployment-topology)): postgres image + API image + frontend, wired by `DATABASE_URL` and provider env vars, with health checks.

---

### Design principles that recur throughout the codebase
- **One PostgreSQL, three roles** (relational + pgvector + AGE) — no separate vector/graph DBs.
- **Traits over implementations** — storage, LLM, extractor, sinks are all dependency-inverted, enabling the memory adapter and per-workspace swaps.
- **CQRS split** — AGE + `eq_*_vectors` are the write/read truth for retrieval; relational `entities/relationships` are a queryable projection.
- **Lineage-first** — every entity/relationship keeps pipe-separated source chunk/doc IDs, enabling cascade delete and citations.
- **Tenant isolation in the database** — RLS GUCs set per checked-out connection, never pool-wide.
- **Resilience** — checkpoints, orphan recovery, circuit breakers, adaptive batching, O(1) counters, and graceful AGE/degraded-mode fallbacks.
- **LightRAG heritage** — `<|#|>` extraction tuples, gleaning, high/low keyword split, 60/60/20 pools, 30k token budget, five retrieval modes.
```
