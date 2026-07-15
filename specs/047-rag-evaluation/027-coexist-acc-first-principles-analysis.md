# 026 — Coexist Acc Result: First-Principles Analysis

**Artifact:** `specs/047-rag-evaluation/e2e/artifacts/smoke-chart8-026-coexist-20260715-1547/`  
**Build:** `20260715.074054` (W1-coexist only — fig-as-chart **not** in this binary)  
**Protocol:** `026-hardened-2026-07-15` · profile `P0_mm_ite` · n=117 · gateable=True  

---

## 1. One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Acc/F1 ROSE. Wave-1 Chart GATE DID NOT.                                     │
│                                                                              │
│  Acc 0.563  (+0.057 vs crop-expand · +0.064 vs dense)                        │
│  F1  0.457  (+0.054 vs crop-expand · +0.083 vs dense)                        │
│                                                                              │
│  Chart a_in_e_long = 0.214  FAIL  (IDENTICAL to dense + crop-expand)         │
│  Table a_in_e_long = 0.353  FAIL  (worse than crop 0.412)                    │
│  Chart exclusive Acc = 0.143  (↓ from crop 0.286)                            │
│                                                                              │
│  Honest claim: coexist is a necessary plumbing fix (charts indexable when    │
│  wr>0). It is NOT sufficient for Wave 1. Binding remaining causes:           │
│    (A) ink=0 / chart-is-fig → wr=0 (political, some full-bleed)              │
│    (B) gold-page ∩ residual-write often empty (e.g. 2311 gold 4,8)           │
│    (C) specialize still does not put gold needles into page markdown         │
│    (D) Acc↑ mostly list_gold + unanswerable, not Chart representation        │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Do not claim Wave 1 exit. Do claim coexistence works as designed where crops exist.**

---

## 2. Scoreboard (same protocol, same fixture)

| Metric | Dense | Crop-expand | **Coexist** | Δ vs crop |
|---|---:|---:|---:|---:|
| Acc | 0.500 | 0.506 | **0.563** | **+0.057** |
| F1 | 0.374 | 0.403 | **0.457** | **+0.054** |
| Chart ML Acc | 0.182 | 0.273 | **0.318** | +0.045 |
| Chart exclusive Acc | — / 0.143\* | 0.286 | **0.143** | **−0.143** |
| Chart **a_in_e_long** | 0.214 | **0.214** | **0.214** | **0** |
| Table a_in_e_long | — | 0.412 | **0.353** | −0.059 |
| Aggregate a_in_e_long | — | 0.404 | **0.383** | −0.021 |
| page_hit@5 | 0.747 | 0.747 | **0.773** | +0.026 |

\*Dense exclusive Chart Acc was 0.143 in prior notes; crop’s exclusive bump was the MMMU unwrap.

Protocol law: **Acc ↑ ≠ W1 win**. Gate requires Chart `a_in_e_long ≥ 0.50` and Chart exclusive Acc ↑ attributable to representation.

---

## 3. First-principles map (what each lever can/cannot do)

Info flow (binding law):

```text
PDF → Pass A vision MD → residual chart crop? → <drawing chart> → Pass B specialize
    → indexable numbers on page N → retrieve page N → Gen quotes → Acc soft score
```

| Cause | What we observed this run | Lever |
|---|---|---|
| Fig wins over chart in assemble/inject | Fixed: 2311 has chart hrefs on 12/23/25/113; fig∩chart on 23/25 | **W1-coexist** ✅ |
| Alongside ink empty → wr=0 | Political still `along=12→ink=0→wr=0`, **0 chart hrefs** | **W1-fig-as-chart** (coded, not in this Acc) |
| Residual pages ≠ gold chart pages | 2311 gold Chart Ex pages **4,8**; writes **12,23,25,113** | Gold-page ranking / promote gold figs |
| Specialize density / needles absent | Chart long: **11/14 misses** have `in_doc=False` | Densify specialize + fig promote on gold |
| Extract/format | 2311 p4: gold `"MMMU"` vs pred `["MMMU"]` → Acc 0 (crop had Acc 1) | W4 extract unwrap |

---

## 4. Causal dissection of the Acc rise

Paired vs crop-expand (n=117): **15 improved · 8 worsened · ΔAcc +0.057**

| Attribution bucket | Acc points | Interpretation |
|---|---:|---|
| list_gold | **+0.026** | Mostly extract/list normalize (W4) — **not** Chart W1 |
| unanswerable | **+0.026** | Honesty / refusal calibration — **not** Chart W1 |
| other_answerable | **+0.006** | Tiny; includes Chart ML noise |

**Conclusion:** Headline Acc/F1 gains are real for the product scorecard, but under FP3 they are **not evidence that Chart representation is fixed**. The W1 chart gate is flat.

---

## 5. Chart exclusive (n=7) — exact mechanics

