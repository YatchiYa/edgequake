# 02 — Cross-Reference Matrix (Code Is Law)

> Every claim in SPEC-115 must map to a code or measurement artifact.

## Spec / doc cross-refs

| Claim | Authority |
|-------|-----------|
| M ≠ U on EQ document card | [SPEC-108](../108-extraction-compared-light-rag/01-first-principles.md) LAW-X1; `pipeline/helpers/stats.rs` |
| Adaptive thresholds 1200/800/600 | `edgequake-pipeline/src/adaptive_chunking.rs` |
| Fair Acc pin 1200/100 | SPEC-001 fair pins; `EDGEQUAKE_ADAPTIVE_CHUNKING=0` |
| Extract caps 40/100 | EQ `prompts/extract_caps.rs`; LR `lightrag/constants.py` |
| LR default chunk 1200/100 | LR `CHUNK_SIZE` / paper; `chunking_by_token_size` defaults |
| LR P strategy default 2000 | `DEFAULT_CHUNK_P_SIZE` in `constants.py` |
| EQ PDF auto strategy | `IngestionPipelineOptions::for_pdf()` → `ChunkStrategy::Pdf` |
| Dual-SUT Mistral pins | `tools/bench001` Acc defaults; `lightrag_runner.py` |
| Paper chunk=1200, gleaning=1 | arXiv LightRAG HTML / PDF § experiments |

## Code SSOT table

| Concern | EdgeQuake | LightRAG |
|---------|-----------|----------|
| Adaptive size | `adaptive_chunking.rs` `calculate_adaptive_chunk_size` | **None** (fixed env/ctor) |
| Size applied at ingest | `text_insert/prepare.rs` `from_document_size(text_content.len())` | `pipeline.py` + `addon_params["chunker"]` |
| Default F chunker | `chunker/strategies.rs` TokenBased / Fixed | `chunker/token_size.py` `chunking_by_token_size` |
| Recursive R | `chunker/recursive.rs` | `chunker/recursive_character.py` |
| PDF page-aware | `chunker/page_aware.rs` + `for_pdf()` | Filename hint / strategy P/F/R/V |
| Extract entry | `extractor/llm.rs` + gleaning | `operate.py` `extract_entities` |
| Caps | `extract_caps.rs` | `constants.py` DEFAULT_MAX_EXTRACTION_* |
| Gleaning default | max 1; local oft disabled | `DEFAULT_MAX_GLEANING = 1` |
| Mention sum M | `stats.rs` += `entities.len()` | No EQ-style card M |
| Unique U | merger → AGE | `_merge_nodes_then_upsert` → graph KV |

## External / research cross-refs (Aug 2026)

| Source | Use |
|--------|-----|
| LightRAG paper (chunk=1200) | Baseline geometry |
| HKUDS FileProcessingPipeline.md | Strategy F/R/V/P + env precedence |
| GraphRAG hybrid guides (osFoundry / VentureBeat 2025–26) | Density without precision hurts |
| arXiv:2601.14123 chunking study | Overlap often cost-only; context cliff ~2.5k |

## Measurement artifacts (this pack)

| File | Content |
|------|---------|
| `measurements/geometry_results.json` | Real LR F chunker N @ 1200/800/600 |
| `measurements/lightrag_live.json` | Live LR Mistral insert: N, U, edges |
| `measurements/edgequake_live.json` | Live EQ product + fair arms |
| `measurements/SUMMARY.md` | One-page scoreboard |
