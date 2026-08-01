# ISSUE — Serving fence presentation (StatusCell)

| Field | Value |
|-------|-------|
| ID | ISSUE-serving-fence-presentation |
| Findings | F-099-02 |
| Laws | LAW-099-3 |
| Wave | W2 |
| Status | Open |
| Inherits | SPEC-091 IS3 / LD-09 (`query_ready`) — **semantics kept** |

## Problem

`EnhancedStatusBadge` renders `StatusBadge` and `ServingFenceBadge` as sibling emerald success pills (`Completed` + `Ready`). Operators read dual success; fence meaning (queryable vs indexed) is lost. Evidence: [`evidence/01-idle-completed.png`](../evidence/01-idle-completed.png).

## Why it hurts UX

Cognitive load on every completed row; weakens the actual fence signal; compounds mid-delete honesty work (SPEC-098 fights Completed/Ready flash).

## Approach (locked)

Composite **StatusCell** — one visual cell:

```ascii
 Completed · Ready              (query_ready === true)
 Indexed · not queryable        (query_ready === false) — amber secondary
 Completed                      (query_ready null/undefined — no fence)
```

Keep `data-testid="spec091-serving-fence-badge"` and `data-query-ready` for SPEC-091 Playwright, or equivalent attributes on the composite cell.

## DoD

- [ ] No peer dual green pills  
- [ ] `spec091-ingestion-surface` green  
- [ ] `spec099-status-cell-fence` green  
- [ ] a11y name includes fence  

## Non-goals

Removing `query_ready` from the API; changing when the fence appears.
