# Ablation — C1D_a1_rr_cer_bm25_heuristic_kw_v1

**Step:** c1d  
**Pins:** 060 c1d: A1 + BM25-all + KEYWORD_MODE=heuristic — latency peer; not Acc promote  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`

## Results

| Metric | C1b T013842Z | **C1d T014632Z** |
|--------|-------------:|-----------------:|
| EQ Acc | 0.712 | 0.736 |
| EQ p50 | 5791 | 5995 |
| EQ/LR p50 | 3.91× | **4.08×** |
| keyword p50 | 1782 | **0** |
| embed p50 | 2212 | 2180 |
| retrieve p50 | 539 | 983 |
| rerank p50 | 9 | 9 |
| generate p50 | 2421 | **2985** |

## Verdict

- [x] **KEYWORD_MODE=heuristic works** — keyword stage p50 = 0.  
- [x] Overall p50 **did not improve** (generate↑ / retrieve↑ variance) — generate still the ceiling.  
- [x] Do **not** promote as Acc Fact peer; do **not** claim latency win from keyword-zero alone.  
- Acc Fact peer stays B5+a1fp.

## Next (product)

- Fast KEYWORD **LLM** (nano / local) that preserves Mix keywords — not Acc Soft Mix.  
- Generate/TTFT path (stream, smaller answer model).
