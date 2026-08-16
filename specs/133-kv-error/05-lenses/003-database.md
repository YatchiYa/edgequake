# Lens 003 — Database Expert

## Integrity model

```ascii
  entities(id, workspace_id, name)          ← parent
  relationships(id, source_id, target_id, …) ← FK parents
  relationship_embeddings(relationship_id, …)← FK to relationships
```

Mirror misses are **application-level wrong lookups**, not deferred-constraint
ordering bugs. Parents usually exist; the child write never runs because resolve
returns `None`.

## What NOT to do

| Anti-pattern | Why |
|--------------|-----|
| Re-run 139/140 solely for `995/1000` arrow samples | Ops runbook already warns — wrong class |
| `SET CONSTRAINTS DEFERRED` / disable triggers | Masks symptoms; doesn't fix parse |
| Delete orphan embeddings manually as “fix” | Leaves document Failed; no identity repair |

## Useful diagnostics

```sql
-- Do both intended endpoints exist?
SELECT name FROM entities
WHERE workspace_id = $ws
  AND name IN ('FLOW_DIRECTION', 'ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET)');

-- Is the relationship spine row present for intended pair?
SELECT r.id, e1.name, e2.name, r.relation_type
FROM relationships r
JOIN entities e1 ON e1.id = r.source_id
JOIN entities e2 ON e2.id = r.target_id
WHERE r.workspace_id = $ws
  AND e1.name = 'FLOW_DIRECTION'
  AND e2.name LIKE 'ARROW_1_%';
```

## Index / plan impact

Index-guided parse is CPU-side over an already-loaded `EntityNameIndex` (one
workspace load per mirror batch). No new SQL indexes required for WP-1.

## Cross-refs

- Laws: [../01-first-principles.md](../01-first-principles.md)
- Ops parent: [`docs/operations/spec098-entity-spine-ensure.md`](../../../docs/operations/spec098-entity-spine-ensure.md)
