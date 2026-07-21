# 047 — Type-scoped answer specificity (Complex-only)

**Status:** **REJECT** Acc pin · gate proven · keep B5+`a1fp`  
**Date:** 2026-07-20  
**Archive:** [`T131406Z`](../e2e/artifacts/history/smoke-20260720T131406Z/)  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Prior reject:** [046](./046-answer-specificity-prompt.md) global `a1fpspec` Acc **0.746**  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md)

---

## 1. Assess vs LightRAG (binding)

| Gap | EQ (B5+a1fp) | LR | Law |
|-----|-------------:|---:|-----|
| Overall Acc | 0.801 | 0.782 | Δ+0.019; CI includes 0 → no Beat |
| Complex Acc | 0.813 | 0.863 | **Generation** (ER=1.0 both) |
| Fact ER | 0.85 | 0.90 | smaller |
| Sum ER | 0.863 | 0.983 | SELECT / CE_GAP — Acc fishing STOP |
| recall | 0.926 | 0.966 | Parity blocker (≥LR−0.03) |

**First principles:** LR names Context members; EQ often paraphrases class labels. 046 proved naming helps Complex Δ but **global** Acc-taxed Fact. Intent-scope fails (`query_intent` often `factual` on Complex rows). **Confound:** gate specificity by request `question_type`.

---

## 2. Spec (shipped)

| Item | Value |
|------|--------|
| Env | `EDGEQUAKE_ANSWER_PROMPT=specific` + `EDGEQUAKE_ANSWER_SPECIFIC_TYPES=complex` |
| Semantics | Empty `SPECIFIC_TYPES` → always specific (046). Non-empty → specific iff `question_type` contains a token. Missing type → default. |
| API | Optional `question_type` on `/query` → engine `params["question_type"]` |
| Bench | Pass GraphRAG-Bench `question_type` |
| Ladder | `a1fpscx` |

### Live gate check (post-run)

| `question_type` | Specific prompt? |
|-----------------|------------------|
| Fact Retrieval | **No** (default) |
| Complex Reasoning | **Yes** |

PARP probe (`Medical-54a3a465`): answer includes **olaparib** ✓.

---

## 3. Acc results (`a1fpscx` / T131406Z)

| Gate | Threshold | Result |
|------|-----------|--------|
| Acc | ≥ 0.781 (prefer ≥0.801) | **FAIL 0.764** (Δ vs LR −0.011) |
| Fact ER | ≥ 0.83 | **PASS 0.85** |
| ctx_rel | ≥ 0.50 | **PASS 0.500** |
| Complex Δ vs LR | ≤ 0.03 | **FAIL −0.065** (peer was −0.05; 046 was −0.014) |
| Complex Acc | improve vs peer 0.813 | **FAIL 0.788** |
| Fact Acc | no tax vs peer 0.765 | **FAIL 0.693** (5/10 Fact answers drifted; one “Not answerable”) |

---

## 4. Decision

```text
REJECT a1fpscx as Acc pin
gate wiring OK — type-scope works
specificity Acc family STOP (046 + 047)
keep B5+a1fp peer (0.801)
next ≠ ANSWER_PROMPT Acc fishing
next = Sum ER / recall ceiling (Horizon B / SELECT) without TOPIC_* Acc stack
```

**Insight:** Naming members ≠ Acc F1 win on n=10 Complex; Fact Acc swings hard under same default prompt (judge/LLM noise + refusal). Generation Acc levers exhausted for promote under current pins.