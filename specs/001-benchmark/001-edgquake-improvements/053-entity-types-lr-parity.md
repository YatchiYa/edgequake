# 053 — Entity type schema = LightRAG default (no DATE noise)

**Status:** Law shipped · Acc **REJECT** on B8 — keep B5+`a1fp` peer  
**Date:** 2026-07-20  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Archive:** [`T161836Z`](../e2e/artifacts/history/smoke-20260720T161836Z/) Acc **0.748** on B8 `b4f595be-…`  
**Audit:** [`ingest-audit/20260720T161613Z`](../e2e/artifacts/ingest-audit/20260720T161613Z/)  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [029](./029-ingest-parity-audit.md) · LightRAG `prompt.py` `default_entity_types_guidance`

---

## 1. Assess vs LightRAG (no flaky heuristics)

| Gap | EQ (B5) | LR | Notes |
|-----|--------:|---:|-------|
| Acc peer | **0.801** | 0.782 | CI includes 0 |
| eq_coverage_of_lr | 0.724 | — | 897 `only_lr` |
| only_eq sample | timespans (`1_TO_2_WEEKS`, …) | — | **DATE-type induced** |
| Default types | PERSON…**DATE**, PRODUCT, TECHNOLOGY, DOCUMENT | Person…NaturalObject + **Other** (no DATE) | **LAW GAP** |

**Law:** LightRAG classifies with Person, Creature, Organization, Location, Event, Concept, Method, Content, Data, Artifact, NaturalObject; else `Other`. No DATE type.

---

## 2. One confound (shipped, always-on)

| Change | Location |
|--------|----------|
| `default_entity_types()` → LR set + OTHER | `prompts/mod.rs` |
| `PipelineConfig::default().entity_types` sync | `edgequake-core/config.rs` |
| B8 Acc re-ingest | `make bench001-b8-reingest` |

`NATURALOBJECT` = UPPER fold of LR `NaturalObject`. Strict remap of unknowns (incl. legacy `DATE`) → `OTHER`.

---

## 3. Gates — results (B8 + `a1fp`)

| Gate | Threshold | Result |
|------|-----------|--------|
| Acc | ≥ **0.781** (peer ≥ **0.801**) | **0.748** ✗ |
| Fact ER | ≥ **0.83** | **0.85** ✓ |
| ctx_rel | ≥ **0.50** | **0.488** ✗ |
| recall | ≥ LR−0.03 | **0.923** ✗ (LR 0.964) |
| STRUCT coverage | ≥ ~0.70 | **0.735** ✓ (was 0.724; only_lr 862) |

**Verdict:** Law closed (drop DATE/PRODUCT/TECHNOLOGY/DOCUMENT; add LR types). Naming coverage edged up; Acc/ctx tax on fresh extract. Keep code. **Do not** replace B5 Acc peer. Warm pointer restored to `8e990410-…`.

---

## 4. First-principles next

Binding leftover is still **surface-form identity** (e.g. LR `5_FLUOROURACIL` vs EQ `5_FU_FLUOROURACIL`) — not Mix Acc fishing. Candidate: align name normalization with LightRAG `normalize_extracted_info` / title-case consistency **before** `EntityId` fold — separate confound, labeled re-ingest.
