# 014 — Ingest & Query Pipeline Study (Code Is Law)

**Cross-ref:** [013](./013-first-principles-improvement-roadmap.md) · smoke [`SUMMARY`](./e2e/artifacts/smoke/SUMMARY.md) · [`FIDELITY`](./e2e/artifacts/smoke/FIDELITY.md)  
**Status:** analysis (2026-07-10)  
**Baseline:** Acc≈0.41 · `page_hit@5`≈0.59 · answer_in_evidence≈0.51 · Chart Acc≈0.05

---

## 0. First principles

| ID | Law | Pipeline meaning |
|----|-----|------------------|
| FP1 | Information only flows forward | Lost at vision/chunk → unrecoverable by prompt |
| FP2 | Measure the bottleneck | `page_hit@k` + answer_in_evidence before changing fusion/prompts |
| FP4 | Honesty > Acc inflation | Do not ban “Not answerable” |
| FP7 | Code is law | Every claim cites a symbol that exists |

**One-line strategy:** Put chart/table numbers into page-faithful chunks; retrieve those pages; show page attribution to the LLM; refuse only when evidence is truly absent.

---

## 1. Ingest call graph (law)

```text
POST /api/v1/documents/pdf
  handlers/pdf_upload/upload.rs::upload_pdf_document
  → helpers.rs::create_pdf_processing_task
  → processor/pdf_processing.rs::process_pdf_processing   # SSOT worker
       → VisionPdfConverter | EdgeParsePdfConverter
            <!-- edgequake-page:N -->
       → multimodal/stage.rs::run_multimodal_analyze_stage  # no-op if !process_options
       → text_insert/prepare.rs  # ChunkStrategy::Pdf → PageAwareChunking
       → pipeline/processing.rs  # extract + embed
       → chunk_storage.rs        # KV + vector metadata: page_start, document_id
       → KnowledgeGraphMerger
```

### Irreversible losses (ingest order)

1. **Vision OCR** (`edgequake-pdf/.../vision.rs`) — pixels → lossy markdown; chart series often wrong/missing  
2. **Empty-page skip** (`vision.rs` — skip if `page.markdown.trim().is_empty()`) — figure-only pages never indexed  
3. **Multimodal analyze off** (`multimodal/analyzer.rs` — `!opts.any_enabled()`) — P0 upload sends **no** `process_options`  
4. **Page markers stripped from chunk body** (`page_aware.rs::split_into_page_segments`) — embeddings never see page numbers  
5. **Intra-page recursive split** — chart block split from caption/numbers  
6. **Text-only entity extraction** (`extractor/llm.rs`) — further compresses visuals  

**Smoke proof:** `answer_in_evidence_rate≈0.51` overall, **Chart≈0.36** → half of golds never land on evidence-page markdown.

---

## 2. Query call graph (law)

```text
POST /api/v1/query  {mode: hybrid}
  query_execute.rs → query_pipeline.rs
    prepare (keywords + embeddings)
    → hybrid.rs  local ∥ global ∥ naive   (Box::pin; intent gate via mix_weights)
    → hybrid_merge.rs  round-robin local→global→naive (or RRF)
    → postprocess: document_filter → rerank → balance_context → …
    → context.rs::to_context_string  # entities + rels + chunks; NO page_start
    → prompt.rs::generate_answer*
    → source_reference_builder + query_stats_mapper  # pages on HTTP sources only
```

### Irreversible losses (query order)

7. **Workspace-wide retrieve** (smoke: 10 docs, `document_scope=false`) — cross-doc dilution  
8. **Intent arm gate** (`mix_weights::intent_arm_mask`) — Factual → often **naive-only**  
9. **Round-robin fusion** — KG chunks can outrank naive page hits  
10. **Token balance** (`truncation::balance_context`) — entities/rels eat budget before chunks  
11. **Pages invisible to LLM** (`to_context_string`) — metadata exists but not in prompt  
12. **Strict grounding** (`prompt.rs`) — missing numerics → honest “Not answerable”

**Smoke proof:** `context_empty_rate=0` but false refusal ≈47%; **33/43** of those lack `page_hit@5`.

---

## 3. Ranked improvements (causal, anti-heuristic)

| # | Change | Why (physics) | Code anchors | Gate |
|---|--------|---------------|--------------|------|
| **1** | Chart/table second-pass at ingest (`process_options` + fidelity) | Chart Acc 0.05 is representation | `analyzer.rs`, `client.upload_pdf`, `fidelity.py` | answer_in_evidence Chart ↑ |
| **2** | Document-scoped retrieve in eval | Cross-doc dilution | `--document-scope`, `document_filter_resolver.rs` | `page_hit@5` ↑ |
| **3** | Inline `page_start` in prompt chunk headers | LLM can cite/ground pages | `context.rs::to_context_string` (W0c) | Acc ↑ when hit; unanswerable Acc held |
| **4** | Naive-first / RRF for factual-chart queries | Round-robin dilutes page hits | `hybrid_merge.rs`, `EDGEQUAKE_HYBRID_FUSION` | `page_hit@5` ↑ on chart slice |
| **5** | Keep empty vision pages as placeholders | Silent page dropout | `vision.rs` empty skip | page coverage ↑ |
| **6** | Don’t split table/chart blocks inside a page | Numeric evidence stays one chunk | `page_aware.rs`, `table_preprocessor.rs` | Chart fidelity ↑ |
| **7** | Ablate `P1_naive` vs `P0` | Isolate fusion tax | `profiles.py` | slice Acc delta |
| **8** | Cap graph token tax before chunk truncate | Chunks starved | `truncation.rs::balance_context` | mean chunk tokens ↑ |

### Reject

- Ban “Not answerable”  
- Feed gold `evidence_pages` into retrieve  
- Prompt-only Acc patches  
- Mid-run provider swaps  

---

## 4. Suggested experiment order

```text
1. bench047 fidelity          # confirm Chart answer_in_evidence
2. --document-scope query-only # W2: page_hit without re-ingest
3. P1_naive ablation           # fusion tax
4. process_options=ite re-ingest sample docs  # W1 representation
5. to_context_string page headers             # W0c / W3 assist
```

Each step: one locked profile · measure `page_hit@5` + Chart Acc + unanswerable Acc · fail closed on empty answers.

---

## 5. Research alignment (2025–2026)

Modern multimodal RAG treats **tables/charts as first-class index objects** (caption-and-index or page-as-image / ColPali). EdgeQuake today is **caption-and-index via vision markdown**, but P0 skips the second-pass structured extract (`process_options`). Closing that gap is the highest-leverage lawful move before late-interaction page embeddings.

Interactive board: canvas `spec047-pipeline-first-principles.canvas.tsx`.
