# 08 — Code-is-Law Traceability Index

Every assessment claim maps to a concrete symbol. Use this when challenging the study.

---

## EdgeQuake — Ingestion

| ID | Claim | Path | Symbol |
|----|-------|------|--------|
| E-I01 | SDK insert entry | `edgequake-core/src/orchestrator/ingestion.rs` | `EdgeQuake::insert` |
| E-I02 | Pipeline factory | `edgequake-pipeline/src/ingestion_pipeline.rs` | `build_ingestion_pipeline` |
| E-I03 | Process stages | `edgequake-pipeline/src/pipeline/processing.rs` | `Pipeline::process` |
| E-I04 | Parallel extract | `edgequake-pipeline/src/pipeline/extraction.rs` | `extract_parallel` |
| E-I05 | JSON extractor | `edgequake-pipeline/src/extractor/llm.rs` | `LLMExtractor` |
| E-I06 | Gleaning | `edgequake-pipeline/src/extractor/gleaning.rs` | `GleaningExtractor` |
| E-I07 | Tuple prompts | `edgequake-pipeline/src/prompts/entity_extraction.rs` | `EntityExtractionPrompts` |
| E-I08 | Embeddings | `edgequake-pipeline/src/pipeline/helpers/embeddings.rs` | `generate_all_embeddings` |
| E-I09 | Rel embed format | same | relationship text `keywords\tsrc->tgt\n...` |
| E-I10 | Entity merge | `edgequake-pipeline/src/merger/entity.rs` | `merge_entities_batch` |
| E-I11 | Rel merge | `edgequake-pipeline/src/merger/relationship.rs` | `merge_relationships_batch` |
| E-I12 | Summarizer | `edgequake-pipeline/src/summarizer.rs` | `LLMSummarizer` |
| E-I13 | Persist saga | `edgequake-pipeline/src/persistence/ingestion_persister.rs` | `persist_processing_result_impl` |
| E-I14 | Chunk strategies (+ Semantic V) | `edgequake-pipeline/src/chunker/registry.rs` | `ChunkStrategy::Semantic` |
| E-I15 | Adaptive size | `edgequake-pipeline/src/adaptive_chunking.rs` | `calculate_adaptive_chunk_size` |
| E-I16 | API upload | `edgequake-api/src/handlers/documents/upload/file_upload.rs` | `upload_file` |
| E-I17 | Async worker | `edgequake-api/src/processor/text_insert/mod.rs` | `process_text_insert` |
| E-I18 | Community persist | `edgequake-storage/src/community_persist.rs` | `detect_and_persist_communities` |
| E-I19 | AGE graph | `edgequake-storage/src/adapters/postgres/graph/mod.rs` | `PostgresAGEGraphStorage` |
| E-I20 | pgvector | `edgequake-storage/src/adapters/postgres/vector/mod.rs` | `PgVectorStorage` |
| E-I21 | Graph quality metrics | `edgequake-storage/src/graph_metrics.rs` | `collect_graph_quality_metrics` |
| E-I22 | Process fingerprint | `edgequake-api/src/services/process_fingerprint.rs` | `ProcessFingerprintInput` |
| E-I23 | Delete rebuild lite | `edgequake-api/src/services/knowledge_rebuild.rs` | `apply_rebuild_to_properties` |
| E-I24 | MM orphan inject | `edgequake-pipeline/src/multimodal/injection.rs` | `inject_modality_relations` |
| E-I25 | Community report index | `edgequake-storage/src/community_reports.rs` | `index_community_reports_with_embedder` |
| E-I26 | TextEmbedder port | `edgequake-storage/src/traits/embedder.rs` | `TextEmbedder` |
| E-I27 | Role LLM matrix | `edgequake-core/src/llm_roles.rs` | `LlmRole`, `resolve_role_llm` |
| E-I28 | Community refresh extras | `community_index_service.rs` | `CommunityRefreshExtras`, `schedule_community_index_refresh_with_extras` |
| E-I29 | LlmTextEmbedder adapter | `edgequake-pipeline/.../text_embedder.rs` | `LlmTextEmbedder` |
| E-I30 | Summary role on merge | `summary_role.rs` + `text_insert/persist.rs` | `resolve_summary_llm_or_fallback` |
| E-I31 | Core ingest auto-embed | `edgequake-core/.../ingestion.rs` | `with_text_embedder(LlmTextEmbedder)` |

