# Ablation — A1FPCOV

**Step:** a1fpcov · `smoke-20260720T101322Z`  
**Pins:** a1fp + `COVERAGE_PROTECT_FIRST=30` (Exploratory)

| Metric | Result |
|--------|--------|
| Acc | 0.748 ✗ |
| ctx | 0.519 ✓ |
| recall | 0.916 ✗ |
| Fact ER | 0.80 ✗ |
| Sum ER | 0.86 flat |
| `0002d2de` parts | **6** (unchanged — Mix ceiling) |

**Verdict:** Reject — Exploratory protect=30 Acc-taxes; Sum ER unbound by CE membership when Mix pool is already small.
