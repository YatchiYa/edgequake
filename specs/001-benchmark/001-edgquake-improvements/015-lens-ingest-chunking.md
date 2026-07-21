# 015 — Lens: Ingest & Chunking

**Priority:** Ingest quality track (Acc pin frozen at 1200/100)  
**Cross-ref:** [005 Pins](../005-mode-map-and-pins.md) · pipeline adaptive chunking

---

## 1. Observation

Publication Acc forces:

- `EDGEQUAKE_CHUNK_SIZE=1200`, overlap 100  
- `EDGEQUAKE_ADAPTIVE_CHUNKING=0`  
- Full medical corpus (~1.05M chars), uncapped  

Earlier capped “full” runs (100k chars) produced Acc ~0.43 and ctx_rel ~0.08 — **invalid for publication**. Ingest completeness is a validity precondition, not an optimization knob for headline Acc.

---

## 2. First-principles diagnosis

- Retriever can only return what the chunker produced (chunk boundary = retrieval atom).
- Fair Acc compares systems under **matched** chunk policy; changing EQ chunking alone breaks peer comparison unless LR matches.
- Production may use adaptive / contextual embeddings; Acc must label any deviation.

---

## 3. July 2026 practice

| Practice | Acc headline | Product / research |
|----------|--------------|--------------------|
| Fixed 512–1024 (or paper 1200) + 10–15% overlap | **Keep 1200/100** | OK |
| Structure-aware / markdown-aware splits | Ablation only if LR matched | Preferred for PDFs/docs |
| **Contextual embeddings** (50–100 token situating prefix before embed) | Labeled profile | High-ROI 2024–2026 practice (~49% fewer retrieval fails; +rerank ~67%) |
| Adaptive chunking (EQ pipeline) | Off for Acc | On for product default |
| Incremental / delta index | Ops | Required at scale |

Anthropic contextual retrieval (2024) remains underused relative to ROI in 2026 production writeups.

---

## 4. EQ insertion points

| Area | File / module | Action |
|------|---------------|--------|
| Adaptive chunking | `edgequake-pipeline/src/adaptive_chunking.rs`, `chunker/registry.rs` | Acc: force off; product: tune per doc type |
| Ingestion pipeline | `edgequake-pipeline/src/ingestion_pipeline.rs` | Hook for contextual prefix generation before embed |
| Admission / corpus caps | `edgequake-api/.../document_admission.rs` | Acc: `INGEST_MAX_CHARS=0` (uncapped); never bleed shell 100k caps into publish |
| Chunk size env | `EDGEQUAKE_CHUNK_SIZE`, overlap | Match LR for fair Acc |

---

## 5. Experiments (one confound each)

| # | Change | Success |
|---|--------|---------|
| I1 | Contextual embed prefix (cheap LLM) on EQ only | Labeled `P_ctx_embed`; ctx_rel↑ ≥ 0.05 vs baseline; Acc not↓ ≥ 0.02 — **not** headline until LR also contextualized or claim scoped “EQ ingest upgrade” |
| I2 | Adaptive on vs off (matched LR policy if comparing) | Report both; Acc headline stays adaptive off |
| I3 | Structure-aware markdown chunker | Medical table/section recall↑ on diagnostic subset |
| I4 | Pin hygiene regression test | `make bench001-full` doctor fails on adaptive=on or char cap > 0 |

---

## 6. Non-goals

- Do not change Acc chunk size to “win” without matching LightRAG.
- Do not re-enable adaptive chunking silently in Acc backends.
- Do not treat PDF/vision pin drift as a chunking win (lineage must stay mistral-small for publish Acc).