---

## EdgeQuake — Query

| ID | Claim | Path | Symbol |
|----|-------|------|--------|
| E-Q01 | Mode enum; Mix default | `edgequake-query/src/modes.rs` | `QueryMode` |
| E-Q02 | Global ≠ community reports | `modes.rs` + `tests/contract_global_mode_semantics.rs` | docs + contract |
| E-Q03 | Pipeline | `engine_impl/query_entry/query_pipeline.rs` | `run_query_pipeline` |
| E-Q04 | Dual embeddings | `engine_impl/mod.rs` | `QueryEmbeddings` |
| E-Q05 | Local | `engine_impl/modes/local.rs` | `query_local_with_vector_storage` |
| E-Q06 | Global | `engine_impl/modes/global.rs` | `query_global_with_vector_storage` |
| E-Q07 | Naive | `engine_impl/modes/naive.rs` | `query_naive_with_vector_storage` |
| E-Q08 | Hybrid | `engine_impl/modes/hybrid.rs` | `query_hybrid_with_vector_storage` |
| E-Q09 | Mix | `engine_impl/modes/mix.rs` | `query_mix_with_vector_storage` |
| E-Q10 | Hybrid merge RR | `hybrid_merge.rs` | `merge_hybrid_contexts` |
| E-Q11 | RRF | `fusion.rs` | `reciprocal_rank_fusion`, `RRF_K` |
| E-Q12 | BM25 fuse | `sparse_retrieval.rs` | `fuse_vector_and_bm25_chunks` |
| E-Q13 | Chunk pick | `engine_impl/modes/chunk_retrieval.rs` + `kg_chunk_pick.rs` | `append_score_ranked_chunks`, `KgChunkPickMethod` |
| E-Q14 | BFS / PPR hops (PPR default) | `graph_expand.rs`, `graph_ppr.rs` | `expand_neighborhood_edges`, `parse_graph_walk_mode` |
| E-Q15 | Community expand + reports | `community_global.rs` | `expand_global_context_with_communities`, `append_community_report_vector_chunks` |
| E-Q16 | Keywords | `keywords/llm_extractor.rs` | `LLMKeywordExtractor` |
| E-Q17 | Intent router (evidence-aligned) | `keywords/intent.rs` | `QueryIntent::recommended_mode` |
| E-Q18 | Truncation (dynamic remainder) | `truncation.rs` | `balance_context` |
| E-Q19 | Prompt | `engine_impl/prompt.rs` | `build_prompt` |
| E-Q20 | Bootstrap | `bootstrap.rs` | `build_production_query_engine` |
| E-Q21 | API query | `edgequake-api/src/handlers/query/query_execute.rs` | `execute_query` |
| E-Q22 | Path prune | `path_prune.rs` | `prune_relationships` |
| E-Q23 | GraphRAG-Bench harness | `eval/graphrag_levels.rs` | `run_spec046_bench_report` |
| E-Q24 | Role LLM query entry | `query_entry/query_workspace.rs` | `query_with_role_llms` |
| E-Q25 | Role LLM stream entry | `query_entry/query_stream.rs` | `query_stream_with_role_llms` |
| E-Q26 | Keyword role resolve (API) | `edgequake-api/.../query_execution.rs` | `resolve_workspace_keyword_llm` |
| E-Q27 | Dual-node PPR chunk pick | `kg_chunk_pick.rs` + `chunk_retrieval.rs` | `pick_chunks_by_entity_ppr` |
| E-Q28 | Postprocess DRY | `query_pipeline.rs` | `postprocess_retrieved_context` |
| E-Q29 | SOTA provider DRY | `query_execution.rs` | `resolve_sota_providers` |
| E-Q30 | Query LLM override (renamed) | `query_context.rs` | `resolve_query_llm_override` |

