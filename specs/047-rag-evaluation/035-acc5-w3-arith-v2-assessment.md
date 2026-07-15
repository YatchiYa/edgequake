# 035 — Acc #5 W3-arith-v2 assessment (query-only)

**Artifact:** `specs/047-rag-evaluation/e2e/artifacts/smoke-chart8-026-w3-arith-v2-20260715-2126/`  
**Lever:** W3-arith-v2 — `MUST compute` + worked example `36%×1503→541`  
**Protocol:** query-only on Acc #4 workspace `95ec2c18-…` (Gen-only causal change)  
**Baselines:** Acc #2 `…-1707` · Acc #4 `…-2012`

---

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  W3-arith-v2 PARTIAL PRODUCT WIN on Gen composition.                         │
│                                                                              │
│  Acc  0.562  ≈ Acc #2 SOTA (0.562) · ↑ from Acc #4 0.545                     │
│  F1   0.457  < Acc #2 0.480 · ↑ from Acc #4 0.429                            │
│  ChartEx Acc 0.286  — FLAT                                                   │
│  Chart a_in_e_long 0.643 PASS  — unchanged (measure; query-only)             │
│                                                                              │
│  Derived counts:                                                             │
│    1251: Acc #4 `82` → Acc #5 `1251` SCORE 1.0  ✅ W3 landed                 │
│    541:  still `872` (wrong operands / arith)                                │
│    4087: `73` → `1114` (attempted count, still wrong)                        │
│    128 / years: still Not answerable (operands not both in Context)          │
│                                                                              │
│  Honesty: Acc↑ here is Gen composition, NOT W1 Chart representation.         │
│  Acc #2 remains Acc/F1 reference until F1 also clears 0.480.                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Score ladder

| Run              | Acc       | F1        | ChartEx | Chart long | Note                 |
| ------------------| ----------:| ----------:| --------:| -----------:| ----------------------|
| Acc #2           | **0.562** | **0.480** | 0.286   | 0.571      | Acc/F1 SOTA          |
| Acc #3 densify   | 0.545     | 0.408     | 0.286   | 0.571      | negative             |
| Acc #4 W3 soft   | 0.545     | 0.429     | 0.286   | **0.643**  | year-span measure    |
| **Acc #5 W3-v2** | **0.562** | 0.457     | 0.286   | 0.643      | **1251 hit**; Acc≈#2 |

---

## First-principles next

1. **Operand retrieval** for 541/128 — if Context lacks both % and N, prompt cannot help (W2 / page routing).  
2. **Wrong-arith guard** — 872 / 1114 show model attempts math with wrong pairing; few-shot alone insufficient → consider deterministic %×N when both operands extracted.  
3. **Years** — fidelity HIT but Gen still NA → quote-from-context / list expand at Gen (W3-quote), not more densify.  
4. Do **not** claim Wave 1 Acc product win from Acc #5.
