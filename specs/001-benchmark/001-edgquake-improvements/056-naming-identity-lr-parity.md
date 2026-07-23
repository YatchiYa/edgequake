# 056 — Naming identity = LightRAG extract normalize (no Acc fishing)

**Status:** Law✓ code + unit tests · Acc re-ingest **ran** · **REJECT** ([081](./081-beat-parity-first-principles.md) F3 [`T021330Z`](../e2e/artifacts/history/medical-mid-20260723T021330Z/))  
**Date:** 2026-07-21 · updated 2026-07-23  
**Peer keep:** E2-B5 gap-close [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/) · Acc headline P0 mid  
**B10 WS (labeled only):** `54806068-4a82-47b8-a7f9-aeb658f5eddc` · peer `LR_OCC_FACT_L2_B10_v1`  
**Cross-ref:** [081](./081-beat-parity-first-principles.md) · [055](./055-post-acc-ceiling-first-principles.md) · [053](./053-entity-types-lr-parity.md) · LightRAG `normalize_extracted_info`

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
| Acc | `make bench001-b10-reingest` → E2 mid **REJECT** vs E2-B5 (Acc CI LR-ahead; ctx 0.489) |

---

## 3. Gates (B10 mid `T021330Z`)

| Gate | Threshold | Result |
|------|-----------|--------|
| STRUCT / audit | stub zero-rate ≤0.01 | PASS |
| Acc CI vs E2-B5 | not clearly LR-ahead | **FAIL** [−0.087, −0.015] |
| ctx | ≥0.50 or ≥E2+0.02 | **FAIL** 0.489 |
| Fact ER | ≥LR−0.03 or ≥E2+0.02 | PASS ≥LR−0.03 alone |

**REJECT:** keep naming filters (law), keep E2-B5 peer, do not Acc-promote B10.
