# 03 — Code As-Is (post SPEC-117)

## Caps SSOT

```ascii
  document API (pair)  ──┐
  workspace metadata   ──┼─► ExtractionCaps::resolve_for_ingestion
  fleet env / 40/100   ──┘              │
                                        ▼
                    IngestionPipelineOptions.extraction_caps
                                        │
                    ┌───────────────────┼───────────────────┐
                    ▼                   ▼                   ▼
              LLMExtractor        GleaningExtractor     SOTAExtractor
              JSON prompts        continue section      tuple prompts
              JSON parser         hard truncate         Hybrid parser
```

- Soft: ranked `prompt_quantity_limits_section` (“do not fill”)  
- Hard: relation-aware truncate (default) — prefer degree>0 entities, then orphans;
  drop orphan rels; trim rels by weight under total. Acc/LR FIFO via
  `EDGEQUAKE_EXTRACT_CAPS_SELECTION=fifo`  
- Metadata: `extract_caps_applied` (+ `selection`) when truncated  
- Precedence (LAW-117-2): document > workspace > env > 40/100  

## Upload paths (wired)

| Path | Caps source |
|------|-------------|
| Text JSON | `extract_max_*` fields → `parse_document_extract_caps` |
| File multipart | form fields + metadata envelope (`MultipartUploadFields`) |
| Batch multipart | same SSOT helper as file |

## Workspace

`extract_budget_mode` / `extract_max_*` on create/update/response; metadata keys via `extract_budget_metadata` (Postgres + in-memory).
