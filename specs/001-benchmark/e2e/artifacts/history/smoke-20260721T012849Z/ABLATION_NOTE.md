# Ablation — C1A_a1_rr_cer_fact_ce_skip_v1

**Step:** c1a  
**Pins:** 058 c1a: A1 + FACT_CE_SKIP=1 + FACT_RERANKER=bm25 — latency peer; not Acc promote  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce` (B5 Acc Fact WS)

## Results

| Metric | Acc Fact peer T120315Z | C1a T012849Z |
|--------|------------------------:|-------------:|
| EQ Acc | **0.801** | 0.729 (tax) |
| ctx_rel | 0.519 | 0.494 |
| recall | 0.926 | 0.930 |
| EQ p50 | 6876 | **6299** |
| EQ/LR p50 | 5.09× | **4.35×** |
| rerank p50 (all) | 1100 | 1031 |
| rerank p50 (intent=factual, n=17) | ~CE | **9 ms** |
| rerank p50 (other, n=23) | ~CE | 1136 |

## Verdict

- [x] **C1a law works** — Fact→BM25 skips CE (~9 ms vs ~1.1 s).  
- [x] Overall p50 improves modestly (5.09→4.35×); **still fails ≤1.5×** (embed+generate dominate).  
- [x] **Do not promote** as Acc Fact peer (Acc tax; ctx &lt; 0.50).  
- Prior `T012604Z` invalid (FACT_CE_SKIP not forwarded to Acc backend) — discarded.

## Next

- C1b: match EQ↔LR concurrency / broader CE policy for product.  
- C1c code (query_vec reuse) already shipped; Acc stage remeasure optional.
