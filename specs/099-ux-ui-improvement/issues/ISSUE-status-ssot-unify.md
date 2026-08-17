# ISSUE — Unify status domain SSOT

| Field | Value |
|-------|-------|
| ID | ISSUE-status-ssot-unify |
| Findings | F-099-01, F-099-15 |
| Laws | LAW-099-1 |
| Wave | W1 |
| Status | Open |

## Problem

`lib/documents/status-domain.ts` and `components/documents/status-badge.tsx` both export `normalizeStatus`, `isProcessingStatus`, `isTerminalStatus`, and `getDocumentDisplayStatus`. List merge uses domain; many UI paths and tests import badge. Coverage diverges (e.g. `cancelling` / `held` / `dead_letter`). Failed row highlight uses raw `doc.status === 'failed'`, missing `delete_failed`.

## Why it hurts UX

Merge and paint can disagree → operator sees Completed while filters say processing (or the reverse). Lifecycle honesty from SPEC-098 depends on one predicate set.

## Approach

1. Make `status-domain.ts` the only implementation.  
2. Change `status-badge.tsx` to presentation config + thin wrappers that re-export from domain **or** delete re-exports and migrate all imports. Prefer **no re-export** (forces compile-time migration).  
3. Point `document-status.ts`, `ingestion-run-view.ts`, hooks, and tests at domain.  
4. Failed highlight via `getDocumentDisplayStatus` / domain terminal-failure helpers.  
5. Unit gate: badge does not export domain helpers.

## DoD

- [ ] No duplicate helper bodies  
- [ ] `status-domain.test.ts` covers former badge edge cases  
- [ ] `spec098-bulk-delete-honesty` still green  
- [ ] F-099-15 highlight uses domain  

## Non-goals

Changing backend status enums; changing merge ranking policy beyond consolidating the function source.
