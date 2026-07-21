# Ablation — A1FPM0

**Step:** a1fpm0 · `smoke-20260720T100538Z`  
**Pins:** a1fp + `MIN_RERANK_SCORE=0`

| Metric | Result |
|--------|--------|
| Acc | 0.753 ✗ (≤0.755 step) |
| ctx | 0.525 ✓ |
| recall | 0.914 ✗ (worse) |
| Fact ER | 0.80 ✗ (↓ from 0.85) |
| Sum ER | 0.86 flat |

**Verdict:** Reject — min_rerank Acc/Fact toxic; did not fix Summarize.
