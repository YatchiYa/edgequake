# SPEC-045 — Edge Cases Matrix

**Cross-ref:** [SPEC-011 EDGE_CASES](../011-pipeline-reliabilty/docs/EDGE_CASES.md) · [SPEC-044 edge cases](../044-upgrate-issue-study/004-edge-cases-and-mitigations.md)

---

## 1. Master matrix

```
  Stage × Migration state × Document type
  ┌────────────────────┬──────────────┬─────────────────┬──────────────────────────┬──────────┐
  │ Stage              │ Migration    │ Doc type        │ Symptom                  │ Status   │
  ├────────────────────┼──────────────┼─────────────────┼──────────────────────────┼──────────┤
  │ Admission          │ M042 degrade │ Any             │ Upload 503 /ready        │ 🟡 Gate  │
  │ Admission          │ M038 defer   │ Any             │ Upload OK, slow ingest   │ 🟡 Warn  │
  │ PDF convert        │ N/A          │ 600+ page PDF   │ Vision timeout 7200s     │ 🔴 OPEN  │
  │ PDF convert        │ N/A          │ Born-digital    │ Unnecessary Vision path  │ 🟡 PART  │
  │ Extraction         │ Provider down│ Any             │ Network error Ollama     │ ✅ Class │
  │ Extraction         │ Token clamp  │ Dense PDF       │ JSON EOF                 │ ✅ Fixed │
  │ Extraction         │ All chunks fail│ Any           │ 0 entities completed     │ 🔴 OPEN  │
  │ Embedding          │ N/A          │ Legal/dense     │ Too many inputs 400      │ ✅ Fixed │
  │ Embedding          │ N/A          │ Bulk ingest     │ 429 rate limit           │ 🔴 OPEN  │
  │ Embedding          │ M080 partial │ Any             │ Dimension insert error   │ 🟡 Gate  │
  │ Graph merge        │ M038 missing │ High entity cnt │ merge batch error        │ 🟡 Degrade│
  │ Graph merge        │ SPEC-044     │ Any             │ compensation Cypher fail │ ✅ Fixed │
  │ Graph merge        │ #217 strict  │ Legacy entities │ entity type rejection    │ 🟡 Data  │
  │ Compensation       │ Orphan node  │ Any             │ quarantine log           │ ✅ Fixed │
  │ Finalize           │ M047 gap     │ Pre-migration   │ Doc missing from list    │ ✅ Boot  │
  │ Finalize           │ KV/PG drift  │ Historical      │ Count mismatch           │ 🟡 Merge │
  │ Restart            │ Mid-process  │ Any             │ Stuck processing         │ ✅ Boot  │
  │ Restart            │ Mid-upload   │ Any             │ uploading → failed       │ ✅ Boot  │
  │ Reprocess          │ No cleanup   │ Failed merge    │ Duplicate entities       │ ✅ API   │
  │ Reprocess          │ In-flight    │ Processing      │ Race duplicate task      │ ✅ Purge │
  │ List API           │ M041 partial │ Any             │ (was) column error       │ ✅ JSONB │
  │ Query              │ Dim mismatch │ Provider switch │ Wrong similarity         │ 🟡 Retry │
  └────────────────────┴──────────────┴─────────────────┴──────────────────────────┴──────────┘
```

Legend: ✅ Fixed/mitigated · 🟡 Partial/degraded · 🔴 Open gap

---

## 2. Post-migration scenarios (detailed)

### EC-045-01 — Fresh upgrade, large AGE graph, M038 deferred

| Field | Value |
| ----- | ----- |
| Trigger | Graph > size threshold; inline index build skipped |
| Symptom | `/ready` 503 OR slow merge timeouts |
| Detection | `/health` → `schema.source_ids_indexes.ready: false` |
| Mitigation | `apply_038.sh --apply --concurrent --yes` |
| Auto? | Bootstrap defers; operator script required |

### EC-045-02 — pgvector catalog stuck at 0.7.x

| Field | Value |
| ----- | ----- |
| Trigger | M042 reconcile failed or wrong postgres image |
| Symptom | `/ready` 503; no uploads accepted |
| Detection | `migration_042_degraded` in logs |
| Mitigation | Rebuild EdgeQuake postgres image; restart API |
| Auto? | M042 reconcile attempts upgrade on boot |

### EC-045-03 — Merge failure on upgraded graph (PRIMARY)

| Field | Value |
| ----- | ----- |
| Trigger | Batch upsert/get_nodes failure under load |
| Symptom | `N knowledge-graph merge error(s) during persist` |
| Detection | `failure_class: unknown` (should be `graph_merge`) |
| Mitigation | Fix indexes (M038/M046); `POST reprocess` mode=Full |
| Auto? | Compensation rolls back vectors; graph cleanup on reprocess |

### EC-045-04 — Compensation Cypher regression (SPEC-044)

| Field | Value |
| ----- | ----- |
| Trigger | v0.14.0 inline agtype literal |
| Symptom | `third argument of cypher function must be a parameter` |
| Detection | `quarantine: failed to roll back orphan node` |
| Mitigation | Upgrade to SPEC-044-fixed build |
| Auto? | **Fixed** — `cypher_exec.rs` bare `$1` |

### EC-045-05 — wsdoc index missing (pre-SPEC-027 docs)

