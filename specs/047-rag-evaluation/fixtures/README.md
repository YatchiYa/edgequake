# SPEC-047 fixtures

Committed lists only — **no PDFs**.

| File | Purpose |
|------|---------|
| `smoke_doc_ids_v1.txt` | 10-doc smoke set (placeholder until EQ-047-05 freezes real IDs) |
| `smoke_selection_rationale_v1.md` | How/why the 10 were chosen |
| `core_doc_ids_v1.txt` | Placeholder for ~40-doc core set |

## Freeze process (EQ-047-05)

1. Download official Q&A + PDF manifest.  
2. Run stratification algorithm in [003](../003-fair-evaluation-protocol.md) with seed `047-smoke-v1`.  
3. Replace placeholder IDs with real `doc_id` values from the dataset.  
4. Record dataset revision in this folder’s rationale file.  
5. Never edit IDs after first valid smoke baseline without bumping to `_v2`.
