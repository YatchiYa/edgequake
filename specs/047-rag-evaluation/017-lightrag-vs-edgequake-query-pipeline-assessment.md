# 017 — LightRAG vs EdgeQuake: Query & Ingest Pipeline Assessment

**Status:** assessment (code is law) · 2026-07-11  
**Repos:** [LightRAG](file:///Users/raphaelmansuy/Github/03-working/LightRAG) · EdgeQuake `edgequake-query` / `edgequake-pipeline` / `edgequake-api`  
**Cross-ref:** [014](./014-ingest-query-pipeline-first-principles.md) · [015](./015-modality-aware-vision-improvement-plan.md) · [016](./016-ingest-speed-reliability-battle-plan.md) · [013](./013-first-principles-improvement-roadmap.md)  
**Companion canvas:** [spec047-query-pipeline-eq-vs-lightrag](/Users/raphaelmansuy/.cursor/projects/Users-raphaelmansuy-Github-03-working-edgequake/canvases/spec047-query-pipeline-eq-vs-lightrag.canvas.tsx)  
**Action plan:** [018](./018-quality-speed-improvement-plan.md)

---

## 0. One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  QUERY (quality): EdgeQuake ≥ LightRAG on hybrid retrieval primitives        │
│    EQ: dense + Postgres FTS/BM25 + RRF Mix + rerank + modality + doc scope   │
│    LR: dense-only VDB + optional rerank + mix (KG + vector) · no FTS/scope   │
│                                                                              │
│  QUERY (naming trap): hybrid ≠ hybrid · mix ≠ mix                            │
│    LR hybrid = local+global · EQ hybrid = local+global+naive (round-robin) │
│    LR mix = KG + naive vector · EQ mix = 3-arm RRF + intent arm gate         │
│                                                                              │
│  INGEST (quality): EQ ahead on multimodal / page / atomic / modality stamps  │
│  INGEST (speed): EQ P6–P7f closed many LR gaps; vision+extract still dominate│
│                                                                              │
│  BENCH (smoke P0_mm_ite, document_scope): Acc≈0.38 · page_hit@5≈0.76         │
│    Chart Acc≈0.14 → representation + prompt grounding still the bottleneck   │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Law (FP7):** every claim below cites a symbol that exists today. No vibes.

---

## 1. First principles for this comparison

| ID | Principle | Implication |
|----|-----------|-------------|
| C1 | Same task, fair modes | Cross-system Acc only with explicit mode+env map (see §3) |
| C2 | Information flows forward | Query cannot beat ingest representation (chart numbers) |
| C3 | Measure retrieval before generation | `page_hit@k` / arm hit-rates before prompt tweaks |
| C4 | Production RAG 2026 baseline | Dense + sparse + RRF + rerank ([hybrid RAG practice](https://atlan.com/know/hybrid-rag/); [production RAG 2026](https://1337skills.com/blog/2026-06-12-production-rag-2026-hybrid-search-reranking-graphrag/)) |
| C5 | Speed = amortize irreversible work | Vision/extract dominate; never wipe markdown to “fix” merge |

---

## 2. Query pipeline — call graphs (code)

### 2.1 LightRAG

```text
LightRAG.aquery_llm()                         lightrag/lightrag.py
  local|global|hybrid|mix → operate.kg_query()   operate.py
  naive                   → operate.naive_query()
  bypass                  → LLM only

kg_query
  get_keywords_from_query → extract_keywords_only   (hl / ll JSON)
  _build_query_context
    _perform_kg_search
      local  → _get_node_data(ll) → related edges
      global → _get_edge_data(hl) → related entities
      hybrid → both (round-robin entity/relation merge)
      mix    → both + _get_vector_context(query)
    _apply_token_truncation (entity / relation caps)
    _merge_all_chunks (entity-linked + relation-linked [+ vector])
    process_chunks_unified → apply_rerank_if_enabled
  LLM + PROMPTS["rag_response"]
```

**Defaults** (`lightrag/base.py` `QueryParam`, `constants.py`):

| Knob | LightRAG default |
|------|------------------|
| `mode` | **`mix`** |
| `top_k` | 40 (`TOP_K`) |
| `chunk_top_k` | 20 |
| `max_entity_tokens` | 6000 |
| `max_relation_tokens` | 8000 |
| `max_total_tokens` | 30000 |
| `enable_rerank` | true (needs `rerank_model_func`; server often unbound) |
| Cosine threshold | 0.2 |
| Sparse / FTS | **none** |
| Document scope | **none** (workspace only) |
| Modality filter | **none** |

### 2.2 EdgeQuake

```text
POST /api/v1/query → query_execute → execute_sota_query*
  QueryEngine::run_query_pipeline                 query_pipeline.rs
    pipeline_prepare  (keywords ∥ embed_one; 3-level embeddings)
    pipeline_retrieve → mode router
      Naive  → naive.rs (+ BM25 fuse + modality_retrieve)
      Local  → local.rs (entity VDB → expand → KG chunks)
      Global → global.rs (rel VDB → entities → optional community)
      Hybrid → hybrid.rs  local ∥ global ∥ naive → hybrid_merge
      Mix    → mix.rs     local ∥ global ∥ naive → RRF/weighted
    pipeline_finalize → postprocess (doc filter, rerank, balance_context)
                      → prompt.rs → LLM
```

**Defaults** (`QueryEngineConfig`, `truncation.rs`, `mix_weights.rs`):

| Knob | EdgeQuake default |
|------|-------------------|
| `default_mode` | **`Mix`** |
| Bench047 locked mode | **`hybrid`** (see 000-index) |
| `max_entities` / `max_relationships` | 60 / 60 |
| `max_chunks` | 20 |
| `max_context_tokens` | 30000 |
| Soft entity/rel caps | 10000 / 10000 |
| `enable_bm25_retrieval` | **true** |
| `enable_rerank` | true (`min_rerank_score` 0.1) |
| Mix fusion | **RRF** (`EDGEQUAKE_MIX_FUSION`, k=60) |
| Hybrid fusion | **round-robin** (`EDGEQUAKE_HYBRID_FUSION`) |
| Intent arm gate | **on** (`EDGEQUAKE_MIX_ARM_GATE`) — Factual → naive-only |
| Document scope | Tier-1 `MetadataFilter` + post-filter |
| Chart modality pre-filter | on (`EDGEQUAKE_CHART_MODALITY_FILTER`) |

---

## 3. Naming trap (must not confuse benchmarks)

| Name     | LightRAG                         | EdgeQuake                               | Fair bench alias        |
| ----------| ----------------------------------| -----------------------------------------| -------------------------|
| `naive`  | chunk VDB only                   | chunk VDB + FTS/BM25 + modality         | EQ richer by default    |
| `local`  | entity VDB → 1-hop               | entity VDB → depth-2 expand             | roughly comparable      |
| `global` | relation VDB                     | relation VDB + optional community       | EQ optional richer      |
| `hybrid` | **local + global only**          | **local + global + naive**, round-robin | **not comparable**      |
| `mix`    | local+global **+ vector chunks** | 3-arm **RRF** + intent gate             | closest quality default |
| `bypass` | LLM only                         | LLM only                                | equal                   |

**Fair comparison recipes**

| Goal | EdgeQuake settings | LightRAG settings |
|------|--------------------|-------------------|
| Closest to LR `mix` | `mode=mix`, `EDGEQUAKE_MIX_ARM_GATE=false`, BM25 on | `mode=mix` |
| Closest to LR `hybrid` | `mode=hybrid` is **not** enough (includes naive); prefer Mix with naive weight 0 or Local∥Global only experiment | `mode=hybrid` |
| Dense-only naive | `mode=naive`, `enable_bm25_retrieval=false` | `mode=naive` |
| SPEC-047 smoke | `mode=hybrid` + `--document-scope` (current harness) | N/A |

---

## 4. Capability scorecard (1–5, code-grounded)

| Capability | EQ | LR | Winner | Evidence |
|------------|----|----|--------|----------|
| Dense chunk retrieve | 5 | 5 | tie | both have chunks_vdb / vector storage |
| Sparse / lexical | **5** | 1 | **EQ** | `sparse_retrieval.rs` + Postgres FTS; LR dense-only |
| KG local/global | 4 | 4 | tie | both entity/rel VDB + expand |
| Mix / multi-arm fusion | **5** | 4 | **EQ** | RRF + weights + arm telemetry vs LR round-robin merge |
| Rerank | 4 | 3 | EQ | both default-on; LR often unbound in server |
| Token budget discipline | 4 | 4 | tie | both ~30k total; EQ dynamic chunk remainder |
| Intent routing / arm gate | **4** | 2 | **EQ** | `intent_arm_mask`; LR mode-only |
| Document scoping | **5** | 1 | **EQ** | `document_filter` / metadata filter |
| Modality-aware retrieve | **4** | 1 | **EQ** | `modality_retrieve.rs` + ingest stamps |
| Page grounding in prompt | **2** | 2 | weak | EQ `page_start` on chunk **not** in `to_context_string` |
| Query observability | **5** | 2 | **EQ** | `QueryStats` arm ms/chunks, FTS, modality |
| Multimodal ingest → index | **4** | 4 | tie+ | both i/t/e style; EQ page markers + atomic blocks |
| Unique-before-embed | **5** | 4 | EQ | SPEC-047 P6; LR deferred embed + merge |
| Community / GraphRAG reports | 3 | 3 | tie− | neither full MS GraphRAG Leiden by default |
| Cross-doc multi-hop science | 3 | 3 | open | optional PPR in EQ; not default HippoRAG2-class |

**Weighted read:** EdgeQuake is the stronger **production hybrid RAG** substrate. LightRAG remains the cleaner **reference GraphRAG** for paper-aligned mode semantics and Python ecosystem velocity.

---

## 5. Ingest quality & speed (what query depends on)

### 5.1 Quality chain (forward-only)

| Stage | LightRAG | EdgeQuake | Risk if weak |
|-------|----------|-----------|--------------|
| Parse / vision | native / mineru / docling + VLM analyze | `edgequake-pdf` vision / EdgeParse + MM analyzer | Chart Acc collapse |
| Chunking | F/R/V/P; default ~1200 tok | Recursive default 800 (+ adaptive); Pdf page-aware; **atomic blocks** | Split charts from numbers |
| Multimodal stamps | sidecar entities (i/t/e) | `<drawing>` + `modality` on chunks | No chart pre-filter |
| Extract + glean | max_gleaning=1, async=4 | gleaning cap≤2, concurrent=16 | Graph noise / cost |
| Merge | `<SEP>` + force LLM @8 | P7a–P7f parity landed | Duplicate entities |
| Embed | deferred unique-ish | unique-before-embed + batch | Waste RTT |
| Persist saga | KV + VDB + graph | KV + pgvector + AGE (+ sweeper) | Orphans |

### 5.2 Speed levers (wall-clock)

| Lever | EQ status (016) | vs LightRAG |
|-------|-----------------|-------------|
| Vision concurrency product | admission caps in Makefile | similar ops concern |
| Parallel MM tables | `buffer_unordered` | LR analyze workers |
| Extract stream | Send-safe owned futures | LR semaphore(4) |
| Unique-before-embed | ✅ P6 | LR merge-then-embed |
| Merge LLM gate + parallel | ✅ P7a/b | LR force_llm_summary_on_merge=8 |
| SOURCE_IDS KEEP | ✅ P7d | LR KEEP/FIFO 200 |
| Extraction snapshot reuse | ✅ P7e | LR resume purge |
| Native AGE upserts | ✅ P7f | N/A (Python graph backends) |

**Bottleneck truth:** for PDF benches, **vision + extract RTT** dominate. Fusion/rerank at query are secondary until `answer_in_evidence` / Chart fidelity rise.

---

## 6. Smoke evidence (EdgeQuake only)

From [`e2e/artifacts/smoke/SUMMARY.md`](./e2e/artifacts/smoke/SUMMARY.md) (2026-07-11, `P0_mm_ite`, `mode=hybrid`, document_scope):

| Metric | Value | Reading |
|--------|-------|---------|
| Acc / F1 | 0.38 / 0.22 | Plumbing valid; not LVLM parity |
| `page_hit@5` | **0.76** | Retrieval often finds the page |
| Chart Acc | **0.14** | Representation / grounding gap |
| Unanswerable Acc | 0.69 | Do not ban refusal |
| mean naive arm chunks | 19.0 | Hybrid still floods naive |
| mean local/global chunks | ~3.4 / 3.8 | KG arms present but thin |

**Causal story:** page_hit is decent under document scope; Chart Acc lags → **ingest representation + prompt page headers** (014 §3) before more fusion tuning.

---

## 7. Where EdgeQuake already wins (keep)

1. **Dense + sparse hybrid** — 2026 production baseline; LR lacks query FTS.  
2. **Mix RRF + arm telemetry** — measurable fusion; LR merge is opaque.  
3. **Document scope** — fair MMLongBench adaptation; LR cannot match without fork.  
4. **Modality stamps + chart filter** — unique EQ path for SPEC-047.  
5. **Ingest saga hygiene (016 P0–P7f)** — force_reindex, unique embed, merge gates.  
6. **Tenancy / workspace KEYWORD LLM** — enterprise moat.

---

## 8. Where EdgeQuake must improve (quality first, then speed)

### Query (quality)

| Gap | Symbol | Why it hurts Acc |
|-----|--------|------------------|
| Pages invisible to LLM | `context.rs::to_context_string` | LLM cannot cite/ground pages that retrieval already found |
| Bench locked to `hybrid` round-robin | `000-index`, `hybrid_merge.rs` | Dilutes naive page hits vs Mix RRF |
| Intent gate on factual | `intent_arm_mask` | Good for latency; may starve KG on relational-looking chart Qs if misclassified |
| Graph token tax | `truncation::balance_context` | Soft 10k/10k can starve chunks |
| Rerank without strong cross-encoder | API `enable_rerank` | BM25-ish rerank ≠ Cohere/Jina class when unbound |

### Ingest (quality → query)

| Gap | Symbol | Why |
|-----|--------|-----|
| Chart/table second-pass | `multimodal/analyzer.rs` + `process_options` | Chart Acc 0.14 |
| Empty vision page skip | `edgequake-pdf/.../vision.rs` | Silent page dropout |
| Atomic block + page marker in embed text | `atomic_blocks.rs`, page_aware | Numbers split from captions |
| Prompt still text-only extract | `extractor/llm.rs` | Visual structure compressed |

### Ingest (speed)

| Gap | Still open after 016 | Note |
|-----|----------------------|------|
| Vision dominant | always | Soft-resume / markdown reuse is the #1 speed win |
| Community refresh unguarded | 016 notes | Scale risk |
| Soft-resume without drawings | MM specialize no-op | Re-ingest cost surprise |

---

## 9. Research alignment (2025–2026)

- **Minimum viable production retrieve:** BM25 + dense + RRF, then rerank ([1337skills](https://1337skills.com/blog/2026-06-12-production-rag-2026-hybrid-search-reranking-graphrag/), [Atlan hybrid RAG](https://atlan.com/know/hybrid-rag/)). EdgeQuake already implements this; LightRAG does not (dense-only).  
- **Text+table/chart corpora:** hybrid fusion beats dense alone; lexical helps numerics ([arXiv:2604.01733](https://arxiv.org/html/2604.01733v1)). Supports keeping BM25 on for SPEC-047.  
- **GraphRAG:** expensive; add when multi-hop fails after hybrid baseline — matches EQ intent gate philosophy.  
- **Caption-and-index multimodal:** lawful until `page_hit` / Chart fidelity plateau; then typed vision prompts ([015](./015-modality-aware-vision-improvement-plan.md)).

---

## 10. Bottom line

| Question | Answer |
|----------|--------|
| Is EQ query “better” than LightRAG? | **Yes on production hybrid primitives** (sparse, RRF Mix, scope, modality, telemetry). |
| Is EQ paper-parity with LightRAG modes? | **No** — rename/document the trap; never claim hybrid≡hybrid. |
| What limits SPEC-047 Acc today? | **Ingest representation of charts/tables + prompt page grounding**, not missing BM25. |
| What to do next? | Execute [018](./018-quality-speed-improvement-plan.md) in causal order. |
