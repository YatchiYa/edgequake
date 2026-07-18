# SPEC-058 — Test matrix

| Case                                             | Test / contract                                                                  |
| --------------------------------------------------| ----------------------------------------------------------------------------------|
| Shared entity excluded from compensate artifacts | `merger::tests::spec058_shared_entity_vector_excluded_from_compensate_artifacts` |
| Compensate preserves shared entity vector        | `compensation::tests::spec058_compensate_preserves_shared_entity_vector`         |
| Native upsert uses `eq_merge_graph_properties`   | `contract_spec047_p7ef_graph_upsert::contract_spec058_*`                         |
| Sequential upserts union `source_ids`            | `spec058_source_ids_merge_postgres` (postgres feature)                           |
| FTS hits `content_ref`-only chunks               | `spec058_fts_content_ref_postgres` (postgres feature)                            |
| FTS NULLIF + content_ref join                    | `contract_vector_postgres_fts`                                                   |
| Retract removes sole-source nodes / keeps shared | `retract_document_indexes::tests::*`                                             |
| Workspace filter on BFS edges                    | `graph_hops::tests::spec058_workspace_filter_excludes_foreign_edges`             |
| Dimension fail-closed contract                   | `spec058_dimension_failclosed`                                                   |
| Local/Global vector_type SQL                     | `contract_spec058_vector_type_sql`                                               |
| Mix arm concurrency gate                         | `arm_concurrency` + contract                                                     |
| `ef_construction` default 64                     | `hnsw_ef_construction_tests::default_is_64_when_unset` (postgres feature)        |

## Commands

```bash
cd edgequake
cargo test -p edgequake-storage compensation --lib
cargo test -p edgequake-pipeline spec058_shared_entity --lib
cargo test -p edgequake-api retract_document_indexes --lib
cargo test -p edgequake-query --test contract_spec058_vector_type_sql
cargo test -p edgequake-storage --features postgres --test spec058_fts_content_ref_postgres
cargo test -p edgequake-storage --features postgres --test spec058_source_ids_merge_postgres
```
