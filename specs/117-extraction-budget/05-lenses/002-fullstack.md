# Lens — Full-Stack Engineer

## SSOT

`ExtractionCaps::resolve(workspace, document)` in pipeline. Core `apply_extract_budget_metadata`. No K math in UI.

## Touch points

1. `extract_caps.rs` — resolve, validate, ranked prompt  
2. Prompt builders take `ExtractionCaps` (not only `from_env`)  
3. Gleaning continue when capped  
4. `IngestionPipelineOptions.extraction_caps`  
5. Workspace DTOs + in-memory + Postgres  
6. Document admission fields  
7. OpenAPI → web types → payload  

## SOLID

- SRP: `extract_budget_metadata.rs` separate from chunking  
- OCP: Inherit default preserves old path  
- DIP: Factory injects resolved caps into options  

## Anti-patterns

- Duplicate resolve in API and pipeline  
- Soft-only caps without hard truncate  
- Env mutate in concurrent tests without mutex
