# 04 — Target Architecture

## Metadata keys

| Key | Values |
|-----|--------|
| `chunking_mode` | absent/`inherit` \| `adaptive` \| `fixed` |
| `chunk_token_size` | usize ≥ 1 (Fixed; default 1200) |
| `chunk_overlap_token_size` | 0 ≤ ov &lt; size (Fixed; default 100) |

Clear / inherit: remove keys (same pattern as extraction language).

## API surface

`CreateWorkspaceRequest` / `UpdateWorkspaceRequest` / response:

- `chunking_mode: Option<String>`
- `chunk_token_size: Option<u32>`
- `chunk_overlap_token_size: Option<u32>`

Validation 400 when Fixed and `overlap >= size`, or size == 0.

## Pipeline types

```rust
pub enum ChunkingPolicy {
    Inherit,
    Adaptive,
    Fixed { size: usize, overlap: usize },
}
```

`IngestionPipelineOptions.chunking_policy: Option<ChunkingPolicy>`  
`build_chunker_config(..., policy: Option<&ChunkingPolicy>, ...)`

## Precedence

```ascii
  1. Start from policy:
       Inherit   → env adaptive on/off + env fixed sizes
       Adaptive  → always adaptive thresholds (ignore env off)
       Fixed     → (size, overlap) from workspace (defaults 1200/100)
  2. Small-doc floor when effective adaptive + non-Fixed strategy + ≤50KB → max(800)
  3. Document ChunkOptions.apply_to_config — LAST
```

## Worker inject

`prepare.rs` reads workspace metadata → `ChunkingPolicy` → `IngestionPipelineOptions::with_chunking_policy` before document `chunk_options`.
