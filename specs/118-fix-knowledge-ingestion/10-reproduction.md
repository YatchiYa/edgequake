# 10 — Reproduction

## Goal

Reproduce [GitHub #376](https://github.com/raphaelmansuy/edgequake/issues/376) against local EdgeQuake + PostgreSQL.

## Environment (2026-08-11)

| Item | Value |
|------|-------|
| API | `http://localhost:8090` |
| Version | `0.24.2` (`git_hash` 48804520c) |
| Storage | postgresql |
| LLM | mistral / `mistral-small-latest` |
| Embedding | mistral-embed **1024** |
| `EDGEQUAKE_CHUNK_TEXT_AUTHORITY` | **relational** (process env confirmed) |
| Auth | `EDGEQUAKE_AUTH_ENABLED=false`, `EDGEQUAKE_DEV_MODE=true` |

## Steps attempted

### 1. Confirm authority

Process env on listener PID includes `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational`.

### 2. PUT injection (default tenant workspace)

```bash
curl -X PUT "http://localhost:8090/api/v1/workspaces/00000000-0000-0000-0000-000000000003/injection" \
  -H "Content-Type: application/json" \
  -d '{"name":"SPEC-118 Repro Glossary","content":"Enterprise Brain is a knowledge system. Glossary Term Alpha relates to Term Beta."}'
```

**Result:** HTTP 202 → status `failed` after 3 retries.

**Observed error (local):** SPEC-058 vector dimension mismatch on default workspace vector table (`stored=768`, `required=1024`) — failure occurs **before** relational chunk UUID parse.

### 3. Typed row still created (identity evidence)

Injection id example: `91f261fc-814a-4721-a71c-dbe07f6453f2`

```sql
SELECT id, status, metadata->>'source_document_id' AS source_document_id
FROM documents WHERE id = '91f261fc-814a-4721-a71c-dbe07f6453f2';
-- source_document_id =
--   injection::00000000-0000-0000-0000-000000000003::91f261fc-814a-4721-a71c-dbe07f6453f2

SELECT COUNT(*) FROM chunks WHERE document_id = '91f261fc-814a-4721-a71c-dbe07f6453f2';
-- 0
```

### 4. Path workspace remapped

`put_injection` uses `workspace_id_from_tenant` and **does not** bind path `{workspace_id}`. Creating a fresh 1024-dim workspace and PUTting to its path still wrote `workspace_id=00000000-…0003` in logs/response.

### 5. Issue #376 error shape (unit confirmation)

Composite length matches reporter:

```text
injection::00000000-0000-0000-0000-000000000000::3fc4a415-33e7-4a38-88d9-86ae6b8bb36e
len = 85
```

Code SSOT hard-fails:

```text
relational_chunk_writer::parse_document_id
  → Uuid::parse_str(full composite)
  → StorageError::InvalidData("invalid uuid '…': …")
```

Existing contract `contract_spec091_build_relational_chunks_rejects_bad_document_id` confirms fail-closed for non-UUID strings on the same path.

## Verdict

| Layer | Finding |
|-------|---------|
| Live default WS | Blocked by SPEC-058 dim mismatch before #376 parser |
| Typed documents | Confirms dual identity already (`id` UUID + `source_document_id` composite) |
| Code path | Confirms #376 will fire once vector storage succeeds under relational authority |
| CI | Still blind (`kv` pin / no pg relational injection e2e) |

## Follow-up for full live #376 line

Use a tenant default workspace whose vector table dimension matches the active embedder (or rebuild with `EDGEQUAKE_ALLOW_VECTOR_TABLE_REBUILD=1` in a disposable DB), then re-run PUT — expect the exact `invalid uuid 'injection::…'` log line pre-fix, and `completed` + non-zero `chunks` post-fix.

## Post-fix live smoke (2026-08-11, v0.24.3)

Rebuilt `target/release/edgequake`, restarted on `:8090` with `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational` (+ `EDGEQUAKE_ALLOW_VECTOR_TABLE_REBUILD=1` for SPEC-058 dim mismatch on default WS).

| Step | Result |
|------|--------|
| PUT injection | HTTP 202 |
| Status | **completed** |
| `chunks` for injection UUID | **1** with `legacy_document_id` bridge |
| DELETE | HTTP 200; chunks after = **0** |
| `invalid uuid 'injection::…'` | none |

Worker e2e: `e2e_spec118_injection_relational_pg` 3/3 green under relational authority.
