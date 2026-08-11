# 08 — Test Protocol

## Unit (pipeline)

| Test | Assert |
|------|--------|
| `contract_spec118_resolve_bare_uuid` | Ok(DocumentId) |
| `contract_spec118_resolve_injection_composite` | Trailing UUID |
| `contract_spec118_resolve_rejects_garbage` | InvalidData |
| `contract_spec118_resolve_rejects_malformed_injection` | InvalidData |
| `contract_spec118_build_chunks_maps_injection_doc_id` | chunks[0].document_id == inj UUID; metadata.legacy_document_id set |
| `contract_spec091_build_relational_chunks_rejects_bad_document_id` | Still rejects `not-a-uuid` |

Commands:

```bash
cargo test -p edgequake-pipeline --lib persistence::document_id_resolve
cargo test -p edgequake-pipeline --lib persistence::relational_chunk_writer
```

## API / e2e

| Test | Assert |
|------|--------|
| Existing `e2e_injection` citation tests | No `injection::` in sources |
| New / extended PG relational injection test | Task completed; `chunks` for injection UUID; authority=relational |

Harness requirements for the new test:

```bash
export EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational
# Do NOT force kv in this specific test module
# Provide pg_pool + ChunkRepository
```

## Authority matrix

| Authority | Expect |
|-----------|--------|
| relational | chunks written via mapped UUID |
| dual | chunks written |
| kv | injection still completes (legacy) |

## Manual smoke (local)

```bash
# workspace with matching embedding dimension
curl -X PUT "http://localhost:8090/api/v1/workspaces/$WS/injection" \
  -H "Content-Type: application/json" \
  -d '{"name":"Glossary","content":"Term A relates to Term B."}'
# poll until completed; SQL: SELECT count(*) FROM chunks WHERE document_id = $INJ
```

## Exit criteria

All unit contracts green; at least one relational-authority injection path green; citation exclusion green.
