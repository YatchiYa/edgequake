# ISSUE — Delete failure honesty (lifecycle mislabeled as Failed)

## Repro (observed)

- UI: Documents → selected bulk delete of hyper-connection docs → cascade fails.
- Feedback: **Deleting N document(s)** with red X, title **Delete failed**, detail **Deletion failed**.
- Table: same rows show pipeline **Failed**; **Retry Failed (N)** appears.
- Docs remain in the list (cascade fail-closed — correct presence, wrong label).

## Root cause

1. **F-098-16** — Shell normalize collapses `delete_failed`→`failed` / `deleting`→`cancelled`.  
2. **F-098-17** — Batch result has `failed_ids` only; FE uses generic “Deletion failed”.  
3. **F-098-18** — Feedback header counts failed sessions as “Deleting”; admit SQL CHECK miss can 500 after enqueue.  
4. **F-098-19** — Retry Failed / badge treat lifecycle failure as pipeline failure.

## Fix (SPEC-098 W12)

1. Pass through lifecycle statuses in shell normalizer (mig 141 CHECK).  
2. Batch `failed: [{document_id, reason}]` + `reset_deleting_status` on every failure branch.  
3. Admit: KV hard; SQL mirror warn-not-fail after enqueue.  
4. FE: header split, `delete_failed` badge, Retry Failed excludes lifecycle.  
5. E2E + Playwright gates.

## Acceptance

- Table + feedback show **Delete failed** (not pipeline Failed / not “Deleting N” when all failed).  
- Batch task result includes per-id reasons displayed in the panel.  
- Shell upsert cannot rewrite lifecycle statuses.  
- Retry Failed does not count `delete_failed` docs.
