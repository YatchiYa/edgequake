# SPEC-115 measurements SUMMARY

**Date:** 2026-08-10  
**LLM:** `mistral-small-latest` + `mistral-embed` (1024-d)  
**Sample:** LightRAG paper gold MD twin of `papers/light_rag_2410.05779v3.pdf` (~61 KB / 14 156 tiktoken)

## Chunk geometry

| Source | Pin | Strategy | N |
|--------|-----|----------|--:|
| LightRAG F chunker | 1200/100 | F | **13** |
| LightRAG F chunker | 800/66 | F | **20** |
| EQ live Arm A (fair) | 1200/100 | Recursive | **12** |
| EQ live Arm B (product) | 800/66 adaptive | Recursive | **16** |

## Live KG yield (same model)

| Arm | SUT | N | M ents | M rels | U nodes | U edges |
|-----|-----|--:|-------:|-------:|--------:|--------:|
| **C** | LightRAG | 13 | ~425† | ~342† | **367** | **318** |
| **A** | EQ fair | 12 | 439 | 325 | **375** | **320** |
| **B** | EQ product | 16 | **584** | 322 | **491** | 305 |

† LightRAG mentions summed from per-chunk extract log lines (pre-merge). U from graph after merge.

## Ratios (signal)

```text
N_B / N_A     = 16/12 = 1.33
M_B / M_A     = 584/439 = 1.33   ← tracks N (H-C2)
U_B / U_A     = 491/375 = 1.31
U_A / U_C     = 375/367 = 1.02   ← fair EQ ≈ LightRAG (not H-C5)
U_B / U_C     = 491/367 = 1.34   ← product denser vs LR by adaptive geometry
```

## Verdict line

Product EdgeQuake looks “too dense” mainly because **adaptive chunking shrinks to 800** on this ~61 KB text → **more chunks → more mentions/uniques**. Under **fair 1200/100**, unique graph size matches LightRAG within ~2% on this paper with Mistral Small.
