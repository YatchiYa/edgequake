# 07 — Implementation Plan

## Principles

- **DRY:** one `resolve_relational_document_id` SSOT
- **SOLID:** pure resolver; writers stay persist-only; do not couple to injection CRUD
- **First principles:** map for typed FK; keep composite for citations
- **Test first:** unit contracts for resolve + writer; then PG relational e2e

## Phase A — Resolver SSOT

1. Add `edgequake-pipeline/src/persistence/document_id_resolve.rs`
2. Export from `persistence/mod.rs`
3. Unit tests:
   - bare UUID → Ok
   - `injection::{ws}::{uuid}` → Ok(trailing)
   - `injection::only-one-part` → Err
   - `not-a-uuid` → Err
   - extra segments / empty trailing → Err

## Phase B — Wire writers

1. `relational_chunk_writer::parse_document_id` → call resolver
2. When `is_injection_composite_document_id`, set `metadata["legacy_document_id"] = ctx.document_id`
3. `typed_embedding_writer` → resolve; on Err return `Ok(0)` (preserve soft policy for unknown ids)
4. Keep existing reject contract for garbage ids in chunk writer

## Phase C — Tests / CI blind spot

1. Extend writer tests: `contract_spec118_*`
2. Add PG e2e (or extend SPEC-091 fixture) that:
   - sets `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational`
   - runs injection pipeline with `relational_chunks` Some
   - asserts `chunks.document_id == injection_uuid`
3. Keep citation exclusion e2e green
4. Authority `kv` regression still passes

## Phase D — Docs / GitHub

1. Update SPEC-118 status board when code lands
2. Comment on #376 with links + decision

## Edge-case matrix

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC-01 | Bare UUID doc | Unchanged path | unit |
| EC-02 | Valid `injection::ws::uuid` | Map trailing UUID | unit + e2e |
| EC-03 | Garbage non-UUID | Fail-closed chunk writer | unit |
| EC-04 | Malformed `injection::x` | Err | unit |
| EC-05 | Trailing non-UUID | Err | unit |
| EC-06 | Typed embeddings after map | load_for_document works | unit/integration |
| EC-07 | Citations exclude injection | Prefix filter unchanged | e2e_injection |
| EC-08 | Delete injection | Cascade chunks by UUID + graph composite cleanup | e2e |
| EC-09 | Authority `kv` | No regression | existing e2e |
| EC-10 | Parent missing (race) | `ensure_document_parents` | integration |
| EC-11 | Quarantine path needs UUID | Mapped UUID satisfies | note |
| EC-12 | Workspace path remap / dim mismatch | Out of scope; document in repro | 10-reproduction |

## Rollout

1. Land code + tests
2. No feature flag required (behavior additive for known pattern)
3. Verify on local PG with matching embedding dims
4. Close #376 after acceptance checklist green
