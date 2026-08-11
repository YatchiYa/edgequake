# 09 — Acceptance

## Pass

- [x] Shared resolver maps `injection::{ws}::{uuid}` → injection UUID (unit: issue #376 len-85)
- [x] `public.chunks`-shaped persist succeeds for composite ids (`MemoryChunkRepository` e2e)
- [x] Chunk metadata includes `legacy_document_id` for bridged injections
- [x] Typed embeddings use the same resolver SSOT
- [x] Garbage non-UUID still fail-closed in chunk writer
- [x] GitHub #376 commented with SPEC-118 links and dual-identity decision
- [x] Live API smoke (v0.24.3 release, authority=relational): PUT → `completed`, chunks≥1, legacy bridge set
- [x] Query sources contain no `injection::` document ids (worker e2e)
- [x] Delete injection cascades relational chunks (worker e2e + live smoke → 0 rows)
- [x] CI blind-spot closed: `e2e_spec118_injection_relational_pg` pins `relational` (not kv harness)
- [x] DELETE/PATCH typed-first meta load (parity with GET under relational injection family)

## Fail

- [ ] Soft-skip relational writes for `injection::` leaving empty typed SSOT
- [ ] Changing `injection_doc_id` to bare UUID (breaks citation exclusion)
- [ ] Duplicated injection parse logic in API and pipeline
- [ ] CI only covers `kv` authority for injection

## Sign-off

| Role | Sign |
|------|------|
| PO | Injection works on product default path (live smoke completed) |
| Fullstack | Resolver SSOT wired; relational PG e2e green |
| DB | FK + cascade verified (live + e2e) |
| KG | Citations clean in worker e2e; graph provenance preserved |
