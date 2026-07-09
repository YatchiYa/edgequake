# 03 — Ingestion Pipeline Compare (Code is Law)

**Lens:** Stage-by-stage truth table.  
**Roots:** EdgeQuake crates vs `/Users/raphaelmansuy/Github/03-working/LightRAG`.

---

## End-to-End ASCII

```text
                         INGESTION (both systems)
 ═══════════════════════════════════════════════════════════════════════

  Document / Text
       │
       ▼
  ┌─────────────┐   EdgeQuake: upload_file / process_text_insert / EdgeQuake::insert
  │  ADMIT      │   LightRAG:  ainsert / apipeline_enqueue_documents / API upload
  └──────┬──────┘
         ▼
  ┌─────────────┐   EQ: PDF→md, markdown IR, page markers
  │  PARSE      │   LR: native | mineru | docling | legacy  (+ sidecar)
  └──────┬──────┘
         ▼
  ┌─────────────┐   EQ: optional VLM multimodal services (SPEC-026)
  │  VLM ANALYZE│   LR: analyze_multimodal (i/t/e flags) — first-class Layer 2
  └──────┬──────┘
         ▼
  ┌─────────────┐   EQ: Recursive(default) | Fixed | Markdown | Pdf
  │  CHUNK      │   LR: F | R | V(semantic) | P(paragraph)   ← EQ missing V
  └──────┬──────┘
         ▼
  ┌─────────────┐   EQ: LLMExtractor (JSON) + GleaningExtractor
  │  EXTRACT    │   LR: extract_entities (JSON or delimiter) + gleaning
  └──────┬──────┘
         ▼
  ┌─────────────┐   EQ: generate_all_embeddings (chunk/entity/rel)
  │  EMBED      │   LR: chunks_vdb + entities_vdb + relationships_vdb
  └──────┬──────┘
         ▼
  ┌─────────────┐   EQ: KnowledgeGraphMerger + LLMSummarizer
  │  MERGE      │   LR: merge_nodes_and_edges + _handle_entity_relation_summary
  └──────┬──────┘
         ▼
  ┌─────────────┐   EQ: KV → vectors → AGE graph (saga compensate)
  │  PERSIST    │   LR: upsert all stores + index_done_callback flush
  └──────┬──────┘
         ▼
  ┌─────────────┐   EQ: schedule_community_index_refresh (Louvain labels)
  │  COMMUNITY  │   LR: visualization only — NOT in RAG path
  └─────────────┘
```

---

## Stage Truth Table

| Stage | EdgeQuake (code) | LightRAG latest (code) | Winner / Gap |
|-------|------------------|------------------------|--------------|
| **Entry** | `EdgeQuake::insert`, `process_text_insert`, `POST /documents/upload` | `LightRAG.ainsert`, `apipeline_process_enqueue_documents` | Tie (EQ stronger task queue) |
| **Parse** | `edgequake-pdf` embedded pdfium; markdown IR | MinerU / Docling / native / legacy | **LR broader**; EQ PDF strong |
| **VLM** | `services/multimodal/*`, `vlm_process_enabled` | `analyze_multimodal`, `VLM_PROCESS_ENABLE` | **LR more staged**; EQ catching up (SPEC-026) |
| **Chunk** | `ChunkStrategy::{Recursive,Fixed,Markdown,Pdf}`, adaptive 600–1200 | F/R/**V**/P; `CHUNK_SIZE=1200` | **LR has semantic V** |
| **Extract** | `LLMExtractor` JSON + `GleaningExtractor` | `extract_entities` JSON/delimiter + gleaning + section context | Near parity; LR section context richer |
| **Entity types** | `EntityExtractionSchema` / `entity_type_policy` | `prompt.py` defaults + YAML profiles | Tie |
| **Embed format (rel)** | `keywords\tsrc->tgt\ndescription` | Same LightRAG format | Parity |
| **Merge** | `KnowledgeGraphMerger`, Jaccard gate, LLM summary | `merge_nodes_and_edges`, force summary thresholds | Near parity |
| **Persist** | Saga: KV→vec→graph; compensate on fail | Multi-backend flush | **EQ stronger consistency story** |
| **Incremental** | Merge-by-name + source tracking | Merge + `rebuild_knowledge_from_chunks` on delete | **LR delete rebuild more mature** |
| **Community** | Index-time Louvain → `community_id` on nodes | Viz only | **EQ has query-usable labels** (not reports) |
| **Tenancy** | Workspace vector registry + RLS | Workspace string isolation | **EQ** |

---

## Chunking Deep Dive

### EdgeQuake

| Knob | Location | Default behavior |
|------|----------|------------------|
| Strategy | `chunker/registry.rs` | Recursive |
| Adaptive size | `adaptive_chunking.rs` | <50KB→1200; 50–100→800; >100→600 |
| Overlap | ~8.3% of size | Adaptive |
| PDF | `ChunkStrategy::Pdf` | Never cross `<!-- edgequake-page:N -->` |
| Markdown | heading breadcrumbs in `SectionMetadata` | Yes |
| Semantic vector | — | **Not implemented** (registry notes absence) |

