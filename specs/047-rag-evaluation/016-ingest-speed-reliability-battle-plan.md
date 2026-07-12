# 016 — Ingest Speed & Reliability Battle Plan

**Status:** implementing (P0–P7f code landed 2026-07-11)  
**Lenses:** AI Engineering · O(n) · Apache AGE · Postgres · pgvector · System Design  
**Law:** every work item cites a symbol (FP7 / IP7)  
**Companion canvas:** `ingest-battle-plan-speed-reliability.canvas.tsx`  
**Cross-ref:** [014](./014-ingest-query-pipeline-first-principles.md) · [e2e/README](./e2e/README.md)

---

## Implementation status (2026-07-11)

| Phase | Item | Status | Evidence |
|-------|------|--------|----------|
| P0 | Ops freeze | ✅ | bench047 `force_reindex=not resume`; Makefile caps |
| P1a | `delete_by_document` on force_reindex | ✅ | `VectorStorage::delete_by_document`; helpers + reingest; `contract_spec047_delete_by_document` |
| P1b | Orphan vector sweeper | ✅ | storage_inspector INV-01/D uses `COALESCE(document_id, metadata)` + startup auto-repair |
| P1c | Unify MM fail policy | ✅ | `should_abort_multimodal_hard_error` SSOT; PDF + reanalyze |
| P2a | Parallel MM tables/equations | ✅ | `analyzer.rs` `buffer_unordered` + `mm_item_concurrency` |
| P3a | Stream extract futures (Send-safe) | ✅ | own chunk clones → `stream::iter(owned)` (not slice borrow) |
| P4a | Native graph writes + community gate | ✅ | Makefile `NATIVE_GRAPH_WRITES=1`; `community_auto_max_nodes` on ingest refresh |
| P4b | Batch entity lineage | ✅ | `EntityLineageLink` + `record_entity_links_batch` + Postgres UNNEST |
| P5 | Scale hardening | ✅ | slim checkpoints; stage honesty; `EDGEQUAKE_VECTOR_UPSERT_CHUNK`; ingest profiles |
| P6 | LightRAG embed order | ✅ | **2026-07-11:** unique-before-embed (`unique_embed.rs`); parallel sub-batches (`EDGEQUAKE_EMBED_MAX_ASYNC=8`); merger vector dedupe; soft-reprocess single-flight. Contracts: `contract_spec047_unique_before_embed` |
| P7a | LightRAG merge LLM gate | ✅ | **2026-07-11:** `FORCE_LLM_SUMMARY_ON_MERGE=8` + `<SEP>` join (`description_merge.rs`); Jaccard kept as soft-resume skip; DI `DescriptionMergeBackend`. Contracts: `contract_spec047_p7a_force_llm_summary`, `e2e_spec047_p7a_force_llm_summary` |
| P7b | Parallel unique merge | ✅ | **2026-07-11:** `buffer_unordered(merge_max_async)` on unique entity/rel description merges (`entity.rs` / `relationship.rs`); env `EDGEQUAKE_MERGE_MAX_ASYNC` (default 8 = llm×2). |
| P7c | Soft-resume skip LLM | ✅ | Covered by P7a fragment gate + e2e `e2e_p7c_soft_resume_existing_edge_below_force_no_llm` |
| P7d | SOURCE_IDS KEEP | ✅ | **2026-07-11:** `merge_limits.rs` SSOT (`KEEP`/`FIFO`, max 200); saturated skip before description LLM; stats `*_skipped_saturated`. Contracts: `contract_spec047_p7bcd_merge_limits`, `e2e_spec047_p7bcd_merge_limits` |
| P7e | Merge-only / reuse extractions | ✅ | **2026-07-11:** durable `-extraction-snapshot` survives finalize; `ReprocessMode::MergeOnly`; `plan_extraction_reuse` SSOT. Soft-reprocess skips LLM extract when snapshot present. Contracts: `contract_spec047_p7e_extraction_reuse`, `e2e_spec047_p7e_extraction_reuse` |
| P7f | Batch native AGE upserts | ✅ | **2026-07-11:** `resolve_graph_upsert_chunk` SSOT (`EDGEQUAKE_GRAPH_UPSERT_CHUNK=500`); wired into Cypher + native node/edge adaptive chunking. Contract: `contract_spec047_p7ef_graph_upsert` |

**Contract tests:** `edgequake-storage` / `edgequake-pipeline` / `edgequake-api` `contract_spec047_*`

---

## 0. First principles

| ID | Principle | Implication |
|----|-----------|-------------|
| P1 | Forward-only information | Vision/MM losses are permanent |
| P2 | Cost ∝ irreversible work | Never wipe markdown to fix extract |
| P3 | Amortize to O(batch) | UNNEST 1k; avoid Cypher MERGE at scale |
| P4 | Memory peak ≠ concurrency | `buffer_unordered(k)` ≠ O(k) allocation |
| P5 | Admission before throughput | `VISION_JOBS × CONCURRENCY` is a hard product |
| P6 | Compensation ≠ atomicity | Crash windows need sweepers |
| P7 | Code is law | No vibes |

