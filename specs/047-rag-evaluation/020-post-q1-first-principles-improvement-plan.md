# 020 — Post-Q1 Improvement Plan (First Principles)

**Status:** A1–A2 + B1–B2 + **A3 Acc recovery** landed (2026-07-11) · next: **B3 Mix ablation** · 015 Chart  
**Re-assessment:** [022](./022-reassessment-2026-07-11.md) — Acc peak A3 **0.429**; lineage **0.427**; Chart still Rep  
**Scope:** Dual-lane — **query calibration** (false refusal) + **representation** (charts) + **retrieval honesty** (Mix / arm gate)  
**Evidence (locked smoke):** query-only · `P0_mm_ite` · document-scope · same workspace  
**Peers:** [019](./019-query-first-principles-improvement-plan.md) · [015](./015-modality-aware-vision-improvement-plan.md) · [017](./017-lightrag-vs-edgequake-query-pipeline-assessment.md) · [018](./018-quality-speed-improvement-plan.md) · [021](./021-lineage-first-principles-query.md)  
**Canvas:** [spec047-post-q1-first-principles](/Users/raphaelmansuy/.cursor/projects/Users-raphaelmansuy-Github-03-working-edgequake/canvases/spec047-post-q1-first-principles.canvas.tsx)  
**Artifacts:** [`smoke-pre-q1`](./e2e/artifacts/smoke-pre-q1-grounding/SUMMARY.md) → [`smoke-post-q1`](./e2e/artifacts/smoke-post-q1-grounding/SUMMARY.md) → [`smoke-post-b2`](./e2e/artifacts/smoke-post-b2-arm-gate/SUMMARY.md) → [`smoke-post-a3`](./e2e/artifacts/smoke-post-a3-acc-recovery/SUMMARY.md) → [`smoke-post-lineage`](./e2e/artifacts/smoke-post-lineage-la2/SUMMARY.md)

### Implementation status (2026-07-11)

| Ticket | Status | Evidence |
|--------|--------|----------|
| A1 / Q1.5 calibrated grounding | ✅ | `grounding.rs`; e2e `e2e_a1_*` |
| A2 false-refusal metric | ✅ | SUMMARY Refusal diagnostics |
| B1 arm-gate rates | ✅ | SUMMARY Arm-gate (+ `planned_*`) |
| B2 hybrid arm honesty | ✅ | `intent_arm_mask_hybrid`; `planned_naive_only`→0 |
| **A3 + empty-arm prune** | ✅ smoke | Acc **0.393→0.429**; Pure-text **0.192→0.255**; mean `n_sources` 108→35 |
| B3 Mix ablation | ⏳ | Acc ≈ post-Q1 — **now lawful** |
| C* Chart / 015 | ⏳ | Chart Acc still ~0.14–0.18 (Rep) |

---

## 0. What changed (physics, not vibes)

| Metric | Pre-Q1 | Post-Q1 | Δ | Class signal |
|--------|--------|---------|---|--------------|
| Acc | 0.384 | **0.436** | **+0.051** | G helped |
| F1 | 0.224 | **0.255** | +0.032 | — |
| Unanswerable Acc | 0.691 | **0.810** | **+0.119** | Refusal skill ↑ |
| Pure-text Acc | 0.269 | **0.192** | **−0.077** | **Over-refusal** |
| Chart Acc | 0.136 | 0.182 | +0.046 | Still **Rep** floor |
| Table Acc | 0.167 | 0.167 | 0 | Rep / Gen |
| `page_hit@5` | 0.76 | 0.73 | −0.03 | Still **R-OK** |
| False refusal (answerable→NA) | — | **~0.33** | — | Calibrate, don’t ban |
| Cite `[N]` rate | — | ~0.62 | — | Grounding live |

**One-screen law (updated):**

```text
  page_hit@5 ≈ 0.73  +  Acc ≈ 0.44
       │
       ├─ Unanswerable Acc ↑↑  →  refusal detection improved (keep Q8)
       ├─ Pure-text Acc ↓      →  selective refusal mis-calibrated (Q1.5)
       ├─ Chart Acc still ~0.18 →  gold often not in markdown (015 / Rep)
       └─ arm local/global ↓   →  possible Factual→naive-only gate (Q2.4)
```

