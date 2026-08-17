# ISSUE — Inventory scale + filter honesty

| Field | Value |
|-------|-------|
| ID | ISSUE-inventory-scale-honesty |
| Findings | F-099-09, F-099-10, F-099-11, F-099-12, F-099-13 |
| Laws | LAW-099-7, LAW-099-8, LAW-099-4, LAW-099-10 |
| Wave | W7–W8 |
| Status | Open |
| Related | GH-319 pagination · SPEC-030 F-DOC-02/03/04 |

## Problem

1. `VIRTUAL_PAGE_SIZE = 100` in `document-manager.tsx` silently caps fetch — UI can imply a complete corpus.  
2. Evidence 02: header **Documents 17** vs filter **All Status (11)** — count parity break.  
3. `NEW` badge + always-on Cost column add idle scan noise.  
4. `ux-ui-audit.spec.ts` may miss `data-testid="document-dropzone"`.

## Why it hurts UX

Operators cannot trust the inventory as SSOT for “what is in this workspace.” Noise slows scanning for status anomalies.

## Approach

```ascii
 inventoryViewModel
   totalKnown / pageSize / isTruncated
   filteredCount
   rows[]
        │
        ├─ Header: "Documents 17" or "Documents 17 of 240"
        ├─ Filter chip: "All Status (17)"  ← same filteredCount when All
        └─ Table body: rows.length === filteredCount (client filter)
```

1. One view-model drives header, chips, and rows (LAW-099-8).  
2. When truncated, show overflow affordance / “showing N of M” (LAW-099-7). Server pagination may remain GH-319; UI honesty is mandatory now.  
3. Demote or remove NEW badge; Cost default-hidden or column toggle (progressive disclosure).  
4. Fix audit selectors (F-099-13).

## DoD

- [ ] `spec099-filter-count-parity` green  
- [ ] `spec099-scale-overflow` green (or equivalent unit)  
- [ ] NEW/Cost noise reduced per LENS-progressive-disclosure  
- [ ] ux-ui-audit locates dropzone  

## Non-goals

Full server-side cursor pagination implementation (track under GH-319); may stub totals if API already returns `total`.