**System axiom:** ingest is a **multi-store saga** (KV → chunk vectors → AGE), not 2PC.

---

## 1. Complexity map (persist + convert)

| Phase | Asymptotic | Symbol | Note |
|-------|------------|--------|------|
| Vision pages P | O(P / conc) wall | `VisionPdfConverter` | Dominant on fresh ingest |
| MM images I | O(I / mm_conc) | `analyzer.rs` parallel | Tables/eq still sequential |
| Extract C | O(⌈C/16⌉ × RTT) | `resilient_extract_parallel` | **Futures `Vec` = O(C) RAM** |
| KV + vectors | O(C) amortized | batch 1000 | Correct pgvector pattern |
| Entity merge E | O(E) + **O(E×S) lineage** | `entity.rs::record_entity_link` | Silent N+1 |
| Rel merge R | O(R) + O(R²) endpoint scan | `relationship.rs` | Dedup required |
| Community | O(N+E) sample | `community.rs` | Ingest refresh **unguarded** |
| force_reindex | Graph cascade only | `helpers.rs:318–322` | **Chunk vectors not deleted** |

---

## 2. Expert findings (code + research)

### AI Engineering
- Caption-and-index is lawful until `page_hit` / Chart fidelity plateau.
- Entity extract is optional for retrieve-only eval (soft-resume wall-clock is extract-dominated).
- Soft-resume without `<drawing>` tags → MM specialize no-op (MV-32 assets missing).

### Postgres (saga)
- Order: KV upsert → vector upsert (1 txn) → `KnowledgeGraphMerger` → bg community.
- Merge failure → `compensate_merge_failure_with_kv` (best-effort).
- **Gap:** crash after vectors, before merge → orphan window, no sweeper.

### pgvector
- HNSW default `m=16`, `ef_construction=32`; upsert `UNNEST` 1000 in one txn — aligned with 2026 batch guidance.
- **Gap:** `clear_document_derived_data` does not `delete_by_document` chunk vectors.
- FTS: `content_tsv` from `metadata->>'content'` but chunks use `content_ref` → query uses KV JOIN.

