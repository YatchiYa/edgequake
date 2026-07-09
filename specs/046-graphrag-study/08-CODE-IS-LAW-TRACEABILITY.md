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
| E-I14 | Chunk strategies | `edgequake-pipeline/src/chunker/registry.rs` | `ChunkStrategy` |
| E-I15 | Adaptive size | `edgequake-pipeline/src/adaptive_chunking.rs` | `calculate_adaptive_chunk_size` |
| E-I16 | API upload | `edgequake-api/src/handlers/documents/upload/file_upload.rs` | `upload_file` |
| E-I17 | Async worker | `edgequake-api/src/processor/text_insert/mod.rs` | `process_text_insert` |
| E-I18 | Community persist | `edgequake-storage/src/community_persist.rs` | `detect_and_persist_communities` |
| E-I19 | AGE graph | `edgequake-storage/src/adapters/postgres/graph/mod.rs` | `PostgresAGEGraphStorage` |
| E-I20 | pgvector | `edgequake-storage/src/adapters/postgres/vector/mod.rs` | `PgVectorStorage` |

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
| E-Q13 | Chunk pick | `engine_impl/modes/chunk_retrieval.rs` | `append_score_ranked_chunks` |
| E-Q14 | BFS hops | `graph_hops.rs` | `edges_within_depth` |
| E-Q15 | Community expand | `community_global.rs` | `expand_global_context_with_communities` |
| E-Q16 | Keywords | `keywords/llm_extractor.rs` | `LLMKeywordExtractor` |
| E-Q17 | Intent router | `keywords/intent.rs` | `QueryIntent::recommended_mode` |
| E-Q18 | Truncation | `truncation.rs` | `balance_context` |
| E-Q19 | Prompt | `engine_impl/prompt.rs` | `build_prompt` |
| E-Q20 | Bootstrap | `bootstrap.rs` | `build_production_query_engine` |
| E-Q21 | API query | `edgequake-api/src/handlers/query/query_execute.rs` | `execute_query` |

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

## External Evidence

| ID | Claim | Source |
|----|-------|--------|
| X01 | Graphs hurt L1 / help L2+ | GraphRAG-Bench ICLR 2026 |
| X02 | HippoRAG2 denser + better multi-hop | arXiv:2502.14802 + GraphRAG-Bench tables |
| X03 | LightRAG dual-level design | arXiv:2410.05779v3 |
| X04 | Hybrid = dense+sparse+graph+RRF | 2026 Hybrid Search practice guides |
| X05 | pgvector + AGE pattern | Azure HorizonDB Graph-Augmented RAG |

---

## Cross-Ref: Assessment → Evidence

| Assessment statement | Evidence IDs |
|----------------------|--------------|
| EQ is LightRAG-class | E-Q01–09, L-Q01–08, X03 |
| Mix+RRF+BM25 is EQ advantage | E-Q11, E-Q12, X04 |
| Global ≠ community reports | E-Q02, X03 |
| Intent router misaligned | E-Q17, X01 |
| No PPR | E-Q14 vs X02 |
| Missing semantic chunk V | E-I14 vs L-I05 |
| Strong enterprise substrate | E-I19, E-I20, workspaces/RLS |
| Plan P0.1 rewire intent | E-Q17, X01 |
| Plan P1.1 PPR arm | X02, E-Q14 |
