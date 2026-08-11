# Lens — Full Stack Developer

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| SRP | `ChunkingPolicy` resolve ≠ metadata apply ≠ UI |
| OCP | New modes extend enum; thresholds stay in one fn |
| DIP | API depends on pipeline policy type, not env scraping in handlers |
| DRY | No duplicate 50KB/100KB bands in TS or API |

## Touch list

1. `adaptive_chunking.rs` + `ingestion_pipeline.rs`
2. `helpers.rs` `apply_chunking_metadata`
3. `workspace_ops.rs` + request/response DTOs
4. `prepare.rs` / `workspace_pipeline_factory.rs`
5. OpenAPI codegen → web types
6. `WorkspaceChunkingCard` + wizard draft payload

## Anti-patterns

- Reading env in the React card
- Storing only booleans without size when Fixed
- Skipping validation of overlap ≥ size