---

## LightRAG — Ingestion

| ID | Claim | Path | Symbol |
|----|-------|------|--------|
| L-I01 | Insert | `lightrag/lightrag.py` | `ainsert` |
| L-I02 | Enqueue/process | `lightrag/pipeline.py` | `apipeline_process_enqueue_documents` |
| L-I03 | Per-doc | `pipeline.py` | `process_single_document` |
| L-I04 | Multimodal | `pipeline.py` | `analyze_multimodal` |
| L-I05 | Chunk F/R/V/P | `parser/routing.py` | `resolve_chunk_options` |
| L-I06 | Extract | `operate.py` | `extract_entities` |
| L-I07 | Merge | `operate.py` | `merge_nodes_and_edges` |
| L-I08 | Summary | `operate.py` | `_handle_entity_relation_summary` |
| L-I09 | Delete rebuild | `operate.py` | `rebuild_knowledge_from_chunks` |
| L-I10 | Prompts | `prompt.py` | entity extraction prompts |
| L-I11 | Defaults | `constants.py` | `CHUNK_SIZE`, `MAX_GLEANING`, … |

---

## LightRAG — Query

| ID | Claim | Path | Symbol |
|----|-------|------|--------|
| L-Q01 | Router | `lightrag.py` | `aquery_llm` |
| L-Q02 | Default mix | `base.py` | `QueryParam.mode` |
| L-Q03 | KG query | `operate.py` | `kg_query` |
| L-Q04 | Naive | `operate.py` | `naive_query` |
| L-Q05 | Keywords | `operate.py` | `extract_keywords_only` |
| L-Q06 | Local data | `operate.py` | `_get_node_data` |
| L-Q07 | Global data | `operate.py` | `_get_edge_data` |
| L-Q08 | Vector ctx | `operate.py` | `_get_vector_context` |
| L-Q09 | Context build | `operate.py` | `_build_query_context` |
| L-Q10 | Chunk pick | `operate.py` / `constants.py` | `KG_CHUNK_PICK_METHOD` |
| L-Q11 | Rerank | `utils.py` | `process_chunks_unified` |
| L-Q12 | Token budget | `constants.py` | `DEFAULT_MAX_TOTAL_TOKENS=30000` |

---

## EdgeQuake — Ops / Storage / Defense / Observability (2026-07-10)

