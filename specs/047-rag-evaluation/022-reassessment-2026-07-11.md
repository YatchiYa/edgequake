# 022 — SPEC-047 Re-Assessment (2026-07-11)

**Status:** AUTHORITATIVE snapshot · supersedes stale “latest Acc” claims in older docs when they conflict  
**Peers:** [000](./000-index.md) · [012](./012-acceptance-criteria-and-scorecard.md) · [013](./013-first-principles-improvement-roadmap.md) · [015](./015-modality-aware-vision-improvement-plan.md) · [019](./019-query-first-principles-improvement-plan.md)–[021](./021-lineage-first-principles-query.md)  
**Canvas:** [spec047-reassessment-20260711](/Users/raphaelmansuy/.cursor/projects/Users-raphaelmansuy-Github-03-working-edgequake/canvases/spec047-reassessment-20260711.canvas.tsx)  
**Law:** Code + locked smoke artifacts are truth. LVLM GPT-4o F1≈44.9% is **difficulty only**, not a same-task rank.

---

## 0. One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  SPEC-047 harness is LIVE and useful. Query-lane work (Q1→A3→lineage) moved │
│  chart-fixture Acc: 0.384 → 0.436 → 0.393 → 0.429 → 0.427 (±noise).         │
│  MV-18/19 re-ingest: Acc **0.423** · Chart Acc **0.182** · Chart a_in_e     │
│  **0.409** (was ~0.32). Rep moved; Acc flat until G-A (≥0.50).              │
│                                                                              │
│  Next lawful queue:  015 Chart denser extract (clear G-A) ‖ B3 Mix ‖ L-B2   │
│  Do not: ban “Not answerable” · treat LVLM as peer task                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

| Axis | Verdict |
|------|---------|
| **Harness** | Done — `bench047` smoke/core/full, scorecard, diagnostics, document-scope |
| **Ingest ops** | Strong — soft-resume, unique-before-embed, P7a–f merge gates, battle-plan |
| **Query honesty** | Strong — grounding, arm gates, empty-arm prune, lineage scope L-A1–A4 |
| **Representation** | **Moved** — Chart a_in_e 0.32→**0.41**; Chart Acc 0.14→**0.18**; G-A/G-B open |
| **Eval maturity** | Smoke locked on **8-doc chart fixture** + dscope; core/full not yet scored |

---

## 1. Locked evidence chain (same physics)

**Pin (query-lane chain):** workspace `ee47b44c-…` · **Pin (MV-18 HEAD):** `be4c40a9-252d-48a9-a57b-d42ea9f4ef30` · profile `P0_mm_ite` · mode `hybrid` · `--document-scope` · fixture `smoke_chart_doc_ids_v1.txt` · n=117.

| Milestone | Artifact | Acc | F1 | Unans | Pure-text | Chart | page_hit@5 | What landed |
|-----------|----------|-----|-----|-------|-----------|-------|------------|-------------|
| Pre-Q1 | `smoke-pre-q1-grounding` | 0.384 | 0.224 | 0.691 | 0.269 | 0.136 | 0.760 | Soft-resume 8-doc baseline |
| Post-Q1 | `smoke-post-q1-grounding` | **0.436** | 0.255 | **0.810** | 0.192 | 0.182 | 0.733 | Entailment-first grounding |
| Post-B2 | `smoke-post-b2-arm-gate` | 0.393 | 0.175 | 0.810 | 0.192 | 0.182 | 0.747 | Hybrid arm honesty (Acc tax) |
| Post-A3 | `smoke-post-a3-acc-recovery` | **0.429** | 0.238 | 0.810 | **0.255** | 0.136 | 0.747 | Empty-arm prune + factual tax |
| Post-lineage | `smoke-post-lineage-la2` | **0.427** | 0.225 | **0.833** | 0.192 | 0.136 | **0.760** | L-A2/A3 fail-closed scope (no re-ingest) |
| Post-MV18 | `smoke-post-mv18-full-chart` | **0.423** | 0.232 | 0.786 | **0.216** | **0.182** | 0.720 | Pass A + specialize re-ingest (Chart a_in_e **0.409**) |

**Canonical “latest” for comparison:** treat **`smoke-post-a3-acc-recovery`** as Acc peak after query calibration, **`smoke-post-lineage-la2`** as post-scope hygiene, and **`smoke-post-mv18-full-chart`** as current HEAD after Chart Rep prompts. `artifacts/smoke/` currently mirrors MV-18 full.

**Ignore for scoring:** `smoke-invalid-api-down-*`, 1-doc MV-32 toys with n≪117, any run with `valid=false` / `context_empty_rate≈1`.

### Reading the deltas

| Δ | Causal class | Lawful response |
|---|--------------|-----------------|
| Acc +0.05 (pre→post Q1) | Gen / grounding | Keep; calibrate (done A1) |
| Acc −0.04 (Q1→B2) | R-gate honesty tax | Expected; recovered by A3 |
| Acc flat (A3→lineage) | Scope hygiene | Pass — no Acc regression |
| Unans 0.69→0.83 | Selective refusal skill | Keep; never ban NA |
| Pure-text 0.27→0.19→0.26→0.19 | G-cal + lineage drop tax | Watch; Q1.5 + optional L-B2 rates |
| Chart ~0.14–0.18 | **Rep** | **015** — MV-18/19 lifted Chart Acc 0.14→0.18; keep pushing a_in_e |
| page_hit@5 ~0.73–0.76 | R-OK | Fusion not the bottleneck |

