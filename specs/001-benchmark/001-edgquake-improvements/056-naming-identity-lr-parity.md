# 056 — Naming identity = LightRAG extract normalize (no Acc fishing)

**Status:** Law✓ code + unit tests · Acc re-ingest **deferred** (B10)  
**Date:** 2026-07-21  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Cross-ref:** [055](./055-post-acc-ceiling-first-principles.md) · [053](./053-entity-types-lr-parity.md) · LightRAG `lightrag/utils.py::normalize_extracted_info`

---

## 1. Assess vs LightRAG (First Principles)

| Gap | EQ | LR | Binding? |
|-----|----|----|----------|
| Storage id fold | `EntityId` → `UPPER_SNAKE` | Title-case display names; case-sensitive graph | Different conventions — audit compares after normalize |
| Short pure numeric | Kept (`2022`, `42`) | **Drop** if `len < 3` and digits-only | **LAW** |
| Short dotted numeric | Kept | **Drop** if `len < 6` and only digits+dots | **LAW** |
| Surface synonyms | `5_FU_FLUOROURACIL` vs LR `5_FLUOROURACIL` | Exact-name merge only | **Not** soft-match Acc fishing |

**One confound now:** port LR empty-after-normalize filters for short numeric / dotted-numeric names.  
**Not now:** LLM synonym merge, embedding alias merge, Acc soft-overlap fishing.

---

## 2. Implementation

| Change | Location |
|--------|----------|
| Reject empty after LR-style numeric filters | `edgequake-storage` `normalize_entity_name` or extract pre-filter |
| Unit tests | digits `<3`, dotted `<6` → empty; real names unchanged |
| Acc | **Deferred** `make bench001-b10-reingest` — promote only if Acc gates clear |

---

## 3. Gates (when B10 runs)

| Gate | Threshold |
|------|-----------|
| STRUCT | fewer pure-year/digit-only `only_eq` samples; coverage not crash |
| Acc | ≥ 0.781 (prefer ≥ 0.801) · Fact ER ≥ 0.83 · ctx ≥ 0.50 |

On REJECT: keep code (law), keep B5 peer.