| ID | Claim | Path | Symbol |
|----|-------|------|--------|
| E-O01 | PG extension pins SSOT | `edgequake/docker/extension-pins.sh` | `EQ_POSTGRES_PROFILE` pg16/17/18 |
| E-O02 | HNSW DDL fail-closed | `.../vector/ddl.rs` | `create_table` ANN index `map_err` |
| E-O03 | HNSW defaults m=16 ef_c=32 | `.../postgres/config.rs` | `PostgresConfig::default` |
| E-O04 | ef_search + iterative_scan | `.../vector/search_tuning.rs` | `search_tuning_statements` |
| E-O05 | halfvec policy | `.../capabilities.rs` | `AnnIndexPolicy::resolve` |
| E-O06 | AGE ensure_indexes | `.../graph/helpers/graph_lifecycle.rs` | `ensure_indexes`, `bootstrap_concurrent_indexes` |
| E-O07 | Bounded community load | `community.rs` | `load_graph_bounded` |
| E-O08 | get_all_nodes (admin/legacy) | `.../graph/nodes_ops.rs` | `pg_get_all_nodes` (not community hot path) |
| E-O09 | sqlx + reconcile bootstrap | `.../migration_bootstrap/mod.rs` | `run_postgres_migrations` |
| E-O10 | Checksum repair M071/M078 | `reconcile/m071.rs`, `m078.rs` | `repair_migration_*_checksum_if_needed` |
| E-O11 | Readiness gating SSOT | `migration_bootstrap/mod.rs` | `is_ready_for_traffic` ← `readiness_blockers` |
| E-O12 | Saga compensation | `edgequake-storage/src/compensation.rs` | `compensate_merge_failure_with_kv` |
| E-O13 | StorageInspector + hourly | `edgequake-api/src/storage_inspector.rs` | `auto_repair_safe`, `spawn_hourly_monitor` |
| E-O14 | retry-chunks + graph merge | `handlers/documents/recovery/chunks.rs` | `retry_failed_chunks` → `KnowledgeGraphMerger::merge` |
| E-O15 | failed_chunks persist | `failed_chunks.rs` + extraction | `insert_failed_chunks` |
| E-O16 | Process fingerprint | `services/process_fingerprint.rs` | `fingerprint_is_stale` |
| E-O17 | Prometheus metrics | `edgequake-observability/src/metrics.rs` | `record_query_completed`, … |
| E-O18 | Graph quality + Prometheus | `graph_metrics.rs`, `metrics.rs` | `log_graph_quality` → `record_graph_quality` |
| E-O19 | Optional OTEL | `edgequake-observability/src/subscriber.rs` | `otel_enabled` |
| E-O20 | AGE RLS opt-in M081 | `migrations/support/081/apply.sql` | AGE ≥1.7 |
| E-O21 | Semantic fail-loud | `chunker/semantic.rs` | `SemanticChunking::chunk` |
| E-O22 | RLS session API | `.../postgres/rls.rs` | `acquire_rls_connection` |
| E-O23 | Health / ready | `handlers/health.rs`, `health_types.rs` | `HealthResponse`, `MigrationHealthSnapshot` |
| E-O24 | ANN readiness blocker | `migration_bootstrap` | `missing_hnsw_index` |
| E-O25 | iterative_scan relaxed | `search_tuning.rs` | `parse_hnsw_iterative_scan_mode` |
| E-O26 | Intent-gated Mix/Hybrid | `mix_weights.rs`, `modes/{mix,hybrid}.rs` | `resolve_arm_plan`, `resolve_hybrid_arm_plan` |
| E-O27 | Gleaning/concurrency clamps | `pipeline/config.rs`, admission | `clamp_max_gleaning`, `MAX_CONCURRENT_EXTRACTIONS_CAP` |
| E-O28 | Embed truncate policy | `helpers/embeddings.rs` | `EmbeddingTruncationPolicy`, `parse_embedding_truncation_policy` |
| E-O29 | Orphan KV compensate | `compensation.rs` | `compensate_orphan_kv` |
| E-O30 | QueryStats arm timings | `types.rs` QueryStats | `absorb_arm_metadata`, `arm_*_ms` |
| E-O31 | OTel GenAI / rag spans | `observability/rag_span.rs` | `with_rag_retrieval_span`, `RagRetrievalAttrs` |
| E-O32 | Popular-node telemetry | `retrieval_telemetry.rs`, local/global | `mark_popular_node_fallback` |
| E-O33 | Sparse/FTS outcome | `sparse_retrieval.rs` | `SparseRetrievalOutcome`, `fuse_vector_and_bm25_chunks` |
| E-O34 | RlsContext unexported | `postgres/mod.rs` | no `pub use … RlsContext` |
| E-O35 | PG matrix smoke | `e2e/run_ops17_perf_smoke.sh`, nightly workflow | `make ops17-smoke` |
| E-O36 | Drift SLO metrics | `storage_inspector.rs`, `metrics.rs` | `emit_drift_metrics`, `record_storage_drift` |
| E-O37 | Faithfulness sampler | `eval/faithfulness.rs` | `maybe_score_faithfulness`, `score_faithfulness_heuristic` |
| E-O38 | Ops runbooks | `13-OPS-RUNBOOKS.md` | upgrade / REINDEX / drift / retry |
| E-O39 | Arm/mode span wiring | `modes/arm_timed.rs`, `query_pipeline.rs` | `run_arm_timed`, `pipeline_retrieve` |
| E-O40 | LLM-judge faithfulness | `eval/faithfulness_judge.rs` | `score_faithfulness_llm`, `parse_judge_score` |
| E-O41 | ACC CI harness | `eval/acc_harness.rs` | `run_spec046_acc_report`, `AccReport` |
| E-O42 | PPR default walk | `graph_ppr.rs` | `parse_graph_walk_mode`, `GraphWalkMode::default=Ppr` |
| E-O43 | Mistral ACC live | `tests/e2e_spec046_ops_p3_acc.rs` | `e2e_ops_p3_mistral_small_embed_faithfulness_live` |
| E-O44 | ACC CI JSON artifact | `acc_harness.rs`, `e2e/run_spec046_acc.sh` | `write_spec046_acc_report_json`, `make spec046-acc` |
| E-O45 | Bipartite dual-node PPR | `graph_ppr.rs`, `kg_chunk_pick.rs` | `adjacency_from_bipartite`, `pick_chunks_by_bipartite_ppr` |
| E-O46 | Mini corpus retrieval ACC | `eval/graphrag_corpus.rs` | `run_spec046_corpus_acc_report`, `spec046_mini_corpus` |
| E-O47 | Science P4 e2e | `tests/e2e_spec046_science_p4.rs` | ACC artifact + bipartite + Mistral live |

