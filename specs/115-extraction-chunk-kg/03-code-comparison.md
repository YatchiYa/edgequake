# 03 — Code Comparison (Chunk + Extract)

> Paths: EdgeQuake repo-relative; LightRAG = `/Users/raphaelmansuy/Github/03-working/LightRAG`.

## End-to-end

```ascii
  ┌──────────── EQ product ────────────┐     ┌────────── LightRAG default ──────────┐
  │ text_content.len()                 │     │ CHUNK_SIZE=1200 (or ctor)            │
  │   → adaptive → 800 (61KB gold)     │     │   → F chunking_by_token_size         │
  │   → strategy Pdf if PDF source     │     │   → overlap 100                      │
  │   → N ≈ 20 (F@800 measured)        │     │   → N = 13 (F@1200 measured)         │
  │   → extract ≤40 ents, glean≤1      │     │   → extract ≤40 ents, glean=1        │
  │   → stats M = sum mentions         │     │   → merge → unique graph U           │
  │   → merge → AGE U                  │     │                                      │
  └────────────────────────────────────┘     └──────────────────────────────────────┘
```

## Adaptive vs fixed (product confound)

```16:24:edgequake/crates/edgequake-pipeline/src/adaptive_chunking.rs
pub fn calculate_adaptive_chunk_size(document_size_bytes: usize) -> usize {
    if document_size_bytes > 100_000 {
        600
    } else if document_size_bytes > 50_000 {
        800
    } else {
        1200
    }
}
```

Ingest applies **extracted text length**, not PDF file bytes:

```207:211:edgequake/crates/edgequake-api/src/processor/text_insert/prepare.rs
        let ingestion_options =
            edgequake_pipeline::IngestionPipelineOptions::from_document_size(text_content.len())
                .with_gleaning(enable_gleaning, max_gleaning)
                .with_allow_local_gleaning(allow_local_gleaning)
                .with_chunk_strategy(chunk_strategy);
```

PDF sources flip strategy to Pdf unless overridden:

```215:223:edgequake/crates/edgequake-api/src/processor/text_insert/prepare.rs
        let source_is_pdf = source_type.eq_ignore_ascii_case("pdf")
            || source_type.eq_ignore_ascii_case("pdf_upload")
            || text_content.contains(edgequake_pipeline::PAGE_MARKER_PREFIX);
        let ingestion_options =
            if source_is_pdf && chunk_strategy == edgequake_pipeline::ChunkStrategy::default() {
                ingestion_options.for_pdf()
            } else {
                ingestion_options
            };
```

## LightRAG defaults (code)

| Knob | Default | Location |
|------|---------|----------|
| `chunk_token_size` | 1200 | `chunking_by_token_size(..., chunk_token_size=1200)` |
| `chunk_overlap_token_size` | 100 | same |
| `DEFAULT_MAX_GLEANING` | 1 | `constants.py` |
| `DEFAULT_MAX_EXTRACTION_ENTITIES` | 40 | `constants.py` |
| `DEFAULT_MAX_EXTRACTION_RECORDS` | 100 | `constants.py` |
| `DEFAULT_CHUNK_P_SIZE` | 2000 | P strategy only — **not** default F |

## EQ defaults trap

| Layer | Size | Notes |
|-------|------|-------|
| `ChunkerConfig::default()` | **800**/100 | Embed-safety legacy |
| Adaptive fixed-off env | **1200**/100 | Fair Acc |
| Adaptive ON + gold MD 61KB | **800**/~66 | Product |
| Adaptive ON + text &gt;100KB | **600**/~50 | Large MD |

## Mention vs unique (EQ)

```85:87:edgequake/crates/edgequake-pipeline/src/pipeline/helpers/stats.rs
        stats.entity_count += extraction.entities.len();
        stats.relationship_count += extraction.relationships.len();
```

## Caps parity

```11:15:edgequake/crates/edgequake-pipeline/src/prompts/extract_caps.rs
pub const DEFAULT_MAX_EXTRACTION_ENTITIES: usize = 40;
pub const DEFAULT_MAX_EXTRACTION_RECORDS: usize = 100;
```

Matches LightRAG `constants.py` lines 26–27.

## Geometry measurement (this paper, real LR F chunker)

| Pin | N | min/avg/max tokens |
|-----|--:|--------------------|
| 1200/100 | **13** | 955 / 1181 / 1200 |
| 800/66 | **20** | 209 / 770 / 801 |
| 600/50 | **26** | 405 / 592 / 600 |

Doc: 61 353 chars, **14 156** tiktoken tokens (gold MD).
