# 02 — Cross-Ref Matrix (SPEC-098)

| Finding | Code | Test | Law |
|---------|------|------|-----|
| F-098-01 | `merger/entity.rs` saturated → still `sink_rows` | `e2e_spec098_saturated_spine_ensure` · `spec098_saturated_spine_ensure_stat` | LAW-098-1/2 |
| F-098-02 | `normalize_relation_type_str` + `collect_relationship_vector_batch` | `e2e_spec098_relation_type_case` · `contract_spec098_fleet_mirror_report` | LAW-098-3 |
| F-098-03 | `upsert_vectors_chunked` `resolved < eligible` | `contract_spec098_fleet_mirror_report` | LAW-098-4 |
| F-098-04 | `mirror_legacy_batch` invalid workspace list | `contract_spec098_invalid_workspace_loud` | LAW-098-4 |
| F-098-05 | migration `139` + `support/139/apply.sql` | checksum gate · bootstrap include | LAW-098-5/6 |
| F-098-06 | `edges_ops.rs` native upsert + `graph_batch_dedupe` | `e2e_spec098_edge_upsert_cardinality` | LAW-098-7/8 |
| F-098-07 | `graph_lifecycle.rs` reconcile every boot; mig `140` | `e2e_spec098_legacy_edge_unique_reconcile` | LAW-098-7 |
| F-098-08 | `edges_ops` / `nodes_ops` `eq_merge_graph_properties` | `contract_spec058_native_upsert_uses_eq_merge_graph_properties` | LAW-098-8 |
| F-098-09 | Cypher MERGE includes `relation_type` (batch + single) | `e2e_spec098_cypher_edge_multigraph` (native off) | LAW-098-7 |
| F-098-10 | `postgres_entity_sink.rs` rel batch dedupe | `e2e_spec098_rel_sink_batch_dedupe` | LAW-098-8 |
| F-098-11 | `support/140/apply.sql` relationship spine reconcile | checksum · bootstrap m140 | LAW-098-1/5 |
