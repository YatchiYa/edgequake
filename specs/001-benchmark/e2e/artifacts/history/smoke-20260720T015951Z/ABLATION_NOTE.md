# Ablation — P5_latency_arm24_v1

**Step:** p5  
**Pins:** Acc BM25 / `PATH_PRUNE=0` + `QUERY_ARM_CONCURRENCY=24`  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T015951Z`

## Results

| Metric | Value | Gate | Result |
|--------|-------|------|--------|
| EQ Acc | 0.721 | ~P0 class | tie CI |
| EQ / LR query p50 | 8392 / 1553 ms | — | — |
| EQ/LR p50 ratio | **5.404×** | ≤1.5× | **miss** |
| EQ stage p50 | embed=2471, retrieve=460, rerank=9, generate=2246 | waiver evidence | embed+generate dominate |
| query_arm_concurrency | 24 | labeled | ✅ |

## Verdict

- [ ] Gate met (≤1.5×)
- [x] Gate missed — **waiver with stages**: arm=24 does not close wall-clock gap; bottleneck is remote embed+generate (~4.7s of p50), not Mix arm fan-out alone

**Note:** Latency work continues outside Acc promotion (cache / batch embed / concurrency fairness). Acc headline unchanged (P0 pins).
