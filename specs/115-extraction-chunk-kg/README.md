# SPEC-115 — PDF Extraction Chunk Size & KG Density (EQ ↔ LightRAG)

> **Mission:** Deep study of **chunk geometry** and **entity/relation yield** on the LightRAG paper PDF, with **live Mistral Small** dual-SUT execution.  
> **Code is law.** Cross-ref SPEC-108 (M vs U vanity) and SPEC-026 (full product audit) — this pack owns **live PDF-paper protocol + measured arms**.

## Partner / product question

EdgeQuake appears to extract **too many chunks** and therefore **too many entities / relations** vs LightRAG on PDF documents.

## Short verdict (live 2026-08-10, Mistral Small)

| Axis | LightRAG (Arm C) | EdgeQuake product (Arm B) | EdgeQuake fair (Arm A) |
|------|------------------|---------------------------|-------------------------|
| Chunk size | Fixed **1200**/100 (F) | Adaptive **800**/66 on ~61 KB text | Fixed **1200**/100 |
| Chunk count N | **13** | **16** (~1.33× fair) | **12** |
| Unique nodes U | **367** | **491** (~1.34× LR) | **375** (~1.02× LR) |
| Unique edges | **318** | **305** | **320** |
| Mentions M (ents) | ~425 | **584** | 439 |

**Conclusion:** Product looks denser because adaptive shrinks chunks. Fair-pinned EQ ≈ LightRAG unique graph on this paper. Details: [measurements/SUMMARY.md](measurements/SUMMARY.md).

## Document map

```ascii
 00-why.md
   → 01-first-principles.md          (LAW-C1..C6 + Aug 2026 AI eng)
   → 02-cross-ref-matrix.md          (code + specs + papers)
   → 03-code-comparison.md           (chunk + extract SSOT)
   → 04-execution-protocol.md        (arms A/B/C/D, pass rules)
   → 05-execution-report.md          (measured results)
   → 06-root-cause-and-reco.md
   → measurements/                   (geometry + live artifacts)
   → experiments/                    (repro scripts)
```

## Sample (binding)

| Role | Path |
|------|------|
| PDF (user) | `papers/light_rag_2410.05779v3.pdf` (~1.12 MB, 16 pages) |
| Gold MD twin | `zz_test_docs/academic_papers/lighrag_2410.05779v3.pymupdf.gold.md` (~61 KB, ~14 156 tiktoken) |
| PDF copy in fixtures | `zz_test_docs/academic_papers/lighrag_2410.05779v3.pdf` |

**Why gold MD for extract arms:** isolates PDF-parser variance so chunk size + extract density are comparable. PDF path (EQ `ChunkStrategy::Pdf`, LR native/mineru) is a **separate confound** documented in protocol Arm D.

## Cross-spec anchors (DRY)

| Spec | Job of that pack | What 115 adds |
|------|------------------|---------------|
| [SPEC-108](../108-extraction-compared-light-rag/) | M vs U vanity + geometry mock | **Live Mistral** on this PDF/gold |
| [SPEC-026](../026-egdequake-vs-lightrag/) | Full EQ↔LR audit | Chunk/KG density only |
| [SPEC-001 / 054](../001-benchmark/) | Acc fair pins, extract caps 40/100 | Reuse pins; do not change Acc |
| [SPEC-025](../010-ingestion-reliability/) | Adaptive chunking product | Measure product vs fair |

## Status board

| ID | Hypothesis | Status |
|----|------------|--------|
| H-C1 | Product adaptive shrinks chunks → higher N | **Confirmed** (16 vs 12 live; 20 vs 13 F geometry) |
| H-C2 | Higher N → higher mention M | **Confirmed** (M ratio = N ratio) |
| H-C3 | Fair EQ U ≈ LightRAG U (same model) | **Confirmed** (375 vs 367) |
| H-C4 | PDF strategy ≠ F strategy even at same size | Deferred (gold MD used Recursive) |
| H-C5 | True over-extract under fair pins | **Rejected** on this sample |

## Reproduce (quick)

```bash
# Geometry (no LLM) — real LightRAG F chunker
python3 specs/115-extraction-chunk-kg/experiments/geometry_probe.py

# Live LightRAG + Mistral Small (needs MISTRAL_API_KEY)
python3 specs/115-extraction-chunk-kg/experiments/run_lightrag_mistral.py

# Live EdgeQuake product + fair arms (needs MISTRAL_API_KEY + postgres)
python3 specs/115-extraction-chunk-kg/experiments/run_edgequake_mistral.py
```
