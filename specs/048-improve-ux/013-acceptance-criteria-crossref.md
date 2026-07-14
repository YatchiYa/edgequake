# 013 — Acceptance Criteria & Cross-Ref Matrix

---

## 1. Acceptance gates

| ID | Gate | How to verify | Lenses |
|----|------|---------------|--------|
| AC-01 | Busy pill true ⇒ `PipelineActivity.busy` true and ≥1 working doc or task | Contract + UI unit | PO BE FS |
| AC-02 | Banner `stage` == row `stage` for active document | Playwright Documents | UX UI FE |
| AC-03 | After reprocess, stage fields reset within 2s | API assert + UI | PO BE |
| AC-04 | No 404 on `/ingestion/*/progress` in happy path | Network log e2e | FE BE |
| AC-05 | Chunk progress ticks appear on WS during extract | WS capture e2e | BE FS |
| AC-06 | Upload stepper morphs away after `track_id` | Playwright | UX UI FE |
| AC-07 | `mode=merge` visible on soft-reprocess | UI + DTO | PO FE BE |
| AC-08 | Tab title distinguishes Working vs Queued | Unit `useDocumentTitle` | UX FE |
| AC-09 | Filter counts use display status SSOT | Unit filters | FE |
| AC-10 | i18n keys for all UnifiedStage labels present in `en.json` | i18n check | UI FE |
| AC-11 | Heartbeat: `updated_at` advances ≤15s while Working | E2E long ingest | UX FS |
| AC-12 | False Busy with empty working+tasks = 0 in soak | Soak / contract | PO FS |

---

## 2. Defect → AC map

| DEF | Title | AC |
|-----|-------|----|
| DEF-01 | Missing progress route | AC-04 |
| DEF-02 | WS chunk/graph gaps | AC-05 |
| DEF-03 | Reprocess stage stale | AC-03 |
| DEF-04 | Busy semantics | AC-01 AC-12 |
| DEF-05 | Banner gating | AC-02 |
| DEF-06 | Tab title | AC-08 |
| DEF-07 | Filter counts | AC-09 |
| DEF-08 | Duplicate status lines | AC-02 |
| DEF-09 | i18n gaps | AC-10 |
| DEF-10 | Upload FSM ≠ server | AC-06 |

---

## 3. FP → AC map

| FP | Principle | AC |
|----|-----------|-----|
| FP-01 | One vocabulary | AC-02 AC-10 |
| FP-02 | One busy rule | AC-01 AC-12 |
| FP-03 | Live heartbeat | AC-05 AC-11 |
| FP-04 | Countable when known | AC-05 |
| FP-05 | Terminal honesty | AC-03 |
| FP-06 | No false Busy | AC-01 AC-12 |
| FP-07 | Transport completeness | AC-04 AC-05 |
| FP-08 | Auto-update | AC-11 |
| FP-09 | Mode transparency | AC-07 |
| FP-10 | Code is law | all (inventory) |

---

## 4. Doc cross-ref

| Topic | Specs |
|-------|-------|
| 5 WHYs / FP | [001](./001-five-whys-first-principles.md) |
| Code inventory | [002](./002-code-is-law-inventory.md) |
| Lenses | [003](./003-lens-product-owner.md)–[008](./008-lens-fullstack.md) |
| Screens | [009](./009-screens-ascii.md) |
| Components / nav | [010](./010-components-navigation-ascii.md) |
| State machines | [011](./011-state-machines.md) |
| Normative contract | [012](./012-target-ux-contract.md) |
| Roadmap | [014](./014-implementation-roadmap.md) |
| Ingest speed (companion) | [047/016](../047-rag-evaluation/016-ingest-speed-reliability-battle-plan.md) |

---

## 5. Sign-off checklist

- [ ] AC-01…AC-12 assigned to P0/P1 in [014](./014-implementation-roadmap.md)  
- [ ] DEF-01…DEF-10 each have owner (FE/BE)  
- [ ] Playwright scenario “busy vs completed skew” fails on main, passes on 048 branch  
- [ ] CHANGELOG entry when implementation starts  

**Filename note:** index links this file as `013-acceptance-criteria-crossref.md`.
