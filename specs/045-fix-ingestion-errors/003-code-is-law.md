# SPEC-045 — Code is Law

Every claim maps to live source (baseline: **current `main`**, July 2026).

---

## 1. Ingestion pipeline (happy path)

| Claim | File | Symbol / lines |
| ----- | ---- | -------------- |
| Upload admission sets `pending` | `handlers/documents/upload/document_admission.rs` | admission flow |
| Worker dispatches by `TaskType` | `processor/task_impl.rs` | `process()` L11–59 |
| Text insert stages | `processor/text_insert/*.rs` | prepare → extract → persist → finalize |
| PDF path | `processor/pdf_processing.rs` | vision/edgeparse → text insert |
| Pipeline resilience wrapper | `pipeline/process_with_resilience` | extraction retries |
| Persist + merge | `persistence/ingestion_persister.rs` | `persist_processing_result` |
| Status dual-write | `processor/status_updates.rs` | L69–97 legacy + `current_stage` |
| Terminal failure metadata | `processor/status_updates.rs` | L10–17 `failure_class` |

---

## 2. Failure surface (merge + compensation)

| Claim | File | Symbol / lines |
| ----- | ---- | -------------- |
| `stats.errors > 0` → `GraphError` | `ingestion_persister.rs` | L343–357 |
| Merge `Err` → compensate | `ingestion_persister.rs` | L359–370 |
| Compensation delegates | `ingestion_persister.rs` | `compensate_merge_failure` L375+ |
| Quarantine log on rollback fail | `compensation.rs` | `"quarantine: failed to roll back orphan node"` |
| Entity batch failure increments errors | `merger/mod.rs` | L403–411 `merge_entities_batch_global` |
| Relationship batch failure | `merger/mod.rs` | L461–465 `merge_relationships_batch_global` |
| Per-entity build failure | `merger/entity.rs` | ~200–208 |

---

## 3. AGE Cypher binding (SPEC-044 — FIXED)

| Claim | File | Evidence |
| ----- | ---- | -------- |
| Third arg is bare `$1` | `cypher_exec.rs` | `cypher_bound_sql` L25–26, L43 |
| Uses `.bind(PgAgtype)` not `raw_sql` | `cypher_exec.rs` | `cypher_execute_bound` L92–94 |
| Module doc cites AGE prepared stmt SSOT | `cypher_exec.rs` | L3–11 |
| Compensation test proves fix | `tests/spec044_compensation_postgres.rs` | full file |
| spec022 enforces parameterized delete | `tests/spec022_cypher_prepared_postgres.rs` | L71 |

