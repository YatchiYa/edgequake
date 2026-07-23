# SPEC-083 — E2E & Edge-Case Matrix

> Maps clusters → mandatory tests. Implementation PRs must add these (names may be adapted but assertions must hold).
>
> **Evidence legend (full-pack audit 2026-07-23):** **Present** = exact `fn` under `edgequake/crates`; **Adapted** = near-name / include_str hub in `spec083_matrix_contracts.rs`; **Backlog** = CONFIRMED/PARTIAL defect — not required green yet. Many `e2e_*` names are unit/include_str (honest: not live Postgres unless noted).

**Battery:**
```bash
cargo test -p edgequake-api --test spec083_matrix_contracts --features postgres
cargo test -p edgequake-api --test e2e_postgres_rls --features postgres -- --ignored --test-threads=1
```

---

## Cluster 00 — Schema readiness

| Test ID | Type | Status | Assertion |
|---------|------|--------|-----------|
| `e2e_schema_ready_refuses_traffic` | e2e | Adapted | Missing eq_* → 503 on query/ingest (or documented fallback header) |
| `e2e_degrees_match_property_fallback` | e2e | Present | Fallback degrees == property-derived (COALESCE contract) |
| `contract_eq_columns_present_after_reconcile` | contract | Present | After 092, columns+indexes exist |
| `contract_native_upsert_eq_arbiter` | contract | Present | Source contains ON CONFLICT (eq_*) targets |
| EC-P0-1…5 | edge | Backlog | See [INCIDENT-PROD-DIAGNOSIS.md](INCIDENT-PROD-DIAGNOSIS.md) |

---

## Cluster 01 — Tenant isolation

| Test ID | Status | Assertion |
|---------|--------|-----------|
| `e2e_ws_tenant_a_never_sees_tenant_b` | Present | No cross-tenant progress payloads |
| `e2e_cancel_foreign_track_id_404` | Present | Cancel other tenant → 404 |
| `e2e_pdf_progress_foreign_404` | Present | PDF progress scoped |
| `e2e_rls_guc_visible_on_following_insert` | Present (runtime PG) | Same tx sees tenant after set_config |
| `e2e_owner_forced_rls` | Present (runtime PG) | Table owner cannot bypass FORCE |
| `e2e_null_tenant_row_invisible` | Present (runtime PG) | NULL tenant not world-readable |
| `e2e_document_originals_cross_workspace_denied` | Present (runtime PG) | Binary isolation |
| `contract_matches_track_id_deletion_variants` | Present | Deletion* matched |
| `e2e_kv_cross_tenant_denied` | Present | X-37 KV |

---

## Cluster 02 — Auth / transport

| Test ID | Status | Assertion |
|---------|--------|-----------|
| `e2e_logout_rejects_access_jti` | Present | Access rejected after logout |
| `contract_jwt_requires_iss_aud` | Present | Validation fails without |
| `contract_unknown_role_rejected` | Present | Not User |
| `contract_startup_rejects_default_secret` | Present | Fatal without DEV_MODE |
| `contract_cors_default_fail_closed_prod` | Present | No Any in prod (S-10 PARTIAL residual) |
| `e2e_rate_limit_ignores_spoofed_header` | Present | Claims key wins |
| `contract_filename_strips_path` | Present | `../` removed |
| `e2e_exe_as_pdf_rejected` | Present | Magic mismatch |
| `contract_no_eval_in_bench047` | Present | No `eval(` |
| `contract_env_example_vision_not_openai_by_default` | Present | D-50 |

---

## Cluster 03 — Graph identity

| Test ID | Status | Assertion |
|---------|--------|-----------|
| `unit_normalize_THE_COMPANY` | Present | → `COMPANY` (same as The Company) |
| `unit_normalize_curly_apostrophe` | Present | U+2019 stripped |
| `e2e_merge_duplicate_nodes_migration` | Backlog | Dupes collapse (C-14 migrate residual) |
| `e2e_multigraph_two_rel_types_persist` | Present | KNOWS + WORKS_WITH |
| `unit_weight_associative` | Present | Policy documented & stable |
| `e2e_entity_type_conflict_logged_and_resolved` | Present | Majority/confidence type vote (D-32) |
| `e2e_lineage_includes_docs_beyond_source_cap` | Present | LAW-7 |
| `unit_needs_llm_always_summarizes` | Present | No [1200,4000] gap |
| `contract_other_in_default_entity_types` | Present | X-15 regression |

---

## Cluster 04 — Pipeline

