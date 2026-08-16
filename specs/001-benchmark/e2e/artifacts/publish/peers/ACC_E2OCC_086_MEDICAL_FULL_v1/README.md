# Labeled publish peer — `ACC_E2OCC_086_MEDICAL_FULL_v1`

**Not Acc headline.** Acc SSOT remains [`publish/latest/`](../../latest/) (`medical-mid-20260815T110218Z`).

## Configuration (best known Acc pack)

Acc law **E2-occ 086** on frozen full-corpus workspace `23b09c73-…` (query-only, no re-ingest). **Chunk 1200 / overlap 100** (LightRAG CHUNK_SIZE parity — Acc ingest pin).

- chunk **1200/100** · adaptive off · extract 40/100+fifo
- Mix `round_robin` · rerank off · `bfs` · occurrence_sort · LR VECTOR budget
- Fact L2 `fact_replace` · `L2_FACT_BM25_POOL=mix` · `GRAPH_WALK_COMPRESS=0` · `LLM_CACHE=0`

## Result (`medical-full-20260816T012004Z`, n=2062, valid)

| Metric | EQ | LR | vs Beat (080) |
|--------|-----|-----|----------------|
| Acc | **0.7857** | **0.7855** | point Δ +0.0001 · CI [-0.160, +0.047] includes 0 (paired n=16 underpowered) |
| ctx_rel | 0.427 | 0.485 | **FAIL** (&lt;0.50) |
| overall ER | 0.927 | 0.947 | PASS (≥ LR−0.03) |
| Fact ER | 0.914 | 0.945 | **at** LR−0.03 (0.9145) · not a clear Beat pass |
| EQ query p50 | 3784 ms | 878 ms | 4.31× (Acc cold vs LR cache) |

**Vs prior P0 medical-full (`20260722T204100Z`):** Acc 0.724 → **0.786** (closed the 6pp scale gap; now a point tie). ctx 0.394 → 0.427 (still fail). Fact ER 0.905 → 0.914.

**Verdict:** Scale check KEEP as labeled Acc-law full peer. **Not Beat** (ctx unmet; Acc CI not EQ-ahead). Do **not** replace `publish/latest`.

## Artifacts

- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt)
- [SUMMARY.md](./SUMMARY.md)
- [scorecard.json](./scorecard.json)