**Regression window:** v0.14.0 (#278) shipped inline `::agtype` literal; fixed post SPEC-044.

---

## 4. Failure classification — SSOT in edgequake-tasks

| Claim | File | Evidence |
| ----- | ---- | -------- |
| `IngestionFailureClass` enum incl. `GraphMerge` | `edgequake-tasks/src/ingestion_reliability.rs` | L9–18 |
| `classify_ingestion_failure` | same | L68–100 |
| `is_permanent` on GraphMerge, EmbeddingLimit | same | L47–56 |
| Re-exported by API | `large_document_profile.rs` | L11–13 |
| Worker uses `from_processing_error` | `failure.rs` L123–141; `worker.rs` L430–434 |
| KV metadata `failure_class` | `status_updates.rs` | L12–17 |

**Query pipeline:** No `QueryFailureClass` — gap SRE-Q01 (see 010-sre-engineering-review.md).

---

## 5. Cross-pipeline vector resolve (SRE gap)

| Claim | File | Evidence |
| ----- | ---- | -------- |
| Query: evict+retry on dim mismatch | `handlers/query/workspace_resolve.rs` | L149–168 |
| Ingest: cache hit without validate | `workspace_vector_resolve.rs` | L97–98 |
| Startup ensure_dimension recreates table | `state/postgres.rs` | L258–269 |

**Battle-proof fix:** P0-SRE-1 in 007-implementation-plan.md.

---

## 6. Migration bootstrap

| Claim | File | Evidence |
| ----- | ---- | -------- |
| Orchestrator | `state/migration_bootstrap/mod.rs` | `run_postgres_migrations` L637+ |
| Readiness gate | same | `is_ready_for_traffic` L602–634 |
| M038 degrades when indexes missing | same | `Migration038Report::is_degraded` L272–274 |
| M042 degrades when pgvector < 0.8 | same | `Migration042Report::is_degraded` L290–292 |
| M043–M065 never degrade | same | `is_degraded() -> false` each |
| M047 wsdoc backfill every boot | `reconcile/m047.rs` | L20–29 idempotent apply |
| M041 column reconcile pre-sqlx | `reconcile/m041.rs` | cost_usd DDL guard |

---

## 7. Startup orphan recovery

| Claim | File | Evidence |
| ----- | ---- | -------- |
| Task recovery pagination | `src/main.rs` | `recover_orphaned_tasks` L91+ |
| `processing` → `pending` | same | task status reset |
| Document metadata scan | same | `recover_orphaned_documents` L178+ |
| `uploading` only → `failed` | same | re-upload required |
| Later stages → auto-retry pending | same | not marked failed on restart |
| Runs before workers | same | L690–713 ordering |
| Periodic orphan marks task failed only | same | `periodic_orphan_check` L440–451 — **SRE-I01 gap** |

---

## 8. Recovery API

| Claim | File | Evidence |
| ----- | ---- | -------- |
| Recover stuck docs | `handlers/documents/recovery/stuck.rs` | threshold default 10 min |
| Reprocess failed | `handlers/documents/recovery/reprocess.rs` | graph cleanup first |
| Force reprocess purges tasks | same | `purge_persisted_tasks_for_document` |
| Empty markdown → Full reprocess | `reprocess.rs` | PDF auto-upgrade |

---

## 9. Embedding safety (SPEC-010/011 — FIXED)

| Claim | File | Evidence |
| ----- | ---- | -------- |
| Token clamp 16384 | `safety_limits.rs` | `DEFAULT_MAX_TOKENS` |
| Dual-dimension split | `pipeline/helpers/embeddings.rs` | L145–157 comment |
| Safe batch size 256 | `SafetyLimitedEmbeddingProviderWrapper` | `DEFAULT_SAFE_EMBED_BATCH_SIZE` |
| Truncated JSON recovery | `pipeline/prompts/parser/json_parser.rs` | bracket/brace repair |

---

## 10. Dual-store read model

| Claim | File | Evidence |
| ----- | ---- | -------- |
| Cost from metadata JSONB | `document_read_model.rs` | L132–139 comment |
| Status normalize | same | `normalize_relational_status` |
| Entity count reconcile | same | `reconcile_entity_counts_with_graph` |
| wsdoc index first | `services/document_metadata_scan.rs` | scoped scan |

---

## 11. Large PDF (SPEC-038 — PARTIAL)

| Claim | File | Evidence |
| ----- | ---- | -------- |
| `LargeDocumentProfile` SSOT | `services/large_document_profile.rs` | full module |
| Worker timeout floor 7200s | same | `TASK_TIMEOUT_FLOOR_SECS` L24 |
| `TimeoutPhaseConvert` class | same | L54, L90–91 |
| Per-page timeout formula exists | same | `worker_timeout_secs()` |
| **Gap:** formula not wired to worker pool | `processor/task_impl.rs` | uses global timeout |

---

## 12. Health / readiness signals

| Endpoint | Field | Meaning |
| -------- | ----- | ------- |
| `GET /health` | `migration_bootstrap` | Per-migration report |
| same | `schema.source_ids_indexes.ready` | M038 state |
| same | `ready_for_traffic` | Composite readiness |
| `GET /ready` | 503 (no JSON body yet) | Which migration blocks — **SRE-M03 gap** |
| Failed doc metadata | `failure_class` | Operator triage key |
| same | `recommended_action` | Next step verb |

---

## 13. Log correlation (production triage)

| Log pattern | Interpretation |
| ----------- | -------------- |
| `merge_entities_batch_global` WARN | Entity merge batch failed |
| `merge_relationships_batch_global` WARN | Relationship merge failed |
| `knowledge-graph merge error(s)` | `stats.errors > 0` in persister |
| `quarantine: failed to roll back orphan node` | Compensation failed (SPEC-044 class) |
| `migration_038_degraded` | Missing source_ids indexes |
| `migration_042_degraded` | pgvector too old |
| `Safety limit: max_tokens clamped` | Token budget (should be 16384 now) |
| `Too many inputs` / `Too many tokens` | Embedding limit |
| `error sending request for url (localhost:11434` | Ollama down |
| `Vision extraction timed out` | Large PDF convert timeout |

**Correlate:** same `document_id` across task_process → pipeline_merger → compensation logs.