---

## 2. Ticket board (code is law)

### Done (do not reopen without new evidence)

| Track | Tickets | Proof |
|-------|---------|-------|
| Harness | EQ-047 smoke path, scorecard, W0 page_hit, false_refusal, arm gates | `tools/bench047` |
| Ingest reliability | 016 P6–P7f, soft-resume, unique embed, KEEP/FIFO | contracts + live soft-resume |
| Query Q1 | Grounding headers, chunk budget floor | `grounding.rs`, `e2e_spec047_query_grounding` |
| Post-Q1 | A1–A3, B1–B2 | 020 table + smokes above |
| Lineage | L-A1–L-A4 | `lineage_scope`, `merger/lineage`, diverse KEEP e2e |

### Open — ranked by leverage × lawfulness

| # | Ticket | Why now | Gate | Effort |
|---|--------|---------|------|--------|
| **1** | **015 Chart denser extract** | Chart a_in_e **0.41** < G-A 0.50; Acc flat | Chart a_in_e ≥0.50 (G-A) then ≥0.60 (G-B) | M |
| **2** | **020 B3 Mix ablation** | Acc recovered ≈ post-Q1; fusion tax isolable | Acc/F1 vs A3; page_hit held | S |
| **3** | **021 L-B2 lineage telemetry** | Pure-text dip after fail-closed needs rates | SUMMARY: drop / multi-doc / in-scope rates | S |
| **4** | **021 L-B1 cite chain** | UX/audit; not Acc primary | Entity→chunk→page in sources API | M |
| **5** | **Optional re-ingest** | Stamp `source_document_ids[]` on AGE nodes | Contract already green; Acc optional | L |
| **6** | Core stage (~40 docs) | Smoke physics locked; expand signal | valid core scorecard | L |

### Explicitly rejected (still)

- Ban / soft-ban “Not answerable”
- Gold `evidence_pages` in retrieve (leak)
- Mid-run provider swaps
- Acc patches that ignore Chart fidelity
- Treating TeleMM / GPT-4o LVLM Acc as EdgeQuake peer ranking

---

## 3. Dual-lane physics (still true)

```text
          ┌─────────────────────┐
 PDF ───▶ │  Representation     │──▶ markdown / chunks / modalities
          │  (015 · vision)     │         │
          └─────────────────────┘         │ missing numbers = hard floor
                                          ▼
          ┌─────────────────────┐    retrieve + scope + Gen
 Q ──────▶│  Query lane         │──▶ Acc / Unans / Pure-text
          │  (019–021 · done‡)  │
          └─────────────────────┘
 ‡ query lane “done” for current Acc band — Mix ablation + telemetry remain
```

**Corollary:** Query Acc 0.38→0.43 was real. Further Acc >~0.45 on this fixture **without** Chart representation work is unlikely and would be suspicious.

---

## 4. Spec tree health (staleness)

| Doc | Status after this re-assess |
|-----|-----------------------------|
| 000 | **Updated** — status + reading order + Acc band |
| 001–012 | Still foundational; Acc numbers in narrative may be early-smoke — prefer 022 table |
| 013 | Strategy still valid; baseline Acc 0.45 is **old 10-doc** — use 022 chain |
| 014–016 | Historical physics + battle plan — keep |
| 017–018 | EQ↔LightRAG — still guide Mix/015 |
| 019–020 | Query/post-Q1 — **A3 landed**; B3 next |
| 021 | L-A1–A4 **done**; L-B* open |
| **022** | **This doc** — current program state |
| e2e/README | **Updated** — latest Acc + artifact map |

---

## 5. Definition of “next done”

A workstream is done only when:

1. Code + e2e/contract tests land  
2. Scorecard artifact archived under `e2e/artifacts/<name>/`  
3. Acc / Unans / Chart / page_hit@5 reported vs **A3 (0.429)** and **lineage (0.427)**  
4. Class of failure named (Rep / R-fusion / G-cal / Scope)

**Program-level DoD (unchanged spirit of 012):**

- Smoke valid on locked profile  
- Unanswerable Acc ≥ 0.70 (held: 0.83)  
- Chart Acc improvement owned by 015, not query prompts  
- No leaderboard cosplay  

---

## 6. Recommended next 48h

1. **015 denser Chart extract** — clear G-A (Chart a_in_e ≥0.50); MV-18/19 proved direction.  
2. **B3 Mix** query-only ablation vs A3 — isolates fusion tax (orthogonal).  
3. Ship **L-B2** telemetry (cheap).  
4. Do **not** expect more Acc from query-only without further Rep.

---

## Citation

Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.  
Upstream: https://github.com/mayubo2333/MMLongBench-Doc · Leaderboard F1≈44.9% (GPT-4o) is difficulty reference only.
