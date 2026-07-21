# 024 — Acc Parity / Beat LightRAG

**Status:** Ladder complete — **no Acc headline promote** (Q4 recall miss); best peer = P2b  
**Date:** 2026-07-20  
**Parent:** [022](./022-deep-top-performance-plan.md) · [023](./023-p4-acc-ci-decision-gate.md) · [017](./017-beat-lightrag.md)  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`

---

## 1. Outcome gates

| Outcome | Gate |
|---------|------|
| **Beat** | Δ Acc 95% CI excludes 0 (EQ) **and** ctx_rel ≥ 0.50 **and** recall ≥ LR − 0.03 |
| **Parity** | CI includes 0 **and** ctx_rel ≥ 0.50 on ≥2/3 runs **and** recall ≥ LR − 0.03 |
| **No promote** | Keep Acc headline P0 BM25 / `PATH_PRUNE=0` / `PROTECT=0` |

Claim language: “beats LightRAG” **only** on Beat. Parity → “peer / statistical tie.”

---

## 2. Baseline (do not re-learn)

| Fact | Archive |
|------|---------|
| Best peer pack P2b | `T014814Z` Acc 0.752 · ctx_rel 0.50 · Complex Δ −0.023 |
| P4 stack toxic | `T015647Z` Acc 0.677 — never restack gw+lexical on S1 |
| Fact = noise/order | Fact recall EQ ≥ LR; Fact ctx_rel lag |
| Exhausted | related_chunk↑, naive×2, query_score alone, BM25+path, RR Acc default |

---

## 3. Ladder (Q0–Q4)

```bash
make bench001-q0   # P2b stability (×3)
make bench001-q1   # occurrence-sort on P0 BM25
make bench001-q2   # VECTOR LR budget on P0 BM25
make bench001-q3   # single Fact winner on P2b
make bench001-q4   # final CI decision / promote gate
# script: tools/bench001/scripts/run_p_ladder_acc.sh q0|q1|q2|q3|q4
```

| Step | Package | Success |
|------|---------|---------|
| **Q0** | Exact P2b pins ×3 | ≥2/3 L2 parity; Acc ≥ P0−0.02 |
| **Q1** | `EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=1` on P0 | Fact Acc +≥0.03 **or** Fact ctx_rel +≥0.05 |
| **Q2** | `EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=1` on P0 | same (do not stack with Q1) |
| **Q3** | Better of Q1/Q2 **only** on P2b | Complex Δ ≤0.05; ctx_rel ≥0.48 |
| **Q4** | Winning package | Beat or Parity → promote; else peer-only |

---

## 4. Code hooks

| Env | Behavior | Files |
|-----|----------|-------|
| `EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=1` | Sort entity `source_chunk_ids` by citation freq before `related_chunk` take | `kg_chunk_pick.rs` |
| `EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET=1` | Uncapped VECTOR pool; take `related×n_entities/2` | `kg_chunk_pick.rs`, `chunk_retrieval.rs` |

---

## 5. Promote targets (only after Q4 green)

Flip Acc headline in `acc_env.py` `PUBLICATION_ENV` + `start_acc_backend.py` `ACC_EXPORTS` to P2b (+Fact winner if any). Require `DASHSCOPE_API_KEY`.

---

## 6. Ledger

| Step | Archive | EQ Acc | LR Acc | Notes |
|------|---------|--------|--------|-------|
| **Q0×3** | `T022256Z` / `T022614Z` / `T022950Z` | 0.732–0.737 | 0.770–0.785 | L2 0/3 (ctx flaky; recall miss) |
| **Q1** | `T023313Z` | 0.736 | — | Occurrence on P0: Fact Acc −0.069 **miss** |
| **Q2** | `T023625Z` | 0.701 | — | LR budget: Fact ctx +0.05; Fact Acc −0.028 |
| **Q3** | `T023939Z` | 0.755 | 0.800 | P2b+lr_budget **reject** (Complex Δ −0.094) |
| **Q4** | `T024233Z` | **0.756** | 0.780 | ctx 0.506 ✅; recall miss; CI tie → **no promote** |

**Decision:** Acc headline unchanged (P0 BM25 / path off). Labeled peer = P2b. Fact VECTOR knobs stay off by default.

---

## 7. Non-goals

- Restack P4 (gw_compress + lexical on S1)  
- BM25 + soft path  
- Soft Mix Acc-win fishing  
- Latency ≤1.5× (deferred)  
- SOTA / HippoRAG2 claims without gates  
