# ISSUE — Typed fleet mirror resolved 0/N

## Repro (observed)

- UI: Documents → `pro_long_2607.20064v2.draft.md` → **Failed**  
- Error: `Knowledge graph persist failed` / `SPEC-091: typed fleet mirror resolved 0/18 rows…`  
- Entities count shown (e.g. 36) before persist fail — extraction succeeded; CQRS spine/fleet failed.

## Root cause

Saturated SOURCE_IDS KEEP skipped `PostgresEntitySink` for entities that already existed in AGE, while `collect_entity_vector_batch` still emitted embeddings. Under typed authority, fleet mirror requires relational FKs → `0/N`.

## Fix (SPEC-098)

1. Spine ensure on saturated KEEP (AGE still skipped).  
2. Relation-type uppercase SSOT.  
3. Fail closed on partial miss + loud invalid workspace.  
4. Migration 139 historical reconcile.

## Acceptance

Reprocess the failing document → Completed; `entity_embeddings` rows present for workspace.
