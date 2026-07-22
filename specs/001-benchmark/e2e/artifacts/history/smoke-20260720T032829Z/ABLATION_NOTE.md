# Ablation — T0_p2b_fact_bm25_v1

**Step:** t0  
**Pins:** 027 T0: P2b + FACT_RERANKER=bm25 on prompt  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T032829Z`

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Fact ER | ≥0.90 | **0.90** (was 0.85) |
| Acc floor | ≥0.736 | **0.720 MISS** |
| ctx_rel | ≥0.50 | **0.494 MISS** |
| recall | ≥ LR−0.03 (0.941) | **0.934 MISS** |
| Δ Acc CI | Beat/Parity | includes 0 |

## Verdict

- [x] Gate missed (do not promote) — Acc tax from BM25 prompt  
- Next: **T0b** L2-only BM25∪CE (`make bench001-t0b`)
