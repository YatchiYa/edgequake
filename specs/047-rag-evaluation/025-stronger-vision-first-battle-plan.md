# 025 — Stronger Vision First (Battle Plan)

**Law:** FP1 information only flows forward · FP2 measure bottleneck · FP3 one causal change · FP7 code is law  
**Cross-ref:** [024](./024-first-principles-code-acc-bottleneck.md) · **[026 post-ablation plan](./026-first-principles-score-improvement-brainstorm.md)** · chart-8 runs 2026-07-15  
**Result:** Chart a_in_e **flat 0.40** under `mistral-medium-3-5` — Wave 0 complete; proceed to 026 Wave 1 (density/tables), not another model bump.

---

## 0. Verdict

> **Change Pass A/B vision model first** (query LLM stays Small). Do not rewrite hybrid, harness, or refusal prompts until Chart `answer_in_evidence` moves.

| Evidence (chart-8 `P0_mm_ite`, 2026-07-15) | Value | Causal read |
|--------------------------------------------|------:|-------------|
| Acc / Chart Acc / Table Acc | 0.415 / 0.227 / 0.193 | Gen wrong despite retrieve |
| Chart `a_in_e` (fidelity) | **0.40** | Digits never entered markdown |
| page_hit@5 (answerable) | 0.773 | Retrieval not the Acc gap |
| zero+hit+wrong vs FR\|hit | **35** vs ~0.10 | W1 ≫ W3 |

Locked stack must stay comparable: **Small query + mistral-embed + hybrid + document-scope + `ite`**. Only vision pin changes for this ablation.

---

## 1. Model choice (fact-grounded, July 2026)

| Candidate | Status | Why chosen / rejected |
|-----------|--------|------------------------|
| **`mistral-medium-3-5`** | **Selected** | Official frontier multimodal ID ([docs](https://docs.mistral.ai/models/model-cards/mistral-medium-3-5-26-04)); live API lists it; Pixtral Large retirement alternative; `edgequake-llm` 0.10.1 default chat model; `supports_vision=true` |
| `mistral-medium-latest` | Alias OK | Resolves to Medium 3.5 (`2604`) in edgequake-llm catalog — accept as env alias, prefer explicit `3-5` in scorecard |
| `mistral-large-latest` | Later | Higher cost; not first ablation (FP3) |
| `pixtral-large-latest` | **Forbidden** | Deprecated; replaced by Medium 3.5 ([Mistral notice](https://mistral.ai/news/pixtral-large/)); absent from live `/v1/models` |
| Keep Small for vision | Baseline only | Locked Acc chain — `P0_mm_ite` unchanged |

**SOLID:** Profile owns pins · Workspace pins Pass B VLM · Upload form pins Pass A · Doctor asserts catalog vision capability · Scripts do not clobber a stronger vision override.

---

## 2. DRY / SOLID design

```text
BenchProfile.vision_model  ──►  create_workspace(vision_llm_model)
                           ──►  upload_pdf(vision_model)
                           ──►  doctor + scorecard pins

QUERY_LLM_MODEL = mistral-small-latest     # stable for retrieve/answer
VISION_MODEL_STRONG = mistral-medium-3-5   # W1 Pass A+B only
VISION_MODEL_LOCKED = mistral-small-latest # historical Acc chain
```

| Principle | Application |
|-----------|-------------|
| SRP | Query LLM vs vision model are separate profile fields (already) — do not smuggle Medium into `llm_model` |
| OCP | New profile `P0_mm_ite_vision_medium` — locked `P0_mm_ite` untouched |
| DIP | Run/doctor depend on `BenchProfile`, not hard-coded env strings |
| DRY | Shared constants in `profiles.py`; `ensure_backend_small.sh` respects `EDGEQUAKE_VISION_MODEL` |

---

## 3. Execution sequence

1. **Catalog + pins** — `models.toml` lists `mistral-medium-3-5` with `supports_vision=true`; profile + Makefile target.
2. **Doctor / unit / contract tests** — fail closed if vision pin lacks vision capability; workspace pin resolver prefers `vision_llm_*`.
3. **Chart-8 ablation** — `make bench047-smoke-vision-medium` (same fixture/physics, vision Medium only).
4. **Fidelity gate before Acc story** — Chart `a_in_e` ≥ **0.50** on held-out fidelity sample, then re-compare Acc/Chart Acc vs `P0_mm_ite` baseline.
5. **Only then** — denser Pass A prompts / crop residual (024 §6 tickets 1–2).

---

## 4. Success gates

| Gate | Threshold | Fail action |
|------|-----------|-------------|
| `valid` | true | Fix ops — do not claim Acc |
| ingest_coverage | ≥ 0.9 | Lower PDF_VISION_JOBS / retry |
| Chart `a_in_e` | ≥ **0.50** | Stop Acc storytelling; debug Pass A/B |
| page_hit@5 | ≥ ~0.75 | Representation may still be wrong page-tagged |
| Scorecard pins | `vision_model=mistral-medium-3-5`, `llm_model=mistral-small-latest` | Reject mixed Medium query |

---

## 5. Commands

```bash
# Doctor for stronger-vision profile
python3 -m bench047.cli doctor --profile P0_mm_ite_vision_medium

# Chart-8 Acc physics with Medium vision only
make bench047-smoke-vision-medium

# Or:
export EDGEQUAKE_VISION_MODEL=mistral-medium-3-5
tools/bench047/scripts/run_chart8_vision_medium.sh
```

---

## 6. Non-goals (this wave)

- Do **not** change query `llm_model` to Medium (confounds generation).
- Do **not** silently retarget locked `P0_mm_ite` Acc baseline.
- Do **not** cite TeleMM2.0 Acc as same-protocol win condition.
- Do **not** ship Pixtral Large.
