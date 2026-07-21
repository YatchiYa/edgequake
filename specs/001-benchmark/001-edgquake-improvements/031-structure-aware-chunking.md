# 031 — Structure-aware chunking / extract density (Horizon B3)

**Status:** B3a code shipped · labeled Acc run **Acc tax — no promote** · keep B2 WS  
**Cross-ref:** [030 B2](./030-ingest-gleaning-parity.md) · [029 Audit](./029-ingest-parity-audit.md) · [028 Roadmap](./028-first-principles-beat-roadmap.md) · LightRAG [Paragraph Semantic Chunking](https://github.com/HKUDS/LightRAG/blob/main/docs/ParagraphSemanticChunking.md)

---

## 1. Why B2 was not enough

B2 enabled markdown admission + gleaning=1 and produced the best Acc so far ([`T071732Z`](../e2e/artifacts/history/smoke-20260720T071732Z/): EQ Acc **0.785**, Complex Δ **−0.006**). L2 and density gates still fail:

| Signal | B2 WS `e0270f5f-…` | Target |
|--------|-------------------|--------|
| Soft-overlap | 0.640 | ≥0.75 |
| EQ nodes | 392 | ≪ LR 3580 |
| Zero-chunk | 11.2% | ≤5% |
| ctx_rel | 0.494 | ≥0.50 |
| evidence_recall | 0.928 | ≥ LR−0.03 (0.932) |

### First-principles finding (Acc medical corpus)

GraphRAG-Bench `medical.json` is a **single ~1.05M-char prose blob**:

- **0** markdown heading lines (`#`…`######`)
- **~44** newlines total (almost no paragraph breaks)
- FAQ cues (`What is …?`) are **inline**, not line-delimited

Therefore `ChunkStrategy::Markdown` ≈ recursive token split with an empty/preface section — **breadcrumbs never fire**. B2 Acc lift is attributable mainly to **gleaning=1** (+ packing variance), not section-context parity with LightRAG’s DOCX/PDF P-strategy path.

LightRAG’s Paragraph Semantic (P) strategy requires a `.blocks.jsonl` sidecar from structured parsers — **not applicable** to this Acc blob without a structure-induction front-end.

---


### B3a Acc result ([`T074835Z`](../e2e/artifacts/history/smoke-20260720T074835Z/))

| Signal | B3a | B2 A1 ([`T071732Z`](../e2e/artifacts/history/smoke-20260720T071732Z/)) |
|--------|------|------------------------------------------------------------------|
| EQ Acc | **0.663** (CI favors LR) | **0.785** |
| Chunks | 396 | ~188 |
| EQ nodes | 533 | 392 |
| soft-overlap | 0.636 | 0.640 |
| ctx / recall | 0.488 / 0.916 | 0.494 / 0.928 |

**Call:** Over-fragmentation from FAQ heading spam hurt Acc. Leave `STRUCTURE_INDUCE` off for Acc headline. **Do not pursue gated FAQ heuristics** — Acc density gap is graph **identity isolation** ([032](./032-workspace-graph-identity.md)), not missing headings.

## 2. B3 workstreams (one confound each)

| ID | Work | Hypothesis |
|----|------|------------|
| **B3a** | **Inline FAQ / topic induction** → synthetic `##` headings before markdown chunking (detect `What/How/Which…?` + title-case topic runs) | Restores `---Section Context---` on Acc prose; improves extract locality + ctx_rel |
| **B3b** | **Extract density parity** vs LR (max entities/rels per chunk, merge collapse, gleaning=2 labeled) | Close 392 vs 3580 surface-form gap; audit soft-overlap + zero-chunk |
| **B3c** | Optional: paragraph-boundary recursive (sentence/clause split) labeled vs frozen 1200 | Only if B3a alone does not move L2 |

Do **not** port full LR P-strategy (table row split / sidecar) for Acc medical — wrong input shape. Keep P-strategy as a product track for PDF/DOCX.

---

## 3. Experiment protocol

1. New workspace (never silent overwrite of pre-B2 `8b359190-…`; B2 `e0270f5f-…` is the current Acc candidate).  
2. Pins: chunk 1200/100 · adaptive off · gleaning=1 · **+ B3a induction** (label profile).  
3. Audit (`audit_eq_lr_ingest.py`) then A1 query-only (`rr_cer`), concurrency≤4 to avoid pool timeouts.  
4. Promote only if Parity/Beat gates (028) pass.

```bash
# Requires rebuilt release binary (structure_induce in MarkdownChunking)
cargo build --release --bin edgequake
unset BENCH001_EQ_WORKSPACE_ID
export BENCH001_CHUNK_STRATEGY=markdown
export BENCH001_ENABLE_GLEANING=1
export EDGEQUAKE_STRUCTURE_INDUCE=faq
export BENCH001_ACC_QUERY_CONCURRENCY=4
make bench001-b3-reingest
```

**Code:** `edgequake-pipeline/src/structure_induce.rs` · hook in `markdown_chunking.rs` · pin `structure_induce` in `fair_pins.py`.

**Success:** ctx≥0.50 ∧ recall≥LR−0.03 ∧ Acc ≥ B2 A1−0.01 (0.775) ∧ soft-overlap ≥0.70 **or** zero-chunk ≤5%.

---

## 4. Non-goals

- Soft Mix Acc fishing on the query ladder  
- Claiming Beat from entity-count alone  
- Full Docling/P-strategy port for Acc medical text  