### Apache AGE
- Cypher `UNWIND MERGE` degrades superlinearly past ~10k edges ([AGE #2177](https://github.com/apache/age/issues/2177); Kartograph: native SQL ~230× faster at 10k nodes).
- EdgeQuake escape hatch: `EDGEQUAKE_NATIVE_GRAPH_WRITES=1` → SQL `ON CONFLICT`.
- Default statement timeout **15s** (`EDGEQUAKE_GRAPH_QUERY_TIMEOUT_SECS`) — large Cypher batches abort.
- Community: pages edges client-side; size gate 50k nodes; ingest path bypasses `ResourceGuard`.

### System design
- Three checkpoints: L1 DB markdown · L2 vision page store · L3 KG pipeline CP.
- Admission: `WORKER_THREADS` ⊃ `MAX_TASKS_PER_TENANT` ⊃ `PDF_VISION_JOBS` ⊃ `PDF_CONCURRENCY` ⊃ `MM_IMAGE_CONCURRENCY` ⊃ extract concurrency.

---

## 3. Edge-case matrix (must not regress)

| Edge case | Handle | Gap? |
|-----------|--------|------|
| force_reindex on resume | `force_reindex=not resume` + soft-reprocess | Fixed (bench047) |
| Markdown kept, doc row gone | `ensure_document_record` + `link_pdf` | Ops playbook |
| Stuck `processing` | Fail on boot / manual clear | Still manual |
| OOM mid-vision | L2 checkpoints + lower concurrency | `MEM_LIMIT` unset warn |
| Vision 0 pages | Hard fail | OK |
| Empty page | Placeholder kept | OK |
| MM strict fail on PDF | Warn, continue | Inconsistent vs reanalyze |
| 0 chunks / all extract fail | `failed` | OK |
| 0 entities | `partial_failure` | OK for retrieve |
| Merge fail | Compensation | Best-effort |
| Crash mid-saga | — | **GAP: no sweeper** |
| force_reindex vectors | Same-id overwrite | **GAP: no delete_by_document** |
| Duplicate hash | `document_reingest` | OK |
| Wrong workspace vector table | Hard abort | OK |
| Self-loop / dup (src,tgt) | Skip / dedup | OK |
| AGE 15s timeout | Native writes / smaller batches | Ops |
| Community huge graph | `COMMUNITY_GLOBAL=false` | **GAP: unguarded refresh** |
| LLM merge summarization | Jaccard gate | Can dominate re-ingest |
| Soft-resume w/o drawing tags | Full reprocess if need assets | Documented |
| Multi-tenant starve | `MAX_TASKS_PER_TENANT` | OK |

---

## 4. Phased plan (gates required)

### P0 — Ops freeze ✅
- `force_reindex = not resume`
- Soft-reprocess keeps markdown
- `PDF_VISION_JOBS=2`, `PDF_CONCURRENCY=2`, `MM_IMAGE_CONCURRENCY=4`
- `EDGEQUAKE_COMMUNITY_GLOBAL=false` for bench  
**Gate:** `RESUME: Markdown already stored` on retry; no OOM on 8-doc smoke.

### P1 — Wipe & saga correctness
| Work | Anchor | Test |
|------|--------|------|
| `delete_vectors_by_document_id` on force_reindex | `helpers.rs:318–322` | vector count = 0 after wipe |
| Orphan vector sweeper on boot | `compensation.rs` | kill mid-persist → clean |
| Unify MM fail policy PDF vs reanalyze | `stage.rs` / `reanalyze.rs` | contract |
| Stuck-task → failed on boot | orphan recovery | boot fixture |

**Gate:** no stale vectors after force_reindex; no orphans after crash.

### P2 — Convert speed
| Work | Win | Risk |
|------|-----|------|
| EdgeParse auto-route born-digital | drop VLM tax | scanned PDFs |
| Parallel MM tables/equations | O(T)→O(T/k) | span order |
| Stage wall-clock spans | measure IP6 | log volume |
| MM content_hash cache | skip unchanged | stale specialize |

**Gate:** p50 convert ↓ on chart smoke fixture.

### P3 — Extract O(C)
| Work | Win |
|------|-----|
| Stream futures (no full `Vec`) | peak RAM O(k) not O(C) |
| `P0_retrieve_only` / EntitiesOnly for page_hit eval | skip LLM tax |
| Cap identical JSON retries | avoid 3× dead RTT |

**Gate:** soft-resume 117p extract wall-clock ↓; RSS peak ↓.

### P4 — Persist
| Work | Rationale |
|------|-----------|
| Default `EDGEQUAKE_NATIVE_GRAPH_WRITES=1` on ingest | AGE Cypher superlinear |
| Batch entity lineage | kill O(E×S) |
| `ResourceGuard` on community refresh | prevent Louvain stampede |
| Keep vector 1 txn / 1k | HNSW batch physics |

**Gate:** merge p95 ↓; AGE timeout rate → 0 on 8-doc.

### P5 — Scale hardening ✅ (2026-07-11)
| Work | Anchor | Status |
|------|--------|--------|
| Slim checkpoints (omit embeddings + size guard) | `pipeline_checkpoint.rs` · `ProcessingResult::strip_embeddings` | ✅ |
| Resume re-embed | `Pipeline::ensure_embeddings` in `extraction.rs` | ✅ |
| Stage honesty (no false Embedding 100%) | `extraction.rs` / `persist.rs` / `pdf_tracking.rs` ETA | ✅ |
| Merge vector upsert progress | `merger::upsert_vectors_chunked` + `EDGEQUAKE_VECTOR_UPSERT_CHUNK` | ✅ |
| Early PG `documents.status` sync | `touch_document_status` | ✅ |
| `EDGEQUAKE_INGEST_PROFILE=chunk_only\|retrieve_only` | `IngestProfile` · `PipelineConfig::from_env` | ✅ |
| Require `EDGEQUAKE_MEM_LIMIT` in prod | ops | ⏳ |
| HNSW `REINDEX CONCURRENTLY` playbook | ops | ⏳ |

**Gate:** mega-doc checkpoint saves; UI shows indexing/merge progress; chunk_only skips KG extract.

---

## 5. Reject

- Blind `PDF_CONCURRENCY=8` without `MEM_LIMIT`
- `force_reindex` on every resume
- Single Cypher UNWIND of 10k+ edges
- 2PC across stores (use saga + sweeper)
- ColPali re-architecture before page_hit plateau
- Disabling checkpoints

---

## 6. Operating checklist

1. Fresh: force_reindex once · caps 2×2 · community off  
2. Crash: `--resume` · soft-reprocess · expect RESUME skip convert  
3. Before raising concurrency: `MEM_LIMIT` + stage timers green  
4. Before large graph: `NATIVE_GRAPH_WRITES=1` · watch merge p95  
5. After force_reindex waves: verify vector count by `document_id`  
6. Retrieve eval: prefer skip-extract profile when measuring `page_hit`

---

## 7. Implementation order (next code)

1. **P1a** — `delete_vectors_by_document` in `clear_document_derived_data`  
2. **P4a** — enable native graph writes in bench/Makefile start script  
3. **P3a** — stream extract futures  
4. **P2a** — parallel MM tables/equations  
5. **P1b** — orphan sweeper  

Each PR: one phase slice · contract test · no Acc-chasing.
