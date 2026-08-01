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
