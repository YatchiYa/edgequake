# SPEC-050 Screenshots — E2E Proof

All screenshots taken from live app at http://localhost:3010 with real data.

## 01 — Documents Page Overview
![Overview](01-documents-page-overview.png)
Documents list showing 8 documents with Completed status and the new Refresh / Clear All buttons.

## 03 — Delete Confirm Dialog with Shared Entity Semantics
![Delete Dialog](03-delete-confirm-with-shared-entities.png)

**Proves:**
- AC-050-01: Impact preview loaded BEFORE confirm is possible
- SPEC-050/EC-1: "227 entities" under **PERMANENTLY REMOVED** (red) — exclusive to this doc
- SPEC-050/EC-2: "653 entities will survive" + "2,389 relationships will survive" under **SURVIVE (SHARED WITH OTHER DOCUMENTS)** (amber)
- Blue info banner: "Shared entities and relationships will NOT be deleted — they survive with evidence from other documents"
- Cancel + "Delete permanently" buttons

## 05 — Bulk Delete Confirm Dialog (2 documents)
![Bulk Delete](05-bulk-delete-confirm-2docs.png)

**Proves:**
- Gap 2 fix: toolbar "Delete" now opens a confirm dialog (no more direct mutation)
- Lists both selected documents: `PIP_Seniors-and-Tech-Use_040314.pdf` and `measuringsuccessonfacebook...`
- Graph impact warning
- "Delete 2 document(s)" confirm button

## 06 — Reprocess Actions Menu
![Reprocess Menu](06-reprocess-actions-menu.png)
Document actions menu showing Reprocess option.

## 07 — Reprocess Dialog
![Reprocess Dialog](07-reprocess-dialog.png)
ReprocessDialog open with "Re-extract entities only" option selected.

## 08 — After Reprocess (Row Updated)
![Reprocess Result](08-reprocess-queued-state.png)
After confirming reprocess — the document was processed (small workspace, processed immediately).
With the Gap 1 fix, the row now shows optimistic "Queued" state immediately on confirm,
using the new `track_id` from the response for WS subscription.

---

## Backend Proof: Resource Safety Tests

```
test resource_safety_delete_cascade_bounded_scope ... ok   ← Shared entity handling
test resource_safety_cascade_tenant_isolation ... ok       ← Tenant isolation
19 passed; 0 failed                                        ← All safety proofs pass
```

Run: `cargo test -p edgequake-api --test resource_safety_proof`
