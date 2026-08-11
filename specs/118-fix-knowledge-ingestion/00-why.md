# 00 — Why SPEC-118

## Trigger

Partners (including Enterprise-Brain glossary bootstrap) create knowledge injections via:

```bash
PUT /api/v1/workspaces/{workspace_id}/injection
```

Under default SPEC-091 Wave D authority (`EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational`), the `knowledge_injection` worker permanently fails:

```text
Storage error: Invalid data: invalid uuid
  'injection::{workspace_id}::{injection_id}':
  invalid length: found 85
```

Observed on `ghcr.io/raphaelmansuy/edgequake:latest` (v0.24.x lineage) — [GitHub #376](https://github.com/raphaelmansuy/edgequake/issues/376).

## Gaps

| Approach | Gap |
|----------|-----|
| SPEC-0002 composite `injection::` doc IDs | Correct for citation exclusion + graph tagging; **not** a UUID |
| SPEC-091 relational writer | Requires `documents.id` UUID FK; `Uuid::parse_str` on full composite → hard fail |
| Wave B6 `injection_relational` | Already stores injection metadata under bare injection UUID — unused by chunk writer |
| Soft-skip elsewhere (`document_stage_mirror`, typed embeddings) | Asymmetric: chunk writer hard-fails; embeddings soft-return `Ok(0)` |
| CI injection e2e | `AppState::test_state()` / worker harness pin `CHUNK_TEXT_AUTHORITY=kv` → blind spot |

## Partner impact

- Glossary / enrichment injections never complete → status `failed` after 3 retries
- Graph bootstrap paths that depend on injection silently degrade RAG quality
- Docker/latest with PostgreSQL defaults is broken for injection even when LLM/embeddings are healthy

## Non-goals

- Changing the public `injection::{` composite format (citation contract)
- Soft-skip-only hotfix that leaves relational chunk SSOT empty
- Schema migration (parent row already exists)
- Fixing unrelated SPEC-058 vector dimension mismatches on legacy workspaces (noted in repro)

## Success

1. With `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational` + Postgres, injection reaches `completed`.
2. `public.chunks` rows exist for `document_id = injection_id` (UUID).
3. Typed embeddings resolve those chunks (same identity helper).
4. Query citations still exclude `injection::` sources.
5. Delete injection cascades relational chunks; graph cleanup still uses composite doc_id.
6. Unit + PG e2e cover the relational path (CI no longer hides the bug).
