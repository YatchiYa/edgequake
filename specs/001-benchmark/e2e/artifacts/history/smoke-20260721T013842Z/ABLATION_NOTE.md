# Ablation — C1B_a1_rr_cer_bm25_all_v1

**Step:** c1b  
**Pins:** 059 c1b: A1 + RERANKER=bm25 (all intents; no CE) — latency peer; not Acc promote  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`

## Results

| Metric | Acc Fact T120315Z | C1a T012849Z | **C1b T013842Z** |
|--------|------------------:|-------------:|-----------------:|
| EQ Acc | **0.801** | 0.729 | 0.712 (tax) |
| EQ p50 | 6876 | 6299 | **5791** |
| EQ/LR p50 | 5.09× | 4.35× | **3.91×** |
| keyword p50 | (folded into embed) | (folded) | **1782** |
| embed p50 | ~2485 (mislabeled) | ~2495 | **2212** (pure) |
| rerank p50 | ~1100 CE | 1031 | **9** BM25 |
| generate p50 | 2316 | 2337 | 2421 |

## Verdict

- [x] Stage honesty: keyword ≠ embed (059).  
- [x] BM25-all removes CE (rerank ~9 ms).  
- [x] **Generate p50 (2421) > 1.5× LR (≈2221)** — SLO impossible without faster generate/keywords under Acc Mistral pins.  
- [x] Do **not** promote as Acc Fact peer.

## Next

- Product: faster KEYWORDS model / Fact heuristic; stream TTFT; optional local embed.  
- Acc Fact peer stays B5+a1fp.
