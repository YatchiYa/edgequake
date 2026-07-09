# SPEC-045 — Post-Migration Ingestion Errors

**Spec:** `045-fix-ingestion-errors`  
**Date:** 2026-07-09  
**Status:** `IMPLEMENTED` — root causes mapped; operator runbook + P0/P1 ingestion shipped; **SRE review adds P0-SRE cross-pipeline plan**  
**Method:** Code is law — live source cross-ref + production incident lineage (SPEC-010/011/038/041/044)  
**Trigger:** EdgeQuake migrated to production; document ingestion intermittently fails after upgrade

---

## TL;DR

> Post-migration ingestion failures are **not one bug** — they are a **stack of interacting failure classes** across migration bootstrap, graph merge, provider limits, and dual-store drift. The highest-signal production incident (SPEC-044) was **graph merge failure + broken saga compensation** when AGE rejected inline `agtype` literals. That Cypher binding is **fixed in current `main`** (`cypher_exec.rs` bare `$1`). Remaining production risk clusters in: **(1)** readiness gates blocking uploads (M038/M042), **(2)** merge errors from slow/missing graph indexes on upgraded volumes, **(3)** provider misconfiguration after deploy, **(4)** large-PDF timeout class (SPEC-038), **(5)** missing `failure_class` for graph merge → operators see `unknown` + generic `retry`.

**Quick operator path:** [005-quick-fix-runbook.md](./005-quick-fix-runbook.md)  
**Bulletproof design:** [006-bulletproof-migration-design.md](./006-bulletproof-migration-design.md)

---

## Failure Taxonomy (ranked by post-migration frequency)

| Rank | Class | Symptom | Root layer | Blocks upload? |
| ---- | ----- | ------- | ---------- | -------------- |
| P0 | `readiness_degraded` | `/ready` 503, upload rejected | M038 indexes / M042 pgvector | **Yes** |
| P0 | `graph_merge` | `N knowledge-graph merge error(s) during persist` | AGE graph merge + compensation | No (per-doc) |
| P1 | `provider_unavailable` | `Network error … localhost:11434` | LLM provider config | No |
| P1 | `embedding_limit` | `Too many tokens/inputs` (400) | Embedding batch limits | No |
| P1 | `wsdoc_index_gap` | Docs in KV but list returns 0 | M047 backfill / write-path | No |
| P2 | `timeout_phase_convert` | Vision timeout on large PDF | Worker timeout (7200s cap) | No |
| P2 | `orphan_processing` | Stuck `processing` after restart | Task/doc recovery | No |
| P2 | `kv_pg_drift` | Count mismatch, reprocess misses doc | Dual-write history | No |
| P3 | `partial_failure` | `completed` with 0 entities | Silent extraction empty | No |

---

## Documents

| File | Lens | Key question |
| ---- | ---- | ------------ |
| [001-five-whys.md](./001-five-whys.md) | 5 WHY | Why does ingestion fail after migration? |
| [002-first-principles.md](./002-first-principles.md) | First principles | What must be true for ingest to succeed? |
| [003-code-is-law.md](./003-code-is-law.md) | Code is law | Exact file/line evidence |
| [004-edge-cases-matrix.md](./004-edge-cases-matrix.md) | Edge cases | Exhaustive register + status |
| [005-quick-fix-runbook.md](./005-quick-fix-runbook.md) | Operator | Immediate triage + recovery |
| [006-bulletproof-migration-design.md](./006-bulletproof-migration-design.md) | Reliability | Auto-migration + self-healing patterns |
| [007-implementation-plan.md](./007-implementation-plan.md) | Engineering | Phased P0–P3 code fixes |
| [008-cross-reference-matrix.md](./008-cross-reference-matrix.md) | Cross-ref | Evidence map to prior specs |
| [009-battle-test-results.md](./009-battle-test-results.md) | Battle test | Gate results + EC coverage |
| [010-sre-engineering-review.md](./010-sre-engineering-review.md) | **SRE lens** | Code-is-law assessment: ingest + migration + query |
| [011-battle-proof-first-principles.md](./011-battle-proof-first-principles.md) | First principles | Invariant matrix + what we missed |

---

## E2E proof

```bash
# Operator health gates (Postgres + AGE + pgvector + wsdoc)
export DATABASE_URL=postgres://edgequake:edgequake@localhost/edgequake
psql "$DATABASE_URL" -f specs/045-fix-ingestion-errors/e2e/sql/post_migration_ingest_health.sql

# Full ingestion health proof (health + Cypher + stuck recovery contract)
./specs/045-fix-ingestion-errors/e2e/run_ingestion_health_proof.sh

# Full battle suite (recommended before production deploy)
make spec045-battle-test-all
```

---

## Requirements (REQ-045-xx)

| ID | Requirement |
| -- | ----------- |
| REQ-045-01 | Every ingestion failure surfaces `failure_class` + `recommended_action` in KV metadata |
| REQ-045-02 | Graph merge failures map to `failure_class=graph_merge`, action `reprocess_full` | ✅ P0 shipped |
| REQ-045-03 | Bootstrap reconcile runs M047 wsdoc backfill on every startup (idempotent) |
| REQ-045-04 | `/ready` documents which migration blocks traffic (M038 or M042 only) |
| REQ-045-05 | Startup orphan recovery runs before workers (tasks + documents) |
| REQ-045-06 | Reprocess cleans graph artifacts before requeue (OODA-08) |
| REQ-045-07 | Post-migration operator runbook executable without reading source |
| REQ-045-08 | Embedding 400 classified permanent — no wasteful 3× retry | ✅ P1 shipped |
| REQ-045-09 | Large PDF admission routes born-digital to EdgeParse (SPEC-038) |
| REQ-045-10 | CI gate: `spec045` ingestion health proof in release pipeline | ✅ `make spec045-battle-test-all` |

---

## Related specs

| Spec | Relationship |
| ---- | ------------ |
| [SPEC-044](../044-upgrate-issue-study/000-index.md) | Primary production incident — graph merge + Cypher |
| [SPEC-041](../041-fix-migration/000-index.md) | M078 startup blocker on upgrade |
| [SPEC-042](../042-update-age-pgvector/000-index.md) | pgvector + AGE upgrade matrix |
| [SPEC-038](../038-ingestion-large-pdf/007-decision-matrix.md) | Large PDF timeout class |
| [SPEC-010](../010-ingestion-reliability/root_cause_analysis.md) | Token clamp + JSON EOF (fixed v0.11.2) |
| [SPEC-011](../011-pipeline-reliabilty/docs/EDGE_CASES.md) | Embedding count + 429 gaps |
