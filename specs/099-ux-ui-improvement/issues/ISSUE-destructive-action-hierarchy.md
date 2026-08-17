# ISSUE — Destructive action hierarchy + selection mode

| Field | Value |
|-------|-------|
| ID | ISSUE-destructive-action-hierarchy |
| Findings | F-099-05, F-099-16 |
| Laws | LAW-099-5, LAW-099-9 |
| Wave | W5 |
| Status | Open |
| Supersedes | SPEC-030 F-DOC-01 · SPEC-029 DI-02 |

## Problem

**Clear All** sits peer to **Refresh** in the Documents header (evidence 01). Confirmation dialog exists (`clear-documents-dialog.tsx`) but proximity to a benign control increases slips ([NN/g proximity of consequential options](https://www.nngroup.com/articles/proximity-consequential-options/)). Separately, selection mode stacks a second toolbar row under search/filters (SPEC-029 DI-02).

## Why it hurts UX

Clear All wipes knowledge-graph-backed corpora — high consequence. Stacked toolbars add chrome when users are already in a high-load multi-select + busy-run state.

## Approach

1. Move Clear All behind overflow (`…`) or a clearly labeled Danger control — not peer to Refresh.  
2. Retain typed confirmation dialog.  
3. Selection mode **replaces** the primary toolbar row (Gmail/Linear pattern): `[✕] N selected · Reprocess · Delete`.  
4. Ensure focus management (LENS-accessibility).

## DoD

- [ ] `spec099-clear-all-demoted` green  
- [ ] `spec099-selection-toolbar` green  
- [ ] Typed confirm still required  
- [ ] Bulk delete honesty (`spec098`) green  

## Non-goals

Removing Clear All entirely; soft-delete trash UX (future spec).