---

## External Evidence (extended)

| ID | Claim | Source |
|----|-------|--------|
| X01 | Graphs hurt L1 / help L2+ | GraphRAG-Bench ICLR 2026 |
| X02 | HippoRAG2 denser + better multi-hop | arXiv:2502.14802 + GraphRAG-Bench tables |
| X03 | LightRAG dual-level design | arXiv:2410.05779v3 |
| X04 | Hybrid = dense+sparse+graph+RRF | 2026 Hybrid Search practice guides |
| X05 | pgvector + AGE pattern | Azure HorizonDB / unified Postgres Graph-RAG |
| X06 | iterative_scan for filtered ANN | pgvector 0.8.0 release (postgresql.org) |
| X07 | AGE needs explicit indexes | Microsoft Learn AGE performance (2026-01) |
| X08 | AGE 1.7 RLS + slow upgrade | apache/age PG17/PG18 v1.7.0 release notes |
| X09 | OTel GenAI + rag retrieval attrs | open-telemetry/semantic-conventions-genai |
| X10 | Faithfulness ≥0.9 production gate | RAG in Production 2026 guides |
| X11 | PG18 checksums default | PostgreSQL 18 / corruption literature |

---

## Cross-Ref: Assessment → Evidence

| Assessment statement | Evidence IDs |
|----------------------|--------------|
| EQ is LightRAG-class | E-Q01–09, L-Q01–08, X03 |
| Mix+RRF+BM25 is EQ advantage | E-Q11, E-Q12, X04 |
| Global ≠ MS GraphRAG reports (optional extractive) | E-Q02, E-Q15, E-I25, X03 |
| Intent router evidence-aligned | E-Q17, X01 |
| PPR available (default BFS) | E-Q14, X02 |
| Semantic chunk V opt-in | E-I14, L-I05 |
| Strong enterprise substrate | E-I19, E-I20, E-O01–O06, workspaces/RLS |
| Process fingerprint stale purge | E-I22, L-I02 |
| Role-LLM Keyword/Summary/Extract/Query/Vlm | E-I27, E-Q24 |
| Plan P0.1 rewire intent | E-Q17, X01 |
| Plan P1.1 PPR arm | X02, E-Q14 |
| HNSW + iterative_scan present | E-O03, E-O04, X06 |
| AGE indexes created by EQ | E-O06, X07 |
| Community O(N) smell | E-O07, E-O08 |
| retry-chunks not implemented | E-O14, E-O15 |
| Defense skeleton (saga+inspector) | E-O12, E-O13 |
| Observability incomplete for arms/graph | E-O17, E-O18, E-O19, X09 |
| Ops plan tickets | docs 09–12 / EQ-046-OPS-* |