| Field | Value |
| ----- | ----- |
| Trigger | Metadata without `wsdoc:{ws}:{doc}` pointer |
| Symptom | Document exists but list returns empty |
| Detection | KV key `{id}-metadata` exists; no wsdoc key |
| Mitigation | M047 reconcile on restart (automatic) |
| Auto? | ✅ Every bootstrap |

### EC-045-06 — Restart mid-extraction

| Field | Value |
| ----- | ----- |
| Trigger | Deploy rolling restart during processing |
| Symptom | Doc stuck `processing` or auto-pending |
| Detection | Startup logs `recover_orphaned_*` |
| Mitigation | Automatic; or `recover-stuck` API |
| Auto? | ✅ main.rs startup |

### EC-045-07 — Ollama not running post-deploy

| Field | Value |
| ----- | ----- |
| Trigger | ECS/K8s task starts before Ollama sidecar |
| Symptom | Entity extraction network error |
| Detection | `failure_class: provider_unavailable` |
| Mitigation | Start Ollama; reprocess |
| Auto? | Classified; no auto-retry |

### EC-045-08 — halfvec mode without M080

| Field | Value |
| ----- | ----- |
| Trigger | `EDGEQUAKE_VECTOR_STORAGE=half` on old schema |
| Symptom | Vector insert type/dimension error |
| Detection | Storage error in persist; health dimension flags |
| Mitigation | M080 reconcile; verify halfvec columns |
| Auto? | M080 runs when half mode enabled |

### EC-045-09 — Embedding 400 retried 3×

| Field | Value |
| ----- | ----- |
| Trigger | Permanent provider limit exceeded |
| Symptom | 3× identical API errors; wasted quota |
| Detection | Task retry count = 3, same error message |
| Mitigation | Classify permanent; skip retry (REQ-045-08) |
| Auto? | 🔴 Not implemented |

### EC-045-10 — Checkpoint resume with wrong provider

| Field | Value |
| ----- | ----- |
| Trigger | Provider changed between attempts |
| Symptom | Stale extraction reused |
| Detection | `pipeline_checkpoint.rs` hash mismatch |
| Mitigation | Force full reprocess |
| Auto? | ✅ Checkpoint invalidation |

### EC-045-11 — PDF failed, empty markdown

| Field | Value |
| ----- | ----- |
| Trigger | Vision timeout or corrupt PDF |
| Symptom | Failed status; PDF visible, no markdown |
| Detection | `markdown_length: 0` in metadata |
| Mitigation | Reprocess Full; try EdgeParse routing |
| Auto? | Reprocess auto-upgrades to Full |

### EC-045-12 — Informational notice shown as Failed

| Field | Value |
| ----- | ----- |
| Trigger | Non-fatal pipeline notice in error string |
| Symptom | UI shows Failed incorrectly |
| Detection | `is_informational_notice` in status_updates |
| Mitigation | Scrubbed to `warning_message` |
| Auto? | ✅ status_updates.rs |

### EC-045-13 — Legacy entity types (#217)

| Field | Value |
| ----- | ----- |
| Trigger | Pre-strict-mode entities in graph |
| Symptom | New ingestions fail entity validation |
| Detection | Entity type audit scripts |
| Mitigation | `entity_reconcile` plan + execute |
| Auto? | Enforcement on new writes only |

### EC-045-14 — Multi-instance bootstrap race

| Field | Value |
| ----- | ----- |
| Trigger | Multiple API pods start simultaneously |
| Symptom | Duplicate reconcile (harmless) or sqlx lock wait |
| Detection | `edgequake.migration` preflight logs |
| Mitigation | sqlx advisory lock serializes |
| Auto? | ✅ Built-in |

### EC-045-15 — Auth enabled, no bootstrap admin

| Field | Value |
| ----- | ----- |
| Trigger | v0.15 auth migration without bootstrap env |
| Symptom | Cannot upload via UI (401) |
| Detection | Login redirect; no users in PG |
| Mitigation | `EDGEQUAKE_BOOTSTRAP_ADMIN_*` env vars |
| Auto? | Bootstrap admin on first start (#288) |

---

## 3. Priority matrix

```
                High impact (blocks prod)     Low impact (single doc)
High frequency  P0: M038/M042 readiness      P1: embedding 429
                P0: graph merge errors        P2: 0-entity silent success
Low frequency   P1: M080 halfvec mismatch      P3: KV/PG count drift
```

---

## 4. Battle test coverage map

| Edge case | Test / proof |
| --------- | ------------ |
| EC-045-04 Cypher bind | `spec044_compensation_postgres.rs`, `make spec044-battle-test-all` |
| EC-045-05 wsdoc | M047 reconcile + `spec027` contracts |
| EC-045-06 orphan | `main.rs` integration / manual restart test |
| EC-045-01 M038 | `migration_readiness_proof.rs` |
| EC-045-02 M042 | `spec042-battle-test-all` |
| EC-045-03 merge | `e2e_pipeline_tests.rs` (memory); **gap: postgres scale** |
| EC-045-09 permanent 400 | `spec045` tasks + api tests | ✅ |
| EC-045-03 graph_merge class | `spec045` tasks + api tests | ✅ |
| EC-045-02 429 retry | `embeddings.rs` spec045 test | ✅ |
| EC-045-06 auto document repair | `main.rs` env + contract test | ✅ partial |

See [009-battle-test-results.md](./009-battle-test-results.md).
