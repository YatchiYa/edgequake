# 033 — Mix packing under full workspace graph (post-032)

**Status:** Shipping · first-principles token-budget parity  
**Cross-ref:** [032](./032-workspace-graph-identity.md) · [028](./028-first-principles-beat-roadmap.md) · LightRAG `lightrag/constants.py`

---

## 1. Observation

After 032 workspace-scoped AGE identity:

| Run | Acc | recall | ctx | Note |
|-----|----:|-------:|----:|------|
| B2 Acc candidate T071732Z | **0.785** | 0.928 | 0.494 | AGE≪vectors (392/4221) |
| B3b identity T084149Z | 0.734 | **0.960** | 0.394 | age/vec≈1.08 |
| B3b A1+labelFTS T085257Z | 0.749 | 0.941 | 0.519 | Tie vs LR; Acc &lt; B2 |

Identity fixed recall → LR. Acc/ctx still tax. **Not FAQ. Not extract starvation.**

---

## 2. First-principles diagnosis

LightRAG Mix packs context with hard caps (`constants.py`):

| Cap | LightRAG | EQ (legacy) |
|-----|---------:|------------:|
| `MAX_ENTITY_TOKENS` | **6000** | 10000 |
| `MAX_RELATION_TOKENS` | **8000** | 10000 |
| `MAX_TOTAL_TOKENS` | 30000 | 30000 |

When AGE only exposed ~392 WS nodes (B2 collision), the 10k entity budget was **under-filled** → Acc looked fine. After 032 admits ~4k WS entities, the same 10k tax **fills with lower-salience graph text** and crowds chunk remainder → ctx_rel / Complex Acc drop.

Law violated: **token budget identity with the peer SUT** (fair Acc packing), not a soft Mix heuristic.

---

## 3. Fix

- Default `TruncationConfig` → LightRAG 6000 / 8000 / 30000
- Env overrides: `EDGEQUAKE_MAX_ENTITY_TOKENS`, `EDGEQUAKE_MAX_RELATION_TOKENS`
- Acc backend pins the same values in `start_acc_backend.py`
- Intent-aware floors (Factual 2k, Exploratory 4k) still apply via `.min(...)`

Non-goals: FAQ induce, relevancy prune Acc fishing, changing top-k fairness pins.

---

## 4. Success gates

A1 (`rr_cer`, concurrency≤4) on B3b WS `2a7bcb2f-…`:

- ctx≥0.50 ∧ recall≥LR−0.03
- Acc ≥ B2−0.01 (0.775) **or** Beat/Parity CI gates
- Promote warm only if Beat/Parity

---

## 5. Results

| Run | Acc | recall | ctx | Note |
|-----|----:|-------:|----:|------|
| T090743Z A1+6k/8k | **0.7735** | 0.914 | 0.481 | Δ+0.016 tie; L2 miss — no promote |

Warm: B3b `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`. FAQ still closed.  
L2 follow-on: [034](./034-l2-dual-list-under-full-ws-graph.md) Parity `a1lrl2` [`T093152Z`](../e2e/artifacts/history/smoke-20260720T093152Z/).
