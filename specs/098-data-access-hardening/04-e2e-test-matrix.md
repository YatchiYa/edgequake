# 04 — E2E Test Matrix (SPEC-098)

| Gate | Command | Asserts |
|------|---------|---------|
| Unit saturated spine | `cargo test -p edgequake-pipeline --lib spec098` | spine ensure stat; no graph mutation on KEEP |
| Contract mirror report | `cargo test -p edgequake-storage --features postgres --test contract_spec098_fleet_mirror_report` | bare resolve; uppercase rel; miss sample; invalid workspace |
| E2E saturated ensure | `cargo test -p edgequake-storage --features postgres --test e2e_spec098_saturated_spine_ensure` | AGE+saturated+no entities → persist + fleet row |
| E2E rel type case | `cargo test -p edgequake-storage --features postgres --test e2e_spec098_relation_type_case` | mixed-case type → relationship_embeddings |
| E2E edge cardinality | `cargo test -p edgequake-storage --features postgres --test e2e_spec098_edge_upsert_cardinality` | native ON: dup + multigraph + casefold; no 21000 |
| E2E Cypher multigraph | `cargo test -p edgequake-storage --features postgres --test e2e_spec098_cypher_edge_multigraph` | native OFF: Cypher MERGE rel_type; dup + multigraph |
| E2E legacy UNIQUE | `cargo test -p edgequake-storage --features postgres --test e2e_spec098_legacy_edge_unique_reconcile` | legacy dropped → multigraph OK |
| E2E rel sink dedupe | `cargo test -p edgequake-api --features postgres --test e2e_spec098_rel_sink_batch_dedupe` | duplicate sink rows OK |
| E2E edge upsert perf | `cargo test -p edgequake-storage --features postgres --test e2e_spec098_edge_upsert_perf` | p50/p95 logged; CI budget |
| Contract merge helper | `cargo test -p edgequake-storage --test contract_spec047_p7ef_graph_upsert` | `eq_merge_graph_properties` in node+edge SQL |
| Checksums | `./scripts/check_migration_checksums.sh` | 139 + 140 + 141 in lockfile |
| E2E delete list merge | `cargo test -p edgequake-api --features postgres --test e2e_spec098_delete_list_merge_deleting` | KV deleting beats SQL completed |
| E2E batch admit | `cargo test -p edgequake-api --features postgres --test e2e_spec098_batch_delete_admit_deleting` | selected ids KV+SQL deleting |
| E2E SQL CHECK | `cargo test -p edgequake-api --features postgres --test e2e_spec098_delete_sql_status_constraint` | deleting/delete_failed accepted |
| FE delete units | `cd edgequake_webui && pnpm test -- deletion-session` | pin + batch poll honesty |
| Playwright bulk delete | `pnpm exec playwright test bulk-delete-progress` | Deleting badge until gone |
| E2E delete_failed persists | `cargo test -p edgequake-api --features postgres --test e2e_spec098_delete_failed_status_persists` | SQL+KV delete_failed not collapsed |
| E2E batch failed reasons | `cargo test -p edgequake-api --features postgres --test e2e_spec098_batch_failed_reasons` | failed[{id,reason}] + reset |
| E2E cascade prune vs eq_merge | `cargo test -p edgequake-api --features postgres --test e2e_spec098_cascade_shared_prune_survives_eq_merge` | shared source_ids prune sticks; post-proof Ok |
| Playwright delete failure | `pnpm exec playwright test spec098-bulk-delete-honesty` | Seeds ≥2 docs; Deleting feedback; Delete failed header/badge |
| Contract GH-350 agtype | `cargo test -p edgequake-storage --test contract_spec350_agtype_batch_get` | batch read SQL uses `::ag_catalog.agtype` only |
| E2E GH-350 agtype merge get | `cargo test -p edgequake-storage --features postgres --test e2e_spec350_agtype_batch_get_nodes` | `get_nodes_batch` after native upsert; no agtype missing |
| Playwright GH-350 bulk upload | `pnpm exec playwright test spec350-bulk-upload-webui` | 2 files via dropzone appear in table |
| PG matrix | nightly / `spec091-data-layer` | contract smoke on pg16+pg17+pg18 when IMAGE available |

## Manual repro (operator)

### Fleet spine

1. Upload/reprocess a document whose entities already saturate SOURCE_IDS KEEP and lack relational rows.  
2. Expect **Completed**, not fleet FK fail.  
3. `SELECT count(*) FROM entity_embeddings e JOIN entities n ON n.id = e.entity_id WHERE e.workspace_id = $ws;`

### Edge cardinality

1. Reprocess a large multi-chunk PDF that extracts many relationships (e.g. hyper-connection paper).  
2. Expect **Completed**, not `ON CONFLICT DO UPDATE cannot affect row a second time`.  
3. `SELECT indexname FROM pg_indexes WHERE schemaname = '<graph>' AND tablename = 'EDGE';` — must include `idx_edge_eq_source_target_rel`, must **not** include `idx_edge_source_target_unique` / `idx_edge_eq_source_target`.

### Delete dual-SSOT

1. Select K completed documents → Delete → Confirm.  
2. While cascade runs: table badges **Deleting** (not Completed/Ready); feedback zone matches.  
3. `SELECT id, status FROM documents WHERE id = ANY($ids);` → `deleting`.  
4. After success: rows absent from list + SQL + KV metadata; feedback completes/dismisses.
