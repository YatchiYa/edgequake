# SPEC-045 — Cross-Reference Matrix

| REQ-045 | Topic | Primary source | Prior spec | Test / proof |
| ------- | ----- | -------------- | ---------- | ------------ |
| 01 | failure_class metadata | `status_updates.rs` L10–17 | SPEC-038 REQ-038-09 | `spec038_large_pdf.rs` |
| 02 | graph_merge class | `ingestion_reliability.rs` | SPEC-044 | `spec045` tests ✅ |
| 03 | M047 wsdoc backfill | `reconcile/m047.rs` | SPEC-027 | spec027 contracts |
| 04 | /ready gate | `migration_bootstrap/mod.rs` L602 | SPEC-042 | `migration_readiness_proof.rs` |
| 05 | orphan recovery | `main.rs` L690–713 | SPEC-024 G1 | manual restart |
| 06 | reprocess cleanup | `recovery/reprocess.rs` | OODA-08 | e2e reprocess |
| 07 | operator runbook | `005-quick-fix-runbook.md` | AGENTS.md troubleshooting | e2e script |
| 08 | permanent 400 skip | `failure.rs` `from_processing_error` | SPEC-011 EC-009 | `spec045` tasks ✅ |
| 09 | EdgeParse routing | `large_document_profile.rs` | SPEC-038 | `spec038_large_pdf.rs` |
| 10 | CI ingest health | `e2e/run_ingestion_health_proof.sh` | SPEC-044 | `make spec045-battle-test-all` |
| 11 | vector resolve parity | `workspace_vector_resolve.rs` | SRE-Q02 | ❌ P0-SRE-1 |
| 12 | runtime task/doc sync | `main.rs` periodic orphan | SRE-I01 | ❌ P0-SRE-2 |
| 13 | query failure taxonomy | `edgequake-query/error.rs` | SRE-Q01 | ❌ P1-SRE-2 |
| 14 | failure_class metrics | `metrics.rs` | SRE-I06 | ❌ P1-SRE-1 |

---

## Failure class → spec lineage

| failure_class | Root spec | Fixed? |
| ------------- | --------- | ------ |
| `timeout_phase_convert` | SPEC-038 | Partial |
| `timeout_phase_extract` | SPEC-038 | Partial |
| `embedding_limit` | SPEC-010, SPEC-011 | ✅ v0.11.2+ |
| `provider_unavailable` | AGENTS.md Ollama | Ops |
| `circuit_breaker` | SPEC-038 | ✅ |
| `graph_merge` | SPEC-044, SPEC-045 | ✅ classify |
| `readiness_degraded` | SPEC-042 M038/M042 | ✅ gate |
| `unknown` | — | Fallback |

---

## Migration → ingestion impact

| Migration | Spec | Ingestion impact | Reconcile file |
| --------- | ---- | ---------------- | -------------- |
| M038 | SPEC-012, bootstrap-first-principles | Merge perf; /ready | `m038.rs` |
| M040 | SPEC-021 | Entity CQRS lag | `m040.rs` |
| M041 | SPEC-027 | List cost columns | `m041.rs` |
| M042 | SPEC-042 | Vector insert; /ready | `m042.rs` |
| M043 | SPEC-042 | AGE version | `m043.rs` |
| M046 | SPEC-027 | Scoped graph perf | `m046.rs` |
| M047 | SPEC-027 | Document list | `m047.rs` |
| M071 | SPEC-041 | HNSW dim | `m071.rs` |
| M078/079 | SPEC-041 #273 | AGE child indexes | `m078.rs` |
| M080 | SPEC-042 | halfvec schema | `m080.rs` |

---

## Production incident timeline

| Date | Version | Incident | Spec |
| ---- | ------- | -------- | ---- |
| 2026-05 | ≤v0.11.1 | JSON EOF + embedding tokens | SPEC-010 |
| 2026-06 | v0.13.2 | M078 `->>>` startup blocker | SPEC-041 |
| 2026-07-01 | — | Large PDF vision timeout | SPEC-038 |
| 2026-07-03 | v0.13.3 | M078 checksum repair | SPEC-041 |
| 2026-07-06 | v0.14.1 | Graph merge + Cypher compensation | SPEC-044 |
| 2026-07-09 | production | Post-migration ingest errors (umbrella) | **SPEC-045** |

---

## Code modules map

```
edgequake-api/
├── handlers/documents/upload/     → admission
├── handlers/documents/recovery/   → stuck, reprocess
├── processor/                     → task worker
├── processor/text_insert/         → text pipeline
├── processor/status_updates.rs    → failure metadata
├── services/large_document_profile.rs → failure_class SSOT
├── document_read_model.rs         → dual-store list
├── state/migration_bootstrap/     → auto-migration
└── src/main.rs                    → orphan recovery

edgequake-pipeline/
├── persistence/ingestion_persister.rs → merge + compensate trigger
├── merger/mod.rs                  → batch merge errors
└── pipeline/helpers/embeddings.rs → token/count split

edgequake-storage/
├── compensation.rs                → saga rollback
├── adapters/postgres/graph/helpers/cypher_exec.rs → AGE bind
└── entity_reconcile.rs            → legacy graph cleanup
```

---

## Document index (this spec)

| File | Purpose |
| ---- | ------- |
| [000-index.md](./000-index.md) | Entry point + TL;DR |
| [001-five-whys.md](./001-five-whys.md) | Root cause chain |
| [002-first-principles.md](./002-first-principles.md) | Invariants + saga model |
| [003-code-is-law.md](./003-code-is-law.md) | File:line evidence |
| [004-edge-cases-matrix.md](./004-edge-cases-matrix.md) | 15 edge cases |
| [005-quick-fix-runbook.md](./005-quick-fix-runbook.md) | Operator procedures |
| [006-bulletproof-migration-design.md](./006-bulletproof-migration-design.md) | Self-healing architecture |
| [007-implementation-plan.md](./007-implementation-plan.md) | P0–P3 engineering |
| [008-cross-reference-matrix.md](./008-cross-reference-matrix.md) | This file |
| [009-battle-test-results.md](./009-battle-test-results.md) | Battle test gates |
| [010-sre-engineering-review.md](./010-sre-engineering-review.md) | SRE assessment |
| [011-battle-proof-first-principles.md](./011-battle-proof-first-principles.md) | Invariant matrix |

---

## SRE gap IDs (quick lookup)

| ID | Area | Priority | Plan item |
| -- | ---- | -------- | --------- |
| SRE-Q02 | Cross-pipeline vector resolve | P0-SRE | P0-SRE-1 |
| SRE-I01 | Periodic orphan doc sync | P0-SRE | P0-SRE-2 |
| SRE-I02 | Pending requeue pagination | P0-SRE | P0-SRE-3 |
| SRE-M05 | M080 cache invalidation | P0-SRE | P0-SRE-4 |
| SRE-I06 | failure_class metrics | P1-SRE | P1-SRE-1 |
| SRE-Q01 | Query failure taxonomy | P1-SRE | P1-SRE-2 |
| SRE-M03 | `/ready` JSON blockers | P1-SRE | P1-SRE-3 |