| Doc · page | Crop Acc | Coexist Acc | What happened |
|---|---:|---:|---|
| 2311 · **4** | **1.0** (`MMMU`) | **0.0** (`["MMMU"]`) | **Format regression only** |
| 2311 · **8** | 1.0 | 1.0 | Unchanged |
| political · 5 | 0 | 0 | wr=0 · no chart crop · wrong label pick |
| afe62 · 2 | 0 | 0 | false refusal |
| afe62 · 2–13 | 0 | 0 | false refusal / miss |
| e79 · 12 | 0 | 0 | false refusal (needle *is* on page per fidelity hit elsewhere) |
| germanwings · 19 | 0 | 0 | near-miss time `13:51` vs `14:04` |

Chart exclusive Acc: crop **2/7=0.286** → coexist **1/7=0.143**.  
**Entire Δ is one extract unwrap failure on MMMU.** Coexist did not create a new ChartEx win.

---

## 6. Chart `a_in_e_long` = 0.214 — why it cannot move yet

Gate: long needles (≥3 chars) must appear on **gold evidence-page markdown**.

This run: **3/14 hits · 11/14 misses** — rate **identical** to dense and crop-expand.

Among misses, most have `answer_in_document=False`: the gold string **never enters the indexed corpus at all**, not merely a retrieval miss.

Concrete gaps:

1. **Political / ink-empty class:** residual alongside fires in eligibility, ink kills all writes → coexist has nothing to emit.  
2. **2311 gold pages 4 & 8:** residual write pages were 12/23/25/113. Page 8 has figs but no chart crop under ink-gate (needs fig-as-chart). Page 4 has neither fig nor chart asset.  
3. **Specialize numeric dump:** even when chart crops exist (PIP wr=7, 2311 wr=4), gold Chart needles often still absent from page text → Pass B density / page targeting still insufficient.

Representation miss long **29** vs crop **28** — flat.

---

## 7. What coexist *did* prove (positive, precise)

- When `residual_crops_written > 0`, chart assets and markdown hrefs **survive** alongside figs (verified on 2311).  
- Specialize routing can still mark `Type: Chart` near residual pages.  
- Chart **multi-label** Acc rose (0.273→0.318): some Chart∩other questions benefited indirectly (retrieval/context composition), but this is secondary and **does not** satisfy exclusive or `a_in_e_long` gates.  
- page_hit@5 edged up (+0.027): mild retrieval help when extra chart chunks exist.

So coexist is **necessary plumbing**. Sufficiency for Wave 1 requires surfaces on **gold chart pages** with **dense numeric text**.

---

## 8. Recommendations (ordered, one causal change each)

### Immediate next (already coded) — do this now
1. **Ship Acc #2 with W1-fig-as-chart**  
   - Restart Small with prebuilt binary (`run_chart8_fig_as_chart_acc.sh`).  
   - Expect: political + full-bleed fig pages get `page-*-chart.png` when ink residual empty.  
   - Success metric: Chart `a_in_e_long` moves; ChartEx Acc not solely MMMU-driven.  
   - Also pin W4: unwrap `["MMMU"]`→`MMMU` so extract noise cannot fake ChartEx drops.

### If Chart gate still fails after fig-as-chart
2. **Gold-page residual prioritization**  
   - Prefer residual/promote pages that are caption-chart / quantitative, not random first-12 alongside.  
   - Goal: maximize `gold_evidence_pages ∩ chart_write_pages`.

3. **Specialize density on chart crops (fail-closed, already partially in W1-dense-B)**  
   - Require non-empty `key_values` / `data_table_md` for Chart; retry; keep Pass A dump.  
   - Measure `specialize_numeric_density` and Chart long needle presence.

4. **W1-table densify**  
   - Table long fell 0.412→0.353; tables still dominate answerable mass. Fix table specialize independently (FP3: do not mix with chart Acc run).

### Later
5. **W3 quote-from-context** — only after needles exist on page (attacks wrong-with-hit / false refusal given hit).  
6. **Typed XML grounding tags** — useful for retrieval/Gen after facts exist; **will not** raise `a_in_e_long` alone.

### Do not
- Claim W1 from Acc/F1 alone.  
- Restart Medium vision as the next Chart fix.  
- Ban “Not answerable”.  
- Ship XML/page tags as a substitute for crop+specialize.

---

## 9. Decision tree

```text
Acc↑ F1↑ but Chart a_in_e_long flat?
  ├─ Was Acc attribution Chart exclusive / other_answerable chart mass?
  │    NO → treat Acc as product noise (list/unans). Continue Chart W1.
  └─ Did gold chart pages receive chart crops + numeric specialize?
       NO → fig-as-chart + gold ranking + densify
       YES but a_in_e still fail → specialize/OCR correctness (harder)
```

**This run’s answer:** Acc attribution was mostly list+unans; gold chart pages largely **did not** receive useful chart specialize dumps → continue W1, ship fig-as-chart Acc #2.

---

## 10. Bottom line (plain language)

Coexist fixed a real bug: “we wrote a chart crop, then threw away the retrieval handle.” That was the right first-principles move.

It did **not** put enough **gold chart answers** into page markdown. So Chart fidelity stuck at **0.214**. Acc rose for other reasons (lists, unanswerables, some multi-label Chart questions).

**Next experiment:** Acc with fig-as-chart (already built). That attacks the next proven blocker: `ink=0 ⇒ wr=0 ⇒ coexist never fires` on political / full-bleed figures.
