# 06 — Root Cause & Recommendations

## Ranking

| Rank | Cause | Effect on this PDF/gold |
|-----:|-------|-------------------------|
| 1 | **Product adaptive chunking** (61 KB text → **800**/~66) | +33% chunks vs fair; +31–34% unique entities vs fair/LR |
| 2 | **Metric confusion (M vs U)** — SPEC-108 | UI mention sums look “huge” vs LR unique graph |
| 3 | **Strategy drift** (EQ Recursive/Pdf vs LR F) | Small N delta at same size (12 vs 13); secondary |
| 4 | True prompt over-extract under fair pins | **Not observed** (U_A ≈ U_C) |

## Causal chain

```ascii
  papers/light_rag_*.pdf
       │ parse → ~61KB markdown
       ▼
  EQ adaptive ON ──────────────► chunk_size=800, N≈16
       │                          M≈584  U≈491
       │
  EQ fair / LightRAG paper pin ► chunk_size=1200, N≈12–13
                                  M≈425–439  U≈367–375
```

## Recommendations (product)

| Goal | Action |
|------|--------|
| Match LightRAG density on academic PDFs | Workspace/env: `EDGEQUAKE_ADAPTIVE_CHUNKING=0`, `EDGEQUAKE_CHUNK_SIZE=1200`, `EDGEQUAKE_CHUNK_OVERLAP=100` |
| Keep adaptive for mega-docs | Leave ON; surface **U** and ents/1k chars on document card (SPEC-108 / 086) |
| PDF path fairness | When comparing to LR F, pin strategy Fixed/Recursive explicitly — auto-`Pdf` is a confound (LAW-C6) |
| Do not “fix” extract prompts first | Geometry + fair pin already explain the gap on this paper |

## Follow-ups (out of this pack’s code scope)

1. Optional product default: disable adaptive above a quality SLA, or raise adaptive floor for &lt;100 KB born-digital papers to 1200.
2. Arm D: same gold text with EQ `ChunkStrategy::Pdf` vs LR F at fixed 1200.
3. Re-run HTTP/AGE path when Postgres/Docker available for exact AGE U.

## Decision

**No emergency extractor rewrite.** The live Mistral dual-SUT shows EdgeQuake under fair pins is LightRAG-parity on unique graph size; product adaptive sizing is the primary lever for “too many chunks / entities.”
