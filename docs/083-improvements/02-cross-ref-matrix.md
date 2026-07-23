# SPEC-083 — Cross-Reference Matrix

> Defect ↔ locus ↔ law ↔ sprint ↔ tests. Generated from defect catalog.

| ID | Laws | Sprint | Primary locus | Tests (names) | Related |
|----|------|--------|---------------|---------------|---------|
| [P0](defects/P0.md) | LAW-2 | 0 | `migrations/support/092/apply.sql; graph_lifecycle.rs ensure_indexes; nodes_ops/r` | e2e_schema_ready_refuses_traffic; e2e_degrees_match_property | X-03, C-20, D-30 |
| [X-03](defects/X-03.md) | LAW-2 | 0 | `nodes_ops/read.rs:148-171; edges_ops.rs:360-365; scan_ops.rs:321-448` | contract_incident_edges_fallback; e2e_chat_local_mode_withou | P0 |
| [C-14](defects/C-14.md) | LAW-6 | 2 | `edgequake-storage/src/entity_id.rs:198-219` | unit_normalize_THE_COMPANY; unit_normalize_curly_apostrophe; | D-32,X-17 |
| [C-17](defects/C-17.md) | LAW-3 | 3 | `extractor/gleaning.rs:199-205 vs extractor/llm.rs complete_with_options` | contract_gleaning_uses_completion_options | X-07 |
| [C-18](defects/C-18.md) | LAW-3 | 2 | `pipeline/extraction.rs:302; config allows 0` | e2e_chunk_max_retries_zero_still_attempts_once_or_rejects | X-06 |
| [D-50](defects/D-50.md) | LAW-4 | 1 | `.env.example:36` | contract_env_example_vision_not_openai_by_default | S-09 |
| [S-01](defects/S-01.md) | LAW-1,LAW-4 | 1 | `edgequake-api/src/handlers/websocket.rs:53-66,166-256` | e2e_ws_tenant_a_never_sees_tenant_b; contract_progress_event | S-02,C-24,X-23 |
| [S-02](defects/S-02.md) | LAW-1,LAW-4 | 1 | `pdf_upload/status.rs:262-278; websocket.rs cancel; task_scope.rs:9-29 (REST OK)` | e2e_cancel_foreign_track_id_404; e2e_pdf_progress_foreign_40 | S-01 |
| [S-03](defects/S-03.md) | LAW-1 | 1 | `migrations/001_init_database.sql:434-444; rls.rs:220-254; conversation.rs acquir` | e2e_rls_guc_visible_on_following_insert; e2e_owner_forced_rl | S-04,S-05,S-06,X-37 |
| [S-04](defects/S-04.md) | LAW-1,LAW-4 | 1 | `001_init_database.sql:507-516; support/081 AGE policies; 085 mm_assets` | e2e_null_tenant_row_invisible; contract_policy_has_no_null_o | S-03,S-06 |
| [S-05](defects/S-05.md) | LAW-1,LAW-3 | 1 | `001 set_tenant_context app.current_*; 012 app.tenant_id; support/081 edgequake.t` | contract_single_guc_namespace; e2e_age_rls_sees_app_current | S-03,X-37 |
| [S-06](defects/S-06.md) | LAW-1 | 1 | `migrations/082_add_document_originals.sql` | e2e_document_originals_cross_workspace_denied | S-03,S-04 |
| [S-07](defects/S-07.md) | LAW-4 | 1 | `edgequake-auth/src/jwt.rs:75-91,162-168` | e2e_logout_rejects_access_jti; contract_jwt_requires_iss_aud | S-08,S-09 |
| [S-08](defects/S-08.md) | LAW-4 | 1 | `edgequake-auth/src/types.rs:26-30; jwt.rs role()` | contract_unknown_role_rejected | S-07 |
| [S-09](defects/S-09.md) | LAW-4 | 1 | `startup_security.rs:39-65; auth config DEFAULT_INSECURE_JWT_SECRET` | contract_startup_rejects_default_secret; e2e_dev_mode_banner | S-10 |
| [S-10](defects/S-10.md) | LAW-4 | 1 | `server.rs:74-94; middleware.rs:553-557` | contract_cors_default_fail_closed_prod; e2e_ws_missing_origi | S-09 |
| [S-11](defects/S-11.md) | LAW-4,LAW-3 | 1 | `middleware.rs:630-638; limiter.rs cleanup never called` | e2e_rate_limit_ignores_spoofed_header; contract_cleanup_sche | S-01 |
| [S-12](defects/S-12.md) | LAW-4 | 1 | `file_upload.rs:63-67; file_validation.rs:111-151; pdf magic only` | contract_filename_strips_path; e2e_exe_as_pdf_rejected | D-51,D-44 |
| [S-13](defects/S-13.md) | LAW-4 | 1 | `tools/bench047/bench047/mmlongbench_eval_score.py:137-179` | contract_no_eval_in_bench047; unit_literal_eval_lists | — |
| [X-06](defects/X-06.md) | LAW-5 | 3 | `embeddings.rs` full jitter + `CircuitBreakerOpen` | unit_retry_has_jitter; test_pipeline_circuit_breaker_is_retryable | X-07,X-30 |
| [X-07](defects/X-07.md) | LAW-5 | 3 | `pipeline/helpers/embeddings.rs:204; LlmError::retry_strategy unused` | contract_no_substring_retry_matching; unit_typed_429 | X-06,X-30 |
| [X-10](defects/X-10.md) | LAW-3 | 3 | `LLM/pipeline embeddings; Ollama vs OpenAI` | e2e_ollama_cosine_after_l2; unit_normalize | C-28,X-04 |
| [X-16](defects/X-16.md) | LAW-3 | 1 | `see register page-6 / cluster doc` | contract_x_16; e2e_x_16 | C-17 |
| [X-28](defects/X-28.md) | LAW-2 | 2 | `PipelineCheckpoint hash first 65536 bytes truncated` | e2e_checkpoint_rejects_suffix_change | X-29 |
| [X-29](defects/X-29.md) | LAW-2 | 2 | `tasks update_task / mark_success` | e2e_cancelled_cannot_mark_success; e2e_optimistic_lock | C-23 |
| [X-30](defects/X-30.md) | LAW-5 | 3 | `ingestion_reliability` typed timeout markers; residual string taxonomy (**PARTIAL**) | unit_failure_class_typed; contract_no_substring_retry_matching | X-06,X-07 |
| [X-35](defects/X-35.md) | LAW-3 | 5 | `specs/001-benchmark; SPEC-055` | bench_acc_at_n_regression_gate | D-38,D-30,C-14,D-54,X-34 |
| [X-37](defects/X-37.md) | LAW-1,LAW-3 | 1 | `vectors per-table; graph properties; relational RLS inert; KV prefix only` | e2e_kv_cross_tenant_denied; e2e_vector_table_suffix_collisio | S-03,S-05,S-01 |
| [C-15](defects/C-15.md) | LAW-3 | 2 | `chunker/page_aware.rs:155-177; markdown_chunking.rs:30-54` | e2e_page_aware_offsets_rebase; e2e_markdown_block_offsets | X-13 |
| [C-16](defects/C-16.md) | LAW-2 | 2 | `chunker/recursive.rs:384-390` | e2e_huge_table_splits; unit_atomic_respects_max | X-08,X-18 |
| [C-20](defects/C-20.md) | LAW-8 | 0 | `contract_spec054_query_postgres_perf.rs:103-109` | contract_native_upsert_eq_arbiter | P0,X-03 |
| [C-21](defects/C-21.md) | LAW-3 | 3 | `chunk_content.rs:30-42; KVStorage::get_by_ids exists` | contract_batch_fetch_uses_get_by_ids; bench_chunk_fetch | — |
| [C-22](defects/C-22.md) | LAW-2 | 2 | `adapters/postgres/kv.rs:257-289` | e2e_kv_upsert_all_or_nothing | C-23 |
| [C-23](defects/C-23.md) | LAW-3 | 2 | `document_reingest.rs:65-71 vs status_updates completed→indexed; reprocess_admiss` | e2e_dedup_matches_completed_and_indexed | X-29 |
| [C-24](defects/C-24.md) | LAW-3 | 1 | `websocket.rs:573-580; websocket_types Deletion*` | contract_matches_track_id_deletion_variants | S-01,X-23 |
| [C-27](defects/C-27.md) | LAW-3 | 4 | `tenant_manager.rs:254-259,480-487` | unit_tenant_cache_lru_touch | — |
| [C-28](defects/C-28.md) | LAW-4 | 2 | `edgequake-core/src/types/embedding.rs:83-88` | unit_cosine_dim_mismatch_is_err | X-10 |
| [D-30](defects/D-30.md) | LAW-3,LAW-6 | 2 | `edges_ops.rs ON CONFLICT (eq_source_id, eq_target_id)` | e2e_multigraph_two_rel_types_persist | D-31,C-20 |
| [D-31](defects/D-31.md) | LAW-3 | 2 | `merger/relationship.rs:619-627; divergent vector/graph policies` | unit_weight_associative; contract_single_weight_policy | D-30,D-37 |
| [D-32](defects/D-32.md) | LAW-3 | 2 | `merger/entity.rs keep first type; update_entity_node never writes entity_type` | e2e_entity_type_conflict_logged_and_resolved | C-14,X-15 |
| [D-33](defects/D-33.md) | LAW-7 | 2 | `merger/entity.rs:430-440; merge_limits` | e2e_lineage_includes_docs_beyond_source_cap | C-26 |
| [D-34](defects/D-34.md) | LAW-3 | 3 | `description_merge.rs:213-224; summarizer.rs:202-207` | unit_needs_llm_always_summarizes; unit_jaccard_normalized | D-53 |
| [D-37](defects/D-37.md) | LAW-3 | 4 | `relevancy_prune; graph_ppr; fusion RRF; mix minmax` | unit_score_scale_no_cross_compare | D-39,D-35 |
| [D-38](defects/D-38.md) | LAW-3 | 3 | `query_pipeline.rs:432-487` | e2e_query_vec_matches_question_only_embedding | D-39,X-20 |
| [D-39](defects/D-39.md) | LAW-3 | 3 | `sparse_retrieval / chunk_retrieval preserve_order` | e2e_min_score_enforced_on_rrf | D-37,D-36 |
| [D-40](defects/D-40.md) | LAW-3 | 4 | `query_types / edgequake-query types / core query stats` | contract_stream_stats_superset | X-21,X-22 |
| [D-41](defects/D-41.md) | LAW-3 | 4 | `progress/mod.rs equal stage weights` | unit_progress_weighted | D-42 |
| [D-42](defects/D-42.md) | LAW-3 | 4 | `progress.rs avg_item_time_ms serde(skip); HashMap process-local` | e2e_progress_survives_restart | D-41 |
| [D-44](defects/D-44.md) | LAW-3 | 4 | `DefaultBodyLimit 50MiB; validation.rs 100MiB dead; messages mention 10MB` | contract_upload_limit_ssot_50mib | S-12,D-51 |
| [D-45](defects/D-45.md) | LAW-2,LAW-3 | 2 | `001,012,docker/init,specs schema; unbounded audit channel` | e2e_audit_insert_next_month_partition; contract_single_audit | — |
| [D-48](defects/D-48.md) | LAW-8 | 4 | `sdks/*/.github/workflows/` | contract_no_nested_github_workflows_or_root_dispatch | X-33,D-49 |
| [D-49](defects/D-49.md) | LAW-8 | 4 | `Makefile; scripts/bump-version.sh` | contract_no_sed_i_empty_string | D-48 |
| [D-51](defects/D-51.md) | LAW-2 | 4 | `pdf_upload/upload.rs:113-119; file_upload.rs` | e2e_batch_file_cap; e2e_upload_streams_to_temp | D-44,S-12 |
| [D-52](defects/D-52.md) | LAW-3 | 4 | `pipeline/cache.rs:358 TODO` | contract_cache_set_or_module_removed | dead-code §5 |
| [D-53](defects/D-53.md) | LAW-3 | 3 | `embeddings ~2.5; text_utils len/4; summarizer; tiktoken in workspace unused by p` | unit_token_estimator_ssot; e2e_chunk_size_respects_tokenizer | D-34,X-08 |
| [X-01](defects/X-01.md) | LAW-3 | 4 | `migrations 001/002/026` | contract_tasks_pk_documented | X-02 |
| [X-02](defects/X-02.md) | LAW-2 | 4 | `state/migration_bootstrap` | contract_checksum_drift_fails_loud | X-01,P0 |
| [X-05](defects/X-05.md) | LAW-3,LAW-8 | 4 | `fts.rs` | e2e_fts_language_config | D-36 |
| [X-08](defects/X-08.md) | LAW-3 | 3 | `safety_limits; wrapper; Makefile Mistral 16` | contract_embed_batch_ssot | D-53,C-16 |
| [X-09](defects/X-09.md) | LAW-3 | 4 | `Cargo.lock; pdf2md` | contract_single_edgequake_llm_version | — |
| [X-11](defects/X-11.md) | LAW-3 | 5 | `processor/task_impl.rs` | e2e_reindex_embedding_model_change | X-35 |
| [X-13](defects/X-13.md) | LAW-3 | 3 | `see register page-6 / cluster doc` | contract_x_13; e2e_x_13 | C-15 |
| [X-14](defects/X-14.md) | LAW-3 | 3 | `see register page-6 / cluster doc` | contract_x_14; e2e_x_14 | D-53 |
| [X-17](defects/X-17.md) | LAW-3 | 4 | `see register page-6 / cluster doc` | contract_x_17; e2e_x_17 | C-14 |
| [X-18](defects/X-18.md) | LAW-3 | 3 | `see register page-6 / cluster doc` | contract_x_18; e2e_x_18 | C-16,X-08 |
| [X-19](defects/X-19.md) | LAW-3 | 3 | `see register page-6 / cluster doc` | contract_x_19; e2e_x_19 | X-06 |
| [X-20](defects/X-20.md) | LAW-3 | 3 | `see register page-6 / cluster doc` | contract_x_20; e2e_x_20 | D-38 |
| [X-23](defects/X-23.md) | LAW-3 | 1 | `see register page-6 / cluster doc` | contract_x_23; e2e_x_23 | S-01,C-24 |
| [X-24](defects/X-24.md) | LAW-3 | 4 | `see register page-6 / cluster doc` | contract_x_24; e2e_x_24 | — |
| [X-25](defects/X-25.md) | LAW-3 | 4 | `see register page-6 / cluster doc` | contract_x_25; e2e_x_25 | X-26 |
| [X-27](defects/X-27.md) | LAW-3 | 4 | `see register page-6 / cluster doc` | contract_x_27; e2e_x_27 | S-01 |
| [X-31](defects/X-31.md) | LAW-2 | 4 | `worker.rs / main server path` | e2e_shutdown_drains_or_cancels_within_budget | X-29 |
| [X-32](defects/X-32.md) | LAW-8 | 4 | `ci.yml` audit blocking; postgres/e2e still `continue-on-error` (**PARTIAL**) | contract_ci_no_continue_on_error_critical (Backlog); contract_frontend_test_must_run (Backlog) | D-48,X-34 |
| [X-33](defects/X-33.md) | LAW-3,LAW-8 | 4 | `sdks/*/version` | contract_sdk_major_matches_server_policy | D-48,X-26 |
| [X-34](defects/X-34.md) | LAW-8 | 5 | `tests/fixtures/spec025_golden_qa.json; skeleton metrics` | nightly_golden_acc_gate | X-35,X-32 |
| [X-36](defects/X-36.md) | LAW-3 | 4 | `core/config.rs; EdgeQuakeConfig; Workspace resolution` | contract_config_precedence | D-50 |
| [C-19](defects/C-19.md) | LAW-8 | 4 | `workspace_vector.rs:204-226; workspace_crud.rs:515-518` | resource_safety_proof drop_workspace_table contracts | — |
| [C-25](defects/C-25.md) | LAW-3 | 4 | `edgequake-llm anthropic.rs:908-912; traits ImageData::from_url` | unit_anthropic_url_image_source | — |
| [C-26](defects/C-26.md) | LAW-3,LAW-7 | 2 | `entity.rs:50; relationship.rs:65; live cap merge_limits 200` | contract_single_source_id_cap | D-33 |
| [D-35](defects/D-35.md) | LAW-3,LAW-8 | 4 | `modes/mix.rs:328-334; QueryEngineConfig docs` | contract_mix_fusion_semantics_documented | D-36,D-37 |
| [D-36](defects/D-36.md) | LAW-3 | 4 | `sparse_retrieval.rs:161-173` | contract_fusion_mode_names | D-35,D-39 |
| [D-46](defects/D-46.md) | LAW-3 | 4 | `observability/subscriber.rs:117-124` | contract_otel_respects_rust_log | — |
| [D-47](defects/D-47.md) | LAW-8 | 4 | `Makefile db-start; AGENTS/CONTRIBUTING say postgres-start` | contract_makefile_has_postgres_start_alias | — |
| [D-54](defects/D-54.md) | LAW-3 | 5 | `storage/community.rs:315-384` | unit_louvain_hierarchy_levels | X-35 |
| [X-04](defects/X-04.md) | LAW-8 | 4 | `vector/capabilities.rs` | contract_vector_metric_cosine_only | X-10 |
| [X-12](defects/X-12.md) | LAW-8 | 4 | `pdf concurrency match 0..=49=>2 ... =>2` | contract_pdf_concurrency_schedule | — |
| [X-15](defects/X-15.md) | LAW-8 | 4 | `see register page-6 / cluster doc` | contract_x_15; e2e_x_15 | D-32 |
| [X-21](defects/X-21.md) | LAW-3 | 5 | `see register page-6 / cluster doc` | contract_x_21; e2e_x_21 | D-40,X-35 |
| [X-22](defects/X-22.md) | LAW-3 | 3 | `see register page-6 / cluster doc` | contract_x_22; e2e_x_22 | D-40 |
| [X-26](defects/X-26.md) | LAW-3 | 4 | `see register page-6 / cluster doc` | contract_x_26; e2e_x_26 | X-25,X-33 |
