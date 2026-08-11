# 06 — UX / UI Spec

## Happy path

1. User uploads / syncs two documents that share a new entity name.
2. Both process concurrently.
3. Both reach Completed.
4. Knowledge graph shows the entity; RAG is usable.

No new screens, modals, or status chips.

## Failure path (must disappear for this class)

```text
BEFORE (bug):
  status = Failed
  message ≈ duplicate key … idx_entity_embeddings_legacy_vector_id

AFTER (target):
  status = Completed (or normal processing continuum)
  message does not mention legacy_vector_id / 23505
```

## Unrelated failures

Other GraphMerge / StorageError classes keep existing copy. Do not blanket-swallow all unique violations outside the absorb helper.

## Accessibility / i18n

N/A — no new user-facing strings in v1.
