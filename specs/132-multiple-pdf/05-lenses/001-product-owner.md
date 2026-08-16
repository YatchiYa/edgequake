# Lens 001 — Product Owner

## Stake

Users must trust that multi-select works. “Stuck uploading” destroys confidence even when capacity is healthy. Separating #378 (admit) from #361 (slow) prevents shipping the wrong fix.

## Job to be done

Select N PDFs → every file admitted or clearly failed → processing may take time with honest queue language.

## Success metrics

| Metric | Gate |
|--------|------|
| Multi-PDF admit success (N=2) | Both rows + distinct task_ids |
| Hang rate on wake saturation | HTTP returns ≤ timeout; no silent forever |
| Support confusion | FAQ distinguishes upload vs processing |

## Non-goals

- Faster KG wall-clock (SPEC-122 Phase B/C)
- Marketing claims of unbounded parallel PDF convert

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
