# 034 — Acc #4 W3-arith final assessment

**Artifact:** `specs/047-rag-evaluation/e2e/artifacts/smoke-chart8-026-w3-arith-20260715-2012/`  
**Build:** `20260715.121257` · Tag: `chart8-026-w3-arith-20260715-2012`  
**Lever:** W3-arith Gen carve-out + year-span (Pass A + fidelity); Acc #3 densify **reverted**  
**Protocol:** `026-listmem-2026-07-15`  
**Baseline:** Acc #2 fig-as-chart `…-1707`

---

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Acc #4 = MEASURE WIN on Chart long; Acc/F1 REGRESS vs Acc #2.               │
│                                                                              │
│  Acc  0.545  (Acc #2 0.562 · Acc #3 0.545)  — no Acc lift from W3-arith      │
│  F1   0.429  (Acc #2 0.480 · Acc #3 0.408)  — between Acc2/Acc3              │
│  ChartEx Acc 0.286  — FLAT vs Acc #2                                         │
│  Chart a_in_e_long 0.643 PASS  (was 0.571) — year-span hit afe620 years      │
│                                                                              │
│  Derived counts still wrong:                                                 │
│    541 → pred 872 (Acc2: 58)   · 128 → NA  · 1251 → 82%  · 4087 → 73%      │
│  Prompt MAY-compute was too weak; model returns % or invents.                │
│                                                                              │
│  Honesty: year-span Chart long ↑ is measure/Pass-A honesty, NOT Acc product. │
│  Keep Acc #2 as Acc/F1 SOTA. Next: W3-arith-v2 (MUST + worked example)       │
│  via query-only Acc #5 on Acc #4 workspace.                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Score ladder (same smoke)

| Run | Acc | F1 | ChartEx | Chart `a_in_e_long` |
|---|---:|---:|---:|---:|
| Acc #2 fig-as-chart | **0.562** | **0.480** | 0.286 | 0.571 PASS |
| Acc #3 densify | 0.545 | 0.408 | 0.286 | 0.571 PASS |
| **Acc #4 W3-arith** | 0.545 | 0.429 | 0.286 | **0.643 PASS** |

Flips vs Acc #2: +8 / −10 / same 99 — noise + false refusals (0.253), not derived-count wins.

---

## First-principles read

1. Year-span expand ✅ for gate honesty (page already had `1981-82`).
2. Soft W3-arith ❌ — Gen still answers percentage (`82`, `73`) or refuses (`128`) or wrong arithmetic (`872`).
3. Acc↑ requires stronger Gen composition (MUST + example) or a deterministic %×N tool — not more densify.

**Do not claim Wave 1 product Acc win.** Acc #2 remains the Acc/F1 reference.
