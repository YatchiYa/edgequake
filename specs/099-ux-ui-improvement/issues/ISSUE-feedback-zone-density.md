# ISSUE — Feedback zone density + single narrative

| Field | Value |
|-------|-------|
| ID | ISSUE-feedback-zone-density |
| Findings | F-099-03, F-099-06, F-099-07, F-099-14 |
| Laws | LAW-099-2, LAW-099-4, LAW-099-6 |
| Wave | W3–W4 |
| Status | Open |
| Inherits | SPEC-048 Active runs · SPEC-086 dual-run |

## Problem

Busy upload shows tall Active runs cards (full stepper + dual progress), table badges for the same docs, optional pipeline banner, and toast — triple/quadruple narrative. Zone is capped at 35vh but card density fills it with few items. Evidence: [`evidence/02-busy-active-runs.png`](../evidence/02-busy-active-runs.png), [`evidence/03-legacy-active-card.png`](../evidence/03-legacy-active-card.png).

## Why it hurts UX

Inventory disappears below the fold; operators cannot monitor corpus while admitting files; NN/g progressive disclosure fails.

## Approach

1. Compact `IngestionRunCard` — single progress + phase strip; expand-on-click for full stepper.  
2. Table: hide stage subtitle for live ids; demote pulsing badge chrome when zone owns narrative.  
3. Demote non-stuck pipeline banner when feedback zone open.  
4. Toast demotion owned by sibling issue `ISSUE-upload-slot-collapse` / LAW-099-6.  
5. Keep dual surfaces: zone = narrative, table = inventory (do **not** remove Active runs).

## DoD

- [ ] `spec099-feedback-viewport` green  
- [ ] `spec099-live-row-no-stage-subtitle` green  
- [ ] `spec099-banner-demote` green  
- [ ] `spec048` / `spec086` green  

## Non-goals

Removing feedback zone; moving stepper into the table.