| Test ID | Status | Assertion |
|---------|--------|-----------|
| `e2e_page_aware_offsets_rebase` | Present | slice(doc)=chunk.text |
| `e2e_huge_table_splits` | Backlog | No single oversize atomic (C-16) |
| `contract_gleaning_uses_completion_options` | Present | include_str / AST |
| `e2e_chunk_max_retries_zero_still_attempts_once_or_rejects` | Present | C-18 |
| `contract_batch_fetch_uses_get_by_ids` | Present | C-21 |
| `e2e_kv_upsert_all_or_nothing` | Present | C-22 |
| `e2e_dedup_matches_completed_and_indexed` | Present | C-23 |
| `contract_no_substring_retry_matching` | Present | X-07 |
| `unit_retry_has_jitter` | Present | X-06 |
| `e2e_ollama_cosine_after_l2` | Present | X-10 (L2 contract; not live Ollama) |
| `contract_embed_batch_ssot` | Backlog | X-08 |
| `e2e_checkpoint_rejects_suffix_change` | Present | X-28 |
| `e2e_cancelled_cannot_mark_success` | Present | X-29 |
| `unit_failure_class_typed` | Present | X-30 PARTIAL residual |
| `e2e_shutdown_drains_or_cancels_within_budget` | Backlog | X-31 |
| `contract_cache_set_or_module_removed` | Backlog | D-52 |

---

## Cluster 05 — Query

| Test ID | Status | Assertion |
|---------|--------|-----------|
| `e2e_query_vec_matches_question_only_embedding` | Present | D-38 |
| `e2e_min_score_enforced_on_rrf` | Present | D-39 |
| `contract_fusion_mode_names` | Present | D-36 (D-35 backlog) |
| `unit_score_scale_no_cross_compare` | Backlog | D-37 |
| `contract_stream_stats_superset` | Backlog | D-40 |
| `unit_cosine_dim_mismatch_is_err` | Present | C-28 |
| `e2e_fts_language_config` | Backlog | X-05 |
| `contract_citation_stable_ids` | Backlog | X-20 |

---

## Cluster 06 — Ops / CI / SDK

| Test ID | Status | Assertion |
|---------|--------|-----------|
| `contract_makefile_has_postgres_start_alias` | Present | D-47 |
| `contract_no_sed_i_empty_string` | Present | D-49 |
| `contract_ci_no_continue_on_error_critical` | Backlog | X-32 PARTIAL |
| `contract_frontend_test_must_run` | Backlog | X-32 PARTIAL |
| `contract_config_precedence` | Present | X-36 |
| `contract_upload_limit_ssot_50mib` | Present | D-44 |
| `contract_single_edgequake_llm_version` | Present | X-09 |
| `unit_progress_weighted` | Present | D-41 |
| `e2e_progress_survives_restart` | Present | D-42 |
| `e2e_audit_insert_next_month_partition` | Present | D-45 |
| `contract_sdk_major_matches_server_policy` | Present | X-33 |
| `contract_tasks_pk_documented` | Present | X-01 |
| `contract_checksum_drift_fails_loud` | Present | X-02 |
| `contract_pdf_concurrency_schedule` | Present | X-12 |
| `contract_x_23` | Present | X-23 |
| `contract_x_26` | Present | X-26 |
| `contract_x_27` | Present | X-27 |
| `contract_otel_respects_rust_log` | Present | D-46 |
| `contract_no_nested_github_workflows_or_root_dispatch` | Present | D-48 |
| `e2e_reindex_embedding_model_change` | Present | X-11 (501) |
| `contract_single_audit_definition` | Present | D-45 |

---

## Cluster 07 — Accuracy

| Test ID | Status | Assertion |
|---------|--------|-----------|
| `nightly_golden_acc_gate` | Present | Scores golden Acc/F1 (mock oracle; X-34) |
| `bench_acc_at_n_regression_gate` | Present | Acc@40 floors JSON gate (X-35) |
| `unit_louvain_hierarchy_levels` | Present | Phase-2 when hierarchy enabled (D-54) |
| `contract_explain_trace_on_query_response` | Present | ExplainTrace field (X-21) |

---

## Cluster 08 — Dead code

| Test ID | Status | Assertion |
|---------|--------|-----------|
| Workspace build after deletion PR | Process | Green |
| `rg` call-site proof attached to PR | Process | Zero prod refs |
| `contract_cache_set_or_module_removed` | Backlog | D-52 CONFIRMED — gate not landed |

---

## Global invariants (any sprint)

1. No new `eval(` in tools  
2. No new `contains("429")` retry paths  
3. No new RLS policy with `tenant_id IS NULL OR`  
4. No contract test with vacuous `|| contains("source_id")` patterns  
5. Production defaults fail closed unless `EDGEQUAKE_DEV_MODE`
