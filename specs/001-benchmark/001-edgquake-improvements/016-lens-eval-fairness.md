# 016 — Lens: Eval Discipline & Fairness

**Priority:** Continuous — gates every other lens  
**Cross-ref:** [Parent 001 First Principles](../001-first-principles.md) · [003 Protocol](../003-fair-evaluation-protocol.md) · [006 Scorecard](../006-scorecard-schema.md)

---

## 1. Observation

Publishable dual-SUT Acc (`smoke-20260719T124903Z`) required:

- Full corpus (not 100k shell bleed)  
- Forced pins: mistral-small + mistral-embed, chunk 1200, adaptive off, Mix arms LR-like  
- L0 Acc + L2 retrieval + `valid=true`  
- Bootstrap CI on Δ Acc (includes 0 → honest **tie**)

Capped / pin-drifted runs are **invalid as publication**, even if Acc looks decisive.

---

## 2. First-principles diagnosis

Eval axioms that bind improvements:

| ID | Rule |
|----|------|
| P1–P3 | Same corpus, questions, judge |
| P5 | Full `pins.lineage` in scorecard |
| P8 | Fail closed on empty / ingest failure |
| P9 | Non-empty retrieved context exported |
| P11 | Matched retrieve_topk (30) |
| P12 | L2 retrieval required for publish |

**Improvement law:** An engineering change that breaks P11/P12 or hides pin diffs is not a win.

---

## 3. July 2026 practice

- RAG Triad / GraphRAG-Bench L2: faithfulness-adjacent metrics need **context quality**, not Acc alone.
- Offline golden set (smoke n=40) before core ladder; archive every run under `e2e/artifacts/history/`.
- One confound; pre-register success criteria (this pack’s lens tables).
- Separate claims: `P0_mistral_mix_*` vs `P0_paper` (GPT-4o-mini + BGE) vs product profiles.

---

## 4. EQ / harness insertion points

| Area | Location | Action |
|------|----------|--------|
| Publication pins | Makefile `bench001-full`, `tools/bench001` Acc env | Force pins; doctor checks |
| FAKE key scrub | `tools/bench001/bench001/acc_env.py` | Prevent bleed |
| Progress / ETA | `progress.py`, LIVE.md | Observability for long Acc |
| Scorecard | `006-scorecard-schema.md` | Record prune/rerank env flags when added |
| PROGRESS | `e2e/artifacts/PROGRESS.md` | Ladder smoke → core |

---

## 5. Experiments (meta)

| # | Change | Success |
|---|--------|---------|
| V1 | Scorecard fields for `mix_relevancy_prune`, path_prune, rerank, protect | Every ablation run self-describes — **shipped** (`fair_pins.py`) |
| V2 | CI + L2 required in publish mode | `valid=false` if L2 missing |
| V3 | Regression: doctor rejects adaptive=on / char cap / vision drift | Harness tests green |
| V4 | Core ladder only after S1 | **S1 green** `T151125Z` — core still waits on Phase 2 Acc+CI honesty |
| V5 | Phase 2: Acc+CI under S1 package pins (CE+protect) | **Done** — Acc CI includes 0 on `T151125Z` + `T151836Z` → persistent **tie**; L2 not stable ≥0.50 → **no promote** ([020 §2b](./020-roadmap.md)) |
| V6 | Phase 2b: stabilize L2 under S1 pins | ≥2/3 Acc runs ctx_rel ≥0.50 **or** EQ ≥ LR ctx_rel; re-CI; then reconsider promotion / core |

**Phase 2 pin discipline:** S1 package env only ([000](./000-index.md) / [020 §1b](./020-roadmap.md)). Scorecard must show `eq_reranker=cross_encoder`, `rerank_protect_first=12`, `path_prune_fraction=0`. Do not mix cosine prune without a new labeled archive.

---

## 6. Non-goals

- Do not soft-fail empty contexts into scored zeros.
- Do not cherry-pick question subsets post-hoc.
- Do not claim UltraDomain win-rates or MMLongBench scores from SPEC-001.
- Do not amend history archives; append new history folders.
- Do not promote Acc headline CE/protect defaults without a Phase 2 CI archive.
