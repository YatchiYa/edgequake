# LENS — Full Stack Developer (SPEC-114)

## End-to-end path

```ascii
WizardDraft.relationTypes
        │
        ▼
buildWorkspaceUpdatePayload()
        │
        ▼
PUT /api/v1/workspaces/{id}
        │
        ▼
apply_relation_types*_metadata → workspaces.metadata JSONB
        │
        ▼
EntityExtractionSchema::from_workspace_metadata  (+ relation fields)
        │
        ▼
workspace_pipeline_factory → JSON prompt + enforce_relation_type
```

## Touchpoints

| Layer | Files |
|-------|-------|
| Core types | `types/multitenancy/requests.rs`, workspace helpers |
| API DTOs | `handlers/workspaces_types/{requests,responses}.rs` |
| Pipeline | `entity_type_policy.rs`, `json_prompts.rs`, parser/gleaning |
| FE draft | `wizard-state.ts`, `model-payload.ts`, `workspace-config-diff.ts` |
| FE UI | `RelationTypeSelector`, `KgSchemaPreview`, extraction step |

## DRY rules

1. One `normalize_type_list` for entity and relation.  
2. Sparse strict encoding mirrors entity (`true` = key absent).  
3. Do not invent a second OpenAPI shape — extend workspace resources.  
4. Share chip/bulk UI primitives; colors remain entity-only (SPEC-102).

## Failure modes

| Failure | Handling |
|---------|----------|
| Invalid types | Normalize/skip empty; cap 50 |
| Old clients omit fields | Free-form relations (compat) |
| Orchestrator path | Keep entity `with_types` behavior; document if relations not on that path |

## OpenAPI

Refresh generated client/docs when DTO fields land (`make codegen-openapi-refresh` if required by repo).
