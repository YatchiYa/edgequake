# 029 — Post Acc #2 First-Principles Plan Update + W1-measure

**Date:** 2026-07-15  
**Prior Acc #2:** [028](./028-fig-as-chart-acc2-assessment.md) · artifact `smoke-chart8-026-fig-as-chart-20260715-1707`  
**Parent plan:** [026](./026-first-principles-score-improvement-brainstorm.md)

---

## 1. One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  FIG-AS-CHART ↑ crop coverage. Chart a_in_e_long STUCK at 0.214.              │
│                                                                              │
│  Binding cause (FP2 measure): fidelity needles the *serialized list string*  │
│  and keeps quote chars — false negatives when every list member (or MMMU)    │
│  is already on the evidence page.                                            │
│                                                                              │
│  Sim on Acc #2 Chart long (n=14):                                            │
│    whole-string hits = 3 → 0.214                                             │
│    + list-member hits = +4 → 0.500  (gate threshold)                         │
│    + quote-strip MMMU = +1 → ~0.571  (likely PASS)                           │
│    remaining TRUE misses = 6–7 scalars (541,128,4087,1251,198,pie years)     │
│                                                                              │
│  Law: MMLongBench scores lists per-element (greedy avg). Fidelity must      │
│  mirror that — all members present ⇔ representation OK for list gold.       │
│                                                                              │
│  Next execute (ordered):                                                     │
│    W1-measure-listmem  → re-audit Acc #2 (no new Acc)                        │
│    W1-dense-scalar     → densify specialize for remaining numeric Chart miss │
│    Acc #3              → only after Chart long moves on true misses too      │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Causal chain (info flow)

```text
PDF → Pass A → crop/promote → specialize → page MD facts
                                              │
                     ┌────────────────────────┴────────────────────────┐
                     │ MEASURE (a_in_e_long)                            │ ACC soft-score
                     │ was: whole list / quoted string as one needle   │ lists: per-element
                     │ bug: false MISS when members on page            │
                     └─────────────────────────────────────────────────┘
```

| Lever | Status | Effect on Chart long |
|---|---|---|
| W1-coexist | landed | plumbing when wr>0 |
| W1-fig-as-chart | Acc #2 measured | wr↑; **0** Chart long flips |
| W4-extract MMMU | landed | ChartEx Acc 0.143→0.286; fidelity still miss on quotes |
| **W1-measure-listmem** | **this wave** | honesty: list + quotes |
| W1-dense-scalar | next if needed | true numeric misses |

---

## 3. Honesty rules

1. Re-auditing Acc #2 after measure fix ≠ new Acc run. Acc/F1 stay 0.562/0.480.
2. Gate PASS from listmem is **measurement honesty**, not new representation. Document as such.
3. Wave 1 *product* claim still needs ChartEx Acc attributable to representation + gate; Acc #2 ChartEx↑ was W4 extract.
4. Do **not** loosen gate threshold; fix containment to match MMLongBench list physics.

---

## 4. Execute checklist

- [x] Update 026 status header
- [x] Implement `answer_in_text` list-all-members + quote strip; bump protocol note
- [x] Unit tests (list hit/miss, quoted MMMU, scalar unchanged) — 12 passed
- [x] Re-run `bench047 fidelity` on Acc #2 artifact / live workspace
- [x] Write assessment ([030](./030-w1-measure-listmem-assessment.md)) — Chart **0.571 PASS**
- [ ] Acc #3 only after densify lands true-miss flips (W1-dense-scalar)
