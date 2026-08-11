# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Bulk upload excessively slow (partner) | [#361](https://github.com/raphaelmansuy/edgequake/issues/361), [#365](https://github.com/raphaelmansuy/edgequake/issues/365) |
| Capacity / expectation, not confirmed logic bug | Maintainer comments on #361; `specs/111-issues/issue-361-bulk-upload.md` |
| WebUI max 3 concurrent file transfers | `edgequake_webui/src/lib/upload/bounded-file-upload.ts` |
| WebUI uses N× single-file admits (not `/upload/batch`) | `use-file-upload.ts`, `perform-file-upload.ts`; SPEC-098 |
| Batch multipart serial admit, ≤20 files | `batch_upload.rs`; `MAX_BATCH_UPLOAD_FILES` |
| Local make: tenant=1, extract=1, vision=1 | `Makefile` provider-aware block |
| Docker default: workers=8, tenant=6, extract=4 | `edgequake/docker/docker-compose.yml` |
| Cloud make: workers=16, tenant=12, extract=32 | `Makefile` else branch |
| PDF convert → separate Insert (SPEC-057) | `pdf_processing.rs` / task types |
| Extract semaphore | `edgequake-pipeline/.../extraction.rs` |
| Embed `buffer_unordered` | `pipeline/helpers/embeddings.rs` |
| Queue observability | `GET /api/v1/pipeline/queue-metrics` |
| DB write amplification under concurrency | SPEC-090 |
| Ollama parallel slots | https://docs.ollama.com/faq (`OLLAMA_NUM_PARALLEL`) |
| Laws | LAW-122-1..10 ([01-first-principles.md](01-first-principles.md)) |

## Code SSOT (as-is → target)

| Concern | Path |
|---------|------|
| WebUI transfer bound | `edgequake_webui/src/lib/upload/bounded-file-upload.ts` |
| WebUI upload orchestration | `edgequake_webui/src/hooks/use-file-upload.ts` |
| Per-file admit router | `edgequake_webui/src/lib/upload/perform-file-upload.ts` |
| Batch API | `edgequake-api/.../upload/batch_upload.rs` |
| Document admission | `.../upload/document_admission.rs` |
| PDF admit helpers | `.../pdf_upload/helpers.rs` |
| PDF convert + resource profile | `.../processor/pdf_processing.rs` |
| Worker pool / claim | `edgequake-tasks/src/worker.rs` |
| Extract concurrency | `edgequake-pipeline/src/pipeline/extraction.rs` |
| Embed async | `edgequake-pipeline/src/pipeline/helpers/embeddings.rs` |
| Resource budget constants | `edgequake-core/src/resource/budget.rs` |
| Local clamps | `edgequake-pipeline/.../config.rs` (`LOCAL_*`) |
| Make profile SSOT | `Makefile` ~333–370 |
| Docker profile SSOT | `edgequake/docker/docker-compose.yml` |
| Perf ops guide | `docs/operations/performance-tuning.md` |
| Quick-start local clamp note | `docs/getting-started/quick-start.md` |
| FAQ fairness / park | `docs/faq.md` |

## Related specs / issues

| Spec / Issue | Relationship |
|--------------|--------------|
| GH #361 / #365 | This mission (duplicates) |
| SPEC-111 issue-361 note | Prior measure-only stub — superseded by this pack |
| SPEC-090 | DB counter serialization / claim_next under load |
| SPEC-091 | Queue admission first principles; weak ETA |
| SPEC-057 | Fairness park; PDF convert→ingest dual task |
| SPEC-083 | PDF concurrency honesty (X-12) |
| SPEC-095 | pdfium concurrent bind risks |
| SPEC-098 / GH-350 | WebUI bulk ops vs batch API |
| SPEC-024 | Async / batch upload product |
| SPEC-121 | Format matrix — orthogonal; PDF path supported |
| SPEC-038 | Large PDF ingest |

## DRY rule

One **concurrency matrix** drives:

1. Makefile local/cloud defaults  
2. docker-compose env defaults  
3. FAQ / performance-tuning / quick-start copy  
4. UI bulk progress / ETA messaging  
5. E2E measurement harness expectations  

Do not invent a fifth “batch mode” that bypasses fairness or provider budget (LAW-122-7, LAW-122-9).
