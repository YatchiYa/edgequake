# 031 — Acc #3 dense-scalar final assessment

**Artifact:** `specs/047-rag-evaluation/e2e/artifacts/smoke-chart8-026-dense-scalar-20260715-1845/`  
**Build:** `20260715.103802` · Tag: `chart8-026-dense-scalar-20260715-1845`  
**Lever:** W1-dense-scalar callout prompts (sample sizes / N= / year labels) on top of Acc #2 stack  
**Protocol:** `026-listmem-2026-07-15`  
**Baseline:** Acc #2 fig-as-chart `…-1707` (listmem re-audit)

---

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  DENSIFY CALLOUT PROMPTS DID NOT MOVE CHART REPRESENTATION.                  │
│                                                                              │
│  Chart a_in_e_long: 0.571 PASS  (IDENTICAL to Acc #2 listmem; 0 flips)       │
│  Table a_in_e_long: 0.647 PASS  (+0.059 vs Acc #2 — modest Table gain)       │
│                                                                              │
│  Acc 0.545  (Δ=-0.017 vs Acc #2)   F1 0.408  (Δ=-0.072)  ← REGRESS           │
│  ChartEx Acc 0.286  (flat)         Chart ML Acc 0.182  (Δ=-0.136)            │
│                                                                              │
│  Mid-run political MD: 541/128 still ABSENT (prompt ≠ printed pixels).       │
│  Same 6 true Chart long misses remain.                                       │
│                                                                              │
│  Honest claim: keep Acc #2 as Acc/F1 SOTA; gates stay PASS under listmem.    │
│  Do NOT ship densify callout prompts as a W1 Acc win. Roll back optional.    │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Scoreboard

| Metric | Acc #2 figas | **Acc #3 densify** | Δ |
|---|---:|---:|---:|
| Acc | 0.562 | **0.545** | −0.017 |
| F1 | 0.480 | **0.408** | −0.072 |
| ChartEx Acc | 0.286 | 0.286 | 0 |
| Chart ML Acc | 0.318 | **0.182** | −0.136 |
| Chart `a_in_e_long` | **0.571 PASS** | **0.571 PASS** | 0 |
| Table `a_in_e_long` | 0.588 PASS | **0.647 PASS** | +0.059 |
| Aggregate a_in_e_long | 0.660 | **0.702** | +0.042 |

Paired Acc Δ vs Acc #2: `other_answerable −0.034` · `unanswerable +0.026` · `list_gold −0.009`

---

## Mid-run → final causal chain

1. **Ops:** Bare nohup smoke clients died mid-pending; Cursor-managed bg succeeded. Political wr=12 / promoted=12 confirmed under densify binary.
2. **Political MD mid-check:** `541`/`128` count=0 after densify specialize (Key values×29 present — but not gold absolute counts).
3. **Fidelity:** Chart long hit/miss set **bit-identical** to Acc #2 listmem (0 flips). Remaining misses unchanged: `541`, `128`, `4087`, `1251`, `198` (wrong page), pie years incomplete.
4. **Acc regress:** Chart multi flips dominated by **list extract noise** (e.g. `XL`→`XL Axiata`, `["3","Esia","Smartfren"]`→`["3 Indonesia",…]`) — W4/format, not densify representation.

---

## First-principles conclusion

| Hypothesis | Result |
|---|---|
| Prompt “include sample sizes / years” → gold scalars in MD | **FAIL** — absolutes still absent |
| Chart gate already PASS from listmem; densify lifts Acc | **FAIL** — Acc/F1 down |
| Densify harms list extract stability | **SUSPECT** — Chart ML Acc −0.136 from format mismatches |

**Law reminder:** fail-closed specialize correctly omits unreadables. If 541 is not printed on the crop (only %), densify cannot invent it. Next representation lever must change **pixels/crop targeting** (gold-page region / multi-crop / page-image specialize), not prompt adjectives.

---

## Recommendation

1. **Keep Acc #2 (`…-1707`) as Acc/F1 reference**; Acc #3 is a densify negative result.
2. **Gates remain PASS** under `026-listmem` — measurement honesty stands; do not re-claim from Acc #3 Acc.
3. **Next lever (product):** gold-page residual ranking / multi-region chart crops for numeric callout regions — only if OCR/VLM can see the digit; else accept those Chart longs as unanswerable from current vision channel.
4. Optional: revert densify callout prompt text if it correlates with list-extract noise (low confidence); or keep (harmless for fidelity gates).

---

## Artifacts

- Smoke: `e2e/artifacts/smoke-chart8-026-dense-scalar-20260715-1845/`
- Compare vs Acc #2: `COMPARE_vs_figas_acc2.md`
- Mid logs: `logs/2026-07-15-18-48-beastmode-acc3-mid-assess-1.md`, `…-18-52-…-mid-assess-2.md`
