# 03 — Code As-Is (before SPEC-116)

## Resolve (env only)

```ascii
  EDGEQUAKE_ADAPTIVE_CHUNKING (default ON)
       │
       ├─ ON  → calculate_adaptive_chunk_size(bytes) + ~8.3% overlap
       └─ OFF → EDGEQUAKE_CHUNK_SIZE / OVERLAP (1200/100)
       │
       ▼
  build_chunker_config → optional ChunkOptions.apply_to_config (LAST)
```

No workspace argument. [`WorkspacePipelineFactory`](../../edgequake/crates/edgequake-api/src/workspace_pipeline_factory.rs) injects LLM/schema/language — **not** chunking.

## What already exists

| Layer | Chunk control? |
|-------|----------------|
| Fleet env | Yes |
| Workspace API/UI | **No** |
| Document upload `chunk_options` | Yes (API; WebUI silent) |
| Acc client | Env off + upload options |

## Gap

Partner cannot pin Acc-fair per workspace without ops or custom upload clients.
