# 23 — Post-Drop KV Hot-Path Closure

> **Status:** **IMPLEMENTED** (2026-07-31) — waves KVH0–KVH2. Code is law.
> **Scope:** After migration **125**, eliminate pool-burning SQL against missing `eq_*_kv` on serving/ingest hot paths; make `/health` and advisor tell the truth; stamp typed admission columns.
> **Inherits:** [15](15-migration-console-cli.md) · [17](17-boot-migration-gating.md) · [18](18-full-completeness-assessment.md) · [19](19-improvement-plan.md) · [22](22-ingestion-migration-system-assessment.md).
> **Does not reopen:** Full `KVStorage` trait deletion (GAP-091-01 remains Partial / IW3).

---

## 1. Verdict

Tolerance of `42P01` without a relation-absent cache is a **pool tax**. Every health ping and typed-miss list path still acquired a connection and ran a failing `SELECT`. This program makes post-drop raw KV SQL **O(0)** via a process-wide `KvRelationState`, fixes health SSOT, stamps `documents.track_id`, routes chunk counts to `chunks`, and aligns advisor residue with migration 125’s verified purge.

---

## 2. Findings (`F-KVH-01..08`)

| ID | Finding |
| --- | --- |
| F-KVH-01 | `KVStorage::ping` always `SELECT 1 FROM eq_*_kv` → 42P01 → Ok (health tax) |
| F-KVH-02 | Typed-miss list/hydration still falls through to KV SQL |
| F-KVH-03 | `count_embedded_chunks_for_docs` always LIKE-scans KV |
| F-KVH-04 | `/health` hardcodes `chunk_text_ssot: "kv"` |
| F-KVH-05 | Admission stores `track_id` in metadata JSON but not `documents.track_id` |
| F-KVH-06 | Advisor residue not purge-aware vs full 125 verified purge |
| F-KVH-07 | `MERGE_STRATEGY` label still KV-centric |
| F-KVH-08 | Facade allowlist residual (GAP-091-01) — out of wave for trait deletion |

---

## 3. Laws (`LAW-KVH1..KVH5`)

| Law | Statement |
| --- | --- |
| **LAW-KVH1** | Short-circuit before SQL when the KV relation is known Absent |
| **LAW-KVH2** | One posture SSOT: `KvRelationState` seeded from `relation_exists` / first 42P01 / boot census |
| **LAW-KVH3** | Health tells truth: `chunk_text_ssot` from authority + drop posture |
| **LAW-KVH4** | Admission stamps typed columns (`documents.track_id`) |
| **LAW-KVH5** | Advisor residue matches full migration 125 (purge + guard) |

---

## 4. Waves

| Wave | Deliverable |
| --- | --- |
| **KVH0** | `kv_relation_state` + short-circuit all `kv.rs` SQL; `kv_raw_sql_attempts` counter |
| **KVH1** | Health SSOT; MERGE_STRATEGY label; track_id stamp; relational chunk counts |
| **KVH2** | Purge-aware advisor residue; contracts/e2e; `make spec091-gates` / CI |

---

## 5. Acceptance (`KVH-AC-01..10`)

| ID | Gate | Status |
| --- | --- | --- |
| KVH-AC-01 | Absent → ping issues no KV SQL | Met |
| KVH-AC-02 | First 42P01 marks Absent for subsequent calls | Met |
| KVH-AC-03 | Boot seeds Absent when census shows drop | Met |
| KVH-AC-04 | `chunk_text_ssot` ≠ `"kv"` when authority relational | Met |
| KVH-AC-05 | `documents.track_id` stamped at admission | Met |
| KVH-AC-06 | Embedded chunk count uses `chunks` when relational/Absent | Met |
| KVH-AC-07 | MERGE_STRATEGY label relational-primary | Met |
| KVH-AC-08 | Advisor purge-aware residue / ReadyToDrop | Met |
| KVH-AC-09 | Post-drop guidance never asks for KV backfill (relational flags) | Met |
| KVH-AC-10 | Hot-path contracts in `make spec091-gates` | Met |

---

## 6. Tests

- `contract_spec091_kv_ping_short_circuits_when_dropped`
- `e2e_spec091_health_no_kv_sql_post_drop`
- `e2e_spec091_hot_path_no_missing_kv_sql`
- `contract_spec091_health_chunk_text_ssot_relational`
- `contract_spec091_admission_stamps_track_id`
- `contract_spec091_advisor_purge_aware_residue`

---

## 7. Residual

GAP-091-01 (full facade deletion) remains Partial. This wave only guarantees **zero SQL** to missing `eq_*_kv` on the hot path.