### LightRAG

| Selector | Function | Notes |
|----------|----------|-------|
| F | `chunking_by_fixed_token` | Default for `ainsert` |
| R | `chunking_by_recursive_character` | Separators configurable |
| V | `chunking_by_semantic_vector` | LangChain SemanticChunker |
| P | `chunking_by_paragraph_semantic` | Default size 2000; heading-aligned |

**Assessment:** EdgeQuake's adaptive Recursive + PDF page awareness is production-grade. Missing **V** is a real quality gap for narrative corpora (GraphRAG-Bench Novel density).

---

## Extraction Deep Dive

```text
EdgeQuake production path
─────────────────────────
  GleaningExtractor
       │ wraps
       ▼
  LLMExtractor  ──► JSON schema prompts (prompts/json_extract.rs)
       │
       │ alternate
       ▼
  EntityExtractionPrompts (tuple delimiter) — LightRAG classic

LightRAG path
─────────────
  extract_entities
       │
       ├─ ENTITY_EXTRACTION_USE_JSON ? JSON prompts : delimiter
       ├─ gleaning pass (entity_extract_max_gleaning)
       ├─ format_heading_context (section breadcrumb)
       └─ multimodal auto-entities for drawing/table/equation
```

**Honest gap:** LightRAG injects multimodal entities into the KG during extract. EdgeQuake multimodal is more service-oriented; ensure extracted VLM text always becomes graph nodes (verify end-to-end in SPEC-026).

---

## Merge & Graph Density

Both systems:

1. Dedupe entities by normalized name.
2. Concatenate / summarize descriptions when too many sources.
3. Upsert entity + relationship vectors.

**Neither** currently exposes ops metrics:

```text
avg_degree = 2|E| / |V|
clustering_coefficient
orphan_entity_rate
description_token_p95
```

Without these, you cannot know if you are building a HippoRAG2-dense graph or a LightRAG-sparse one (GraphRAG-Bench: LightRAG Novel avg degree ~2.1 vs HippoRAG2 ~8.75).

---

## Persistence & Failure Modes

```text
EdgeQuake saga (ingestion_persister.rs)
───────────────────────────────────────
  write KV chunks
       │
       ▼
  upsert chunk vectors
       │
       ▼
  merge graph (+ entity/rel vectors)
       │
       ├─ OK  → schedule community refresh; invalidate query cache
       └─ FAIL→ compensate: delete chunk vectors for document_id
```

**Gap vs ideal:** No 2PC. Acceptable if compensation is tested under kill -9 mid-merge. LightRAG relies on doc_status state machine + resume purge (`_purge_stale_extraction_if_resuming`) — stronger **resume** story.

**Action:** Port LightRAG-style process_options fingerprint + stale purge into EdgeQuake reanalyze path.

---

## Ingestion Scorecard (honest)

| Dimension | EdgeQuake | LightRAG latest | Notes |
|-----------|:---------:|:---------------:|-------|
| Parse breadth | 3/5 | 5/5 | MinerU/Docling |
| PDF fidelity | 5/5 | 4/5 | EQ embedded pdfium |
| Chunk strategies | 4/5 | 5/5 | Missing semantic V |
| Extraction quality knobs | 4/5 | 5/5 | Section + multimodal inject |
| Merge / incremental | 4/5 | 5/5 | LR delete rebuild |
| Persist reliability | 5/5 | 4/5 | EQ saga + Postgres |
| Multi-tenant | 5/5 | 3/5 | EQ RLS / workspaces |
| Graph quality telemetry | 1/5 | 1/5 | Both weak |
| **Overall ingest** | **4.0** | **4.3** | EQ wins ops; LR wins extract/chunk tip |

---

## Code Citations (ingest)

| Claim | EdgeQuake | LightRAG |
|-------|-----------|----------|
| Insert entry | `edgequake-core/.../ingestion.rs` | `lightrag.py:ainsert` |
| Pipeline factory | `edgequake-pipeline/src/ingestion_pipeline.rs` | `pipeline.py:process_single_document` |
| Chunk registry | `chunker/registry.rs` | `parser/routing.py:resolve_chunk_options` |
| Extract | `extractor/llm.rs`, `gleaning.rs` | `operate.py:extract_entities` |
| Merge | `merger/entity.rs`, `relationship.rs` | `operate.py:merge_nodes_and_edges` |
| Summarize | `summarizer.rs` | `operate.py:_handle_entity_relation_summary` |
| Persist | `persistence/ingestion_persister.rs` | `pipeline.py` upsert + `_insert_done` |
| Community | `edgequake-storage/src/community_persist.rs` | visualizer only |
