# 01 — Finding Register (SPEC-098)

| ID | Severity | Finding | Law |
|----|----------|---------|-----|
| **F-098-01** | P0 | Saturated KEEP skips `sink_rows` while vectors still mirror → `0/N` FK miss | LAW-098-1, LAW-098-2 |
| **F-098-02** | P1 | Relationship vector ids use raw `relation_type`; sink stores ASCII-uppercase → RelVectors miss | LAW-098-3 |
| **F-098-03** | P1 | Typed path only fails when `resolved == 0`; partial misses accepted | LAW-098-4 |
| **F-098-04** | P2 | Invalid/missing `workspace_id` UUID silently skipped → opaque `0/N` | LAW-098-4 |
| **F-098-05** | P2 | Historical AGE-only entities lack relational spine; reprocess fails until backfill | LAW-098-1, LAW-098-5 |
| **F-098-06** | P0 | Native AGE edge upsert: `ON CONFLICT DO UPDATE cannot affect row a second time` (SQLSTATE 21000) | LAW-098-7, LAW-098-8 |
| **F-098-07** | P0 | `eq_id_schema_ready` early-exit skips legacy UNIQUE drop; `edge_eq_ok` accepts 2-col index | LAW-098-7 |
| **F-098-08** | P1 | Native upsert DO UPDATE dropped `eq_merge_graph_properties` (SPEC-058 regression) | LAW-098-8 |
| **F-098-09** | P1 | Cypher MERGE keys edges on `(source_id, target_id)` only — D-30 multigraph split-brain (closed: batch + `pg_upsert_edge` + `e2e_spec098_cypher_edge_multigraph`) | LAW-098-7 |
| **F-098-10** | P0 | `upsert_relationships_batch` lacks within-batch dedupe (entities sink has it) | LAW-098-8 |
| **F-098-11** | P1 | Migration 139 entities-only; no AGE→`relationships` historical reconcile | LAW-098-1 |
| **F-098-12** | P0 | `merge_document_summaries` ignores KV `deleting` as inflight → SQL `completed`/`indexed` wins mid-delete | LAW-098-9 |
| **F-098-13** | P0 | `documents_valid_status` CHECK omits `deleting`/`delete_failed` → SQL lifecycle mirror impossible | LAW-098-9 |
| **F-098-14** | P0 | Batch delete admit skips per-doc KV/SQL `deleting` | LAW-098-9 |
| **F-098-15** | P0 | FE dual SSOT: session “Document removed” via shared `batch_track_id` poll; no delete pin; table dimming not session-driven | LAW-098-10 |
| **F-098-16** | P0 | Shell `normalize_documents_column_status` maps `deleting`→`cancelled`, `delete_failed`→`failed` | LAW-098-11 |
| **F-098-17** | P0 | Batch deletion result lacks per-id reasons; FE defaults to “Deletion failed” | LAW-098-11 |
| **F-098-18** | P1 | Post-enqueue SQL admit hard-fails 202 when CHECK pre-141; feedback header says Deleting for failed sessions | LAW-098-9/11 |
| **F-098-19** | P1 | Retry Failed buckets `delete_failed` with pipeline failures; no delete_failed badge | LAW-098-11 |
| **F-098-20** | P0 | Shared-entity cascade prune via `upsert_*_batch` + `eq_merge_graph_properties` re-unions pruned `source_ids` → post-proof fails | LAW-098-12 |
| **F-098-21** | P0 | Edge cascade treats topology `source_id` as provenance; `(src,tgt)` collapse misses multigraph; singular `source_chunk_id` uncleared → `0 nodes / N edges` post-proof | LAW-098-13 |
