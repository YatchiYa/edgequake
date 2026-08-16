# Lens 003 — Database Expert

## Constraint SSOT

Migration `141_spec098_document_lifecycle_status.sql` defines `documents_valid_status`:

```sql
status IN (
  'pending', 'processing', 'chunking', 'extracting', 'embedding', 'indexing',
  'completed', 'indexed', 'failed', 'partial_failure', 'cancelled',
  'deleting', 'delete_failed'
)
```

## Decision: no DDL

Adding `re_embedding` (and every other UI slug) to CHECK would:

- Couple schema churn to UX stage invention
- Duplicate the rich vocabulary already in KV
- Fight LAW-129-1 / SPEC-098 shell design

Application projection is the correct layer ([PostgreSQL dual-write / expand-contract patterns](https://www.devopsness.com/blog/database-migrations-without-downtime-patterns-from-three-real-cutovers-2026-04-05) keep CHECK at the stable grain).

## Verification SQL (e2e)

```ascii
  INSERT documents status='failed'
  UPDATE status='re_embedding'     → expect FAIL (check)
  touch_document_status(..., 're_embedding') → expect OK, status='processing'
  UPDATE status='deleting'         → expect OK (lifecycle)
```

## Cross-refs

- Edge cases: [../10-edge-cases.md](../10-edge-cases.md)
- SPEC-098: [../../098-data-access-hardening/](../../098-data-access-hardening/)
