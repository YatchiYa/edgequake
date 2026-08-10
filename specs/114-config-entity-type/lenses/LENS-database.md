# LENS — Database Expert (SPEC-114)

## Storage decision

**No migration.** All keys live in `workspaces.metadata` JSONB (same as `entity_types`, `extraction_language`, `entity_type_colors`).

```ascii
workspaces
  id | tenant_id | slug | settings JSONB | metadata JSONB | …
                                              │
                                              ├─ entity_types: string[]
                                              ├─ entity_types_strict: bool?  (sparse)
                                              ├─ relation_types: string[]     NEW
                                              ├─ relation_types_strict: bool? NEW
                                              ├─ kg_schema_preset: string?    NEW
                                              └─ entity_type_colors: object
```

## Why not columns / tables

| Option | Rejected because |
|--------|------------------|
| SQL columns | SPEC-096/102 established JSONB; avoid migration churn |
| `kg_schema` table | Overkill for allow-lists; no versioning requirement in v1 |
| AGE labels as config | Observed ≠ configured; graph is output, not SSOT |

## Constraints

| Rule | Value |
|------|-------|
| Max types per list | 50 |
| Normalization | UPPER_SNAKE; dedupe; order-preserving |
| Strict sparse | `true` → remove key; `false` → store false |
| Empty relation list | Remove key → free-form at runtime |
| `settings` JSONB | Unused for this feature |

## Observed vs configured

Document-level `relationship_types` stats and `GET /graph/labels` are **derived**. Never treat them as workspace config. UI copy must not conflate the two (EC-114-15).

## Indexes

None required for v1 (workspace row already loaded by id).