**Master axiom (unchanged):** *You cannot prompt your way out of a missing page, and you cannot fuse your way out of invisible evidence.*

**New corollary (post-Q1):** *You also cannot refuse your way into Acc — selective refusal must be calibrated, not maximized.*

Research anchors (2025–2026): selective refusal is a **separable skill** from answer accuracy ([RefusalBench](https://aclanthology.org/2026.eacl-long.321)); production systems train/calibrate abstention rates rather than banning “I don’t know” ([FinRAG-12B](https://arxiv.org/html/2605.05482v1)). EdgeQuake stays prompt/harness-level — no fine-tune required for Phase A.

---

## 1. First principles (post-Q1 additions)

Carry forward 019 **Q1–Q10**. Add:

| ID | Principle | Operational meaning |
|----|-----------|---------------------|
| **Q11** | Selective refusal ≠ maximal refusal | Optimize **joint** (answerable Acc, unanswerable Acc); track false-refusal rate |
| **Q12** | Answer when a supporting chunk exists | If `page=` / modality chunk entails the fact, prefer a cited short answer over “Not answerable” |
| **Q13** | Dual-lane parallelism is lawful | Query calibration (Q1.5) and ingest (015) are **orthogonal**; run in parallel |
| **Q14** | Arm gates are retrieval law | If `arms_gated` collapses hybrid→naive, Acc deltas are not “fusion science” |
| **Q15** | Measure the tax of each prompt clause | Every grounding sentence must earn its keep on a labeled ablation |

### Five WHYs (post-Q1)

| Why | Because | Therefore |
|-----|---------|-----------|
| Why Acc↑ after Q1? | More honest refusals + some better cites | Keep grounding headers; **calibrate** refusal text |
| Why Pure-text↓? | Prompt says refuse when unsure; text answers look “incomplete” | **Q1.5** entailment-friendly refusal rule |
| Why Chart still ~0.18? | Numbers often absent from evidence markdown | **015** / fidelity — not more Mix arms |
| Why local/global arm counts collapsed? | Intent gate or arm mask → naive-heavy | **Q2.4** audit before trusting Mix ablation |
| Why not ban “Not answerable”? | Unanswerable Acc 0.81 is a product feature (Q8) | Calibrate threshold; never delete refusal |

---

## 2. Failure taxonomy (re-triage)

Assign **primary** class per miss (same R/G/Gen/Rep/Scope as 019):

| Class | Post-Q1 dominant signal | Next lawful move |
|-------|-------------------------|------------------|
| **G-cal** (new subclass) | `page_hit@5` true **and** pred=`Not answerable` on answerable gold | Q1.5 calibrated refusal |
| **Rep** | Chart/Table; fidelity / answer_in_evidence low | 015 Phase C/D; no query Acc hacks |
| **R-gate** | `arms_run=naive` + `arms_gated=true` on hybrid request | Q2.4 intent mask audit |
| **R-fusion** | page_hit flat after gate fixed | Q2.1 Mix RRF ablation |
| **Gen** | Evidence quoted; extractor short-answer wrong | Extractor last; rare |

**Reject starting with:** PPR, more graph hops, BM25-off, “never say Not answerable.”

---

## 3. Decision tree (what to build next)

```text
IF false_refusal_rate on (answerable ∧ page_hit@5) > 0.20
  → Q1.5 calibrated grounding (query lane)     # THIS WEEK
PARALLEL IF Chart Acc < 0.25 AND answer_in_evidence(Chart) low
  → 015 / fidelity (ingest lane)               # THIS WEEK
ELSE IF hybrid requests show arms_gated→naive majority
  → Q2.4 intent / arm-mask fix                 # BEFORE Mix claims
ELSE IF gate honest
  → Q2.1 Mix RRF vs hybrid ablation            # NEXT
ELSE IF page_hit@5 < 0.70 after gate fix
  → Q3 rerank depth / modality filter
ELSE IF cross-page Acc flat after G-cal + Mix
  → Q6 optional graph science
```

---

## 4. Phased plan

### Phase A — Calibrated refusal (query-only, highest ROI Acc recovery)

*Why:* Post-Q1 Acc lift came partly from over-refusal; Pure-text paid the tax (Q11–Q12).

| # | Ticket | Symbol / surface | Change | Gate | Effort |
|---|--------|------------------|--------|------|--------|
| **A1 / Q1.5** | Calibrated grounding copy | `grounding.rs` | Explicit: if a Document Chunk **supports** the asked fact, answer + cite `[N]`; refuse **only** when no chunk/KG fact supports it. Prefer partial grounded answers over NA. | False-refusal ↓ ≥25% rel.; Unanswerable Acc ≥ 0.70; Pure-text Acc ≥ pre-Q1 (0.27) | S |
| **A2** | False-refusal metric in harness | `diagnostics.py` / SUMMARY | `false_refusal = answerable ∧ pred≈NA`; slice by page_hit@5 | Printed in SUMMARY every smoke | S |
| **A3 / Q1.4** | Intent-aware graph tax + empty-arm prune | `truncation_config_for_intent` + `prune_empty_arm_graph` | Factual: entity/rel ≤2k tok, chunk floor ≥0.55; empty local/global drop orphan entities | Acc ≥ post-Q1; Pure-text ↑; mean `n_sources` ↓ | M |
| **A4** | Prompt clause ablation | bench notes | A1-only vs A1+strict cite vs baseline | One change per run (Q9) | S |

**Reject:** Removing “Not answerable” from prompts or scoring.

### Phase B — Retrieval honesty (before claiming Mix wins)

*Why:* Post-Q1 smoke showed `mean_arm_local/global` near-zero vs pre-Q1 ~3–4 — fusion experiments are invalid until arms actually run (Q14).

| # | Ticket | Symbol | Gate | Effort |
|---|--------|--------|------|--------|
| **B1 / Q2.4** | Publish arm-mask + `arms_gated` rates in SUMMARY | `mix_weights.rs`, telemetry | Hybrid smoke: local+global share > 0 on ≥50% of Qs **or** documented intentional naive-only | S |
| **B2** | Fix false Factual→naive-only if misroute proven | `intent_arm_mask_hybrid` | Hybrid: Factual→local+naive (Mix keeps naive-only). Smoke: `planned_naive_only` 0.85→**0.00** | M |
| **B3 / Q2.1** | Ablation `P1_mix_rrf` vs `P0_mm_ite` hybrid | `profiles.py` (ready) | Acc / page_hit / Chart table; same workspace query-only | S |
| **B4 / Q2.2** | If Mix wins → lock bench default | `000-index`, Makefile | Acc↑ held on re-run | S |
| **B5 / Q2.3** | Optional hybrid fusion=RRF for factual | `hybrid_merge.rs` | Only if B3 null and page_hit@1 lag | S |

### Phase C — Representation (ingest lane, parallel)

*Why:* Chart Acc 0.18 with page_hit still high ⇒ gold often invisible (019 Q1 / 015 FP1).

| # | Ticket | Hand-off | Gate |
|---|--------|----------|------|
| **C1** | Chart fidelity audit on smoke chart docs | `bench047 fidelity` | `answer_in_evidence(Chart)` baseline published |
| **C2** | 015 typed chart specialize / numeric extract | [015](./015-modality-aware-vision-improvement-plan.md) | Chart fidelity ↑ → Chart Acc ≥ 0.25 after re-query |
| **C3** | Soft-resume reprocess chart PDFs only | ingest fingerprint | No full wipe; query-only rescore |

Do **not** implement C in `edgequake-query`.

### Phase D — Precision (only if A+B stall on page_hit)

| # | Ticket | Gate |
|---|--------|------|
| **D1 / Q3.1–Q3.2** | Bind cross-encoder; retrieve deep → rerank 20 | page_hit@5 ≥ 0.80 or Acc↑ |
| **D2** | Fail-open rerank (keep) | no INVALID from rerank alone |

### Phase E — Optional science (last)

Cross-page Acc still flat after A+B+C → PPR / communities (019 Q6). Not this week.

---

## 5. Experiment protocol

1. **One causal change** per smoke (or labeled pair: hybrid vs mix).  
2. Always report: Acc, F1, Unanswerable, Pure-text, Chart, `page_hit@5`, **false_refusal**, `arms_gated` rate.  
3. Prefer **query-only + document-scope + soft-resume** on workspace `ee47b44c…` for query tickets.  
4. For ingest tickets: fidelity first, then re-query.  
5. Fail closed on empty answers.  
6. No gold `evidence_pages` in retrieve.

```text
Order this week:
  A2 metric (cheap) → A1 calibrated refusal → smoke
  ∥ C1 fidelity → C2/C3 015 if Chart evidence missing
Then:
  B1 arm telemetry → B2 if broken → B3 Mix ablation
```

---

## 6. Scoreboard targets (post-Q1)

| Gate | Metric | Target | Floor |
|------|--------|--------|-------|
| GQ-A | Acc (smoke, scoped) | ≥ **0.48** after A1 (+B3 if Mix wins) | valid=true |
| GQ-B | Pure-text Acc | ≥ **0.27** (recover pre-Q1) | — |
| GQ-C | False refusal (ans ∧ page_hit@5) | ≤ **0.20** | Unanswerable Acc ≥ 0.70 |
| GQ-D | Chart Acc | ≥ **0.25** (needs C if fidelity low) | — |
| GQ-E | `page_hit@5` | ≥ 0.75 (hold) | context_empty ≤ 0.05 |
| GQ-F | Hybrid arm honesty | local∪global present when mode=hybrid | documented gate |

---

## 7. Anti-patterns

| Anti-pattern | Violates |
|--------------|----------|
| Ban “Not answerable” to pump Acc | Q8, Q11 |
| Mix ablation while arms_gated→naive | Q14, Q9 |
| More fusion to fix Chart Acc | Q1, Rep hand-off |
| Prompt-only Acc patches when fidelity low | Q1 |
| Re-ingest whole corpus to test A1 | Waste; use query-only |
| Mid-run provider/model swap | Q9 |

---

## 8. Definition of done

- [x] A2 false-refusal in SUMMARY  
- [x] A1 landed; Pure-text recovering (0.19→**0.255** post-A3)  
- [x] B1 arm-gate rates published; B2 hybrid mask landed  
- [x] B2 smoke: `planned_naive_only_rate` ≈ 0  
- [x] Acc recovery after B2 tax: **0.393→0.429** (≈ post-Q1 0.436)  
- [ ] B3 Mix vs hybrid Acc table published  
- [ ] C1 Chart fidelity number; 015 triggered if low  
- [x] Unanswerable Acc never sacrificed below 0.70 (held **0.81**)  
- [x] false_refusal ↓ vs post-Q1 (0.333→~0.29)

---

## 9. Locked smoke deltas

### Post-B2 (gate honesty)

| Metric | Post-Q1 | Post-B2 | Signal |
|--------|---------|---------|--------|
| Acc | 0.436 | **0.393** | ↓ context pollution |
| `planned_naive_only` | 0.855 | **0.000** | **B2 gate fixed** |
| mean `n_sources` | ~29 | **~109** | orphan KG flood |

### Post-A3 (Acc recovery — first principles)

| Metric | Post-B2 | Post-A3 | Signal |
|--------|---------|---------|--------|
| Acc | 0.393 | **0.429** | recovered ≈ post-Q1 |
| Pure-text Acc | 0.192 | **0.255** | toward pre-Q1 0.27 |
| Unanswerable Acc | 0.810 | 0.810 | hold |
| mean `n_sources` | ~109 | **~35** | pollution cut |
| `planned_naive_only` | 0.000 | 0.000 | honesty held |
| Chart Acc | 0.182 | 0.136 | still **Rep** → 015 |

**Law after A3:** Empty graph arms must not tax Gen; Factual queries protect chunk budget. B3 Mix ablation is now **lawful**. Chart remains representation work (015), not more fusion.

**Relationship to 019:** 019 remains the query decision system and Q1 grounding SSOT. **020 is the post-Q1 execution plan.**

---

## 9. Immediate next actions (this week)

1. ~~**A2** — add `false_refusal` (+ optional `false_refusal|page_hit@5`) to bench SUMMARY.~~ ✅  
2. ~~**A1** — rewrite `grounding_instructions()` for entailment-first selective refusal (keep NA).~~ ✅  
3. **Query-only smoke** with rebuilt binary; compare to `smoke-post-q1-grounding`.  
4. **B2** if B1 shows naive-only collapse; then **B3** `P1_mix_rrf` ablation.  
5. **∥ C1** — Chart fidelity; if low → 015, not Mix.
